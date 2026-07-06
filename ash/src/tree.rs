use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::{self, ExecuteRequest, ExecuteResponse};
use crate::eval::{EvalError, Evaluator};
use crate::interpolation::Interpolation;
use crate::parser::parse_str;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskKind {
    Markdown,
    Ash,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub path: PathBuf,
    pub kind: TaskKind,
    pub content: String,
    pub prompt: String,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub compact: Option<String>,
    pub on_fail: String,
}

#[derive(Debug, Clone)]
pub struct WalkConfig {
    pub root: PathBuf,
    pub dry_run: bool,
    pub continue_on_error: bool,
    pub default_agent: String,
    pub default_model: String,
}

impl Default for WalkConfig {
    fn default() -> Self {
        WalkConfig {
            root: PathBuf::new(),
            dry_run: false,
            continue_on_error: false,
            default_agent: "echo".to_string(),
            default_model: String::new(),
        }
    }
}

fn extract_numeric_prefix(name: &str) -> Option<u64> {
    let digits: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn file_ext(name: &str) -> Option<&str> {
    if name.ends_with(".md") {
        Some(".md")
    } else if name.ends_with(".ash") {
        Some(".ash")
    } else {
        None
    }
}

fn is_task_file(name: &str) -> bool {
    if let Some(ext) = file_ext(name) {
        let stem = name.strip_suffix(ext).unwrap_or(name);
        extract_numeric_prefix(stem).is_some()
    } else {
        false
    }
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }
    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("\n---") {
        let fm_block = &after_first[..end];
        let body_start = end + 4;
        let body = &after_first[body_start..];
        let fm = parse_frontmatter_lines(fm_block);
        return (Some(fm), body);
    }
    if let Some(end) = after_first.find("\n---\n") {
        let fm_block = &after_first[..end];
        let body_start = end + 5;
        let body = &after_first[body_start..];
        let fm = parse_frontmatter_lines(fm_block);
        return (Some(fm), body);
    }
    (None, content)
}

#[derive(Debug, Clone, Default)]
struct Frontmatter {
    agent: Option<String>,
    model: Option<String>,
    compact: Option<String>,
    on_fail: Option<String>,
}

fn parse_frontmatter_lines(block: &str) -> Frontmatter {
    let mut fm = Frontmatter::default();
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let val = value.trim().trim_matches('"').trim_matches('\'').to_string();
            if val.is_empty() {
                continue;
            }
            match key.trim() {
                "agent" => fm.agent = Some(val),
                "model" => fm.model = Some(val),
                "compact" => fm.compact = Some(val),
                "on_fail" => fm.on_fail = Some(val),
                _ => {}
            }
        }
    }
    fm
}

fn parse_ash_shebang(content: &str) -> (Option<String>, Option<String>) {
    let first_line = content.lines().next().unwrap_or("");
    match crate::lexer::parse_shebang(first_line) {
        Ok(sh) => {
            let agent = Some(sh.engine);
            let model = if sh.model.is_empty() { None } else { Some(sh.model) };
            (agent, model)
        }
        Err(_) => (None, None),
    }
}

fn read_task(file_path: &Path, content: &str) -> Option<Task> {
    let name = file_path.file_name()?.to_string_lossy();
    let kind = if name.ends_with(".ash") {
        TaskKind::Ash
    } else {
        TaskKind::Markdown
    };

    if kind == TaskKind::Ash {
        if content.trim().is_empty() {
            return None;
        }
        let (agent, model) = parse_ash_shebang(content);
        return Some(Task {
            path: file_path.to_path_buf(),
            kind,
            content: content.to_string(),
            prompt: String::new(),
            agent,
            model,
            compact: None,
            on_fail: "stop".to_string(),
        });
    }

    let (fm_opt, body) = parse_frontmatter(content);
    let prompt = body.trim().to_string();
    if prompt.is_empty() {
        return None;
    }
    let fm = fm_opt.unwrap_or_default();
    Some(Task {
        path: file_path.to_path_buf(),
        kind,
        content: content.to_string(),
        prompt,
        agent: fm.agent,
        model: fm.model,
        compact: fm.compact,
        on_fail: fm.on_fail.unwrap_or_else(|| "stop".to_string()),
    })
}

#[derive(Debug, Clone)]
enum Entry {
    File(PathBuf),
    Dir(PathBuf),
}

fn walk_dir(dir: &Path) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();
    walk_dir_into(dir, &mut tasks, dir)?;
    Ok(tasks)
}

fn walk_dir_into(dir: &Path, tasks: &mut Vec<Task>, root: &Path) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut sorted: Vec<Entry> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    let mut prefix_map: std::collections::HashMap<u64, Vec<String>> = std::collections::HashMap::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if is_hidden(&name_str) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            if let Some(prefix) = extract_numeric_prefix(&name_str) {
                prefix_map.entry(prefix).or_default().push(format!("{}/", name_str));
            }
            sorted.push(Entry::Dir(path));
        } else if path.is_file() {
            if let Some(ext) = file_ext(&name_str) {
                if !is_task_file(&name_str) {
                    skipped.push(("no-prefix".to_string(), name_str));
                } else {
                    if let Some(prefix) = extract_numeric_prefix(
                        name_str.strip_suffix(ext).unwrap_or(&name_str),
                    ) {
                        prefix_map.entry(prefix).or_default().push(name_str.clone());
                    }
                    sorted.push(Entry::File(path));
                }
            } else {
                skipped.push(("non-task".to_string(), name_str));
            }
        }
    }

    let conflicts: Vec<_> = prefix_map
        .into_iter()
        .filter(|(_, names)| names.len() > 1)
        .collect();
    if !conflicts.is_empty() {
        let rel = dir.strip_prefix(root).unwrap_or(dir);
        let dir_label = if rel.as_os_str().is_empty() {
            String::new()
        } else {
            format!("{}/", rel.display())
        };
        let mut msg = format!(
            "numbering conflict in {}\n",
            if dir_label.is_empty() {
                "root directory".to_string()
            } else {
                dir_label.trim_end_matches('/').to_string()
            }
        );
        for (prefix, names) in &conflicts {
            msg.push_str(&format!(
                "  prefix {}: {}\n",
                prefix,
                names.join(", ")
            ));
        }
        msg.push_str("suggestion: ensure every file and directory at the same level has a unique numeric prefix");
        return Err(msg);
    }

    for (reason, name) in &skipped {
        let rel = dir
            .strip_prefix(root)
            .unwrap_or(dir);
        if rel.as_os_str().is_empty() {
            println!("[skip] {}: {}", reason, name);
        } else {
            println!("[skip] {}: {}/{}", reason, rel.display(), name);
        }
    }

    sorted.sort_by_key(|e| {
        let name = match e {
            Entry::File(p) | Entry::Dir(p) => {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            }
        };
        extract_numeric_prefix(&name).unwrap_or(u64::MAX)
    });

    for entry in sorted {
        match entry {
            Entry::Dir(d) => {
                walk_dir_into(&d, tasks, root)?;
            }
            Entry::File(f) => {
                let content = match fs::read_to_string(&f) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Some(task) = read_task(&f, &content) {
                    tasks.push(task);
                } else {
                    let rel = f
                        .strip_prefix(root)
                        .unwrap_or(&f);
                    if rel.as_os_str().is_empty() {
                        println!("[skip] empty: {}", f.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());
                    } else {
                        println!("[skip] empty: {}", rel.display());
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn run_tree(config: WalkConfig, eval: &mut Evaluator) -> i32 {
    let tasks = match walk_dir(&config.root) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    let total = tasks.len();

    if total == 0 {
        eprintln!("No tasks found in {}", config.root.display());
        return 0;
    }

    if config.dry_run {
        for (i, task) in tasks.iter().enumerate() {
            let agent = task.agent.as_deref().unwrap_or(&config.default_agent);
            let model = task.model.as_deref().unwrap_or(&config.default_model);
            let model_display = if model.is_empty() { "default" } else { model };
            match task.kind {
                TaskKind::Markdown => {
                    let preview: String = task.prompt.chars().take(80).collect();
                    let preview = if task.prompt.len() > 80 {
                        format!("{}...", preview)
                    } else {
                        preview
                    };
                    println!(
                        "[{}/{}] {} | type=md agent={} model={} on_fail={}",
                        i + 1,
                        total,
                        task.path.display(),
                        agent,
                        model_display,
                        task.on_fail,
                    );
                    println!("       {}", preview);
                }
                TaskKind::Ash => {
                    let preview: String = task.content.chars().take(80).collect();
                    let preview = if task.content.len() > 80 {
                        format!("{}...", preview)
                    } else {
                        preview
                    };
                    println!(
                        "[{}/{}] {} | type=ash agent={} model={}",
                        i + 1,
                        total,
                        task.path.display(),
                        agent,
                        model_display,
                    );
                    println!("       {}", preview);
                }
            }
        }
        println!("{} tasks (dry-run)", total);
        return 0;
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, task) in tasks.iter().enumerate() {
        let rel_path = task
            .path
            .strip_prefix(&config.root)
            .unwrap_or(&task.path)
            .display();
        eprintln!("[{}/{}] {}", i + 1, total, rel_path);

        match task.kind {
            TaskKind::Markdown => {
                let agent_name = task.agent.as_deref().unwrap_or(&config.default_agent);
                let agent = engine::get(agent_name);

                let model = task.model.as_deref().unwrap_or(&config.default_model);

                let prompt = interpolate_prompt(&task.prompt, eval);

                let req = ExecuteRequest {
                    prompt,
                    model: model.to_string(),
                    dir: String::new(),
                    session: false,
                };

                let resp = if let Some(eng) = agent {
                    eng.execute(&req)
                } else {
                    ExecuteResponse {
                        stdout: String::new(),
                        stderr: format!("agent '{}' not found in registry", agent_name),
                        exit_code: -1,
                    }
                };

                if resp.exit_code == 0 {
                    eprintln!("[ok]");
                    passed += 1;
                } else {
                    eprintln!(
                        "[fail] exit={} {}",
                        resp.exit_code,
                        resp.stderr.trim()
                    );
                    failed += 1;
                    if task.on_fail == "stop" && !config.continue_on_error {
                        eprintln!(
                            "stopping after failure ({} passed, {} failed)",
                            passed, failed
                        );
                        return 1;
                    }
                }
            }
            TaskKind::Ash => {
                let script = match parse_str(&task.content) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[fail] parse error: {}", e);
                        failed += 1;
                        if task.on_fail == "stop" && !config.continue_on_error {
                            eprintln!(
                                "stopping after failure ({} passed, {} failed)",
                                passed, failed
                            );
                            return 1;
                        }
                        continue;
                    }
                };

                if let Some(ref shebang) = script.shebang {
                    eval.set_default_agent(&shebang.engine);
                    if !shebang.model.is_empty() {
                        eval.set_default_model(&shebang.model);
                    }
                }

                let result = eval.eval_script(&script);

                match result {
                    Ok(()) => {
                        eprintln!("[ok]");
                        passed += 1;
                    }
                    Err(EvalError::Exit(ex)) => {
                        let exit_code = ex.code;
                        if exit_code == 0 {
                            eprintln!("[ok]");
                            passed += 1;
                        } else {
                            eprintln!("[fail] exit={}", exit_code);
                            failed += 1;
                            if task.on_fail == "stop" && !config.continue_on_error {
                                eprintln!(
                                    "stopping after failure ({} passed, {} failed)",
                                    passed, failed
                                );
                                return 1;
                            }
                        }
                    }
                    Err(EvalError::Msg(e)) => {
                        eprintln!("[fail] {}", e);
                        failed += 1;
                        if task.on_fail == "stop" && !config.continue_on_error {
                            eprintln!(
                                "stopping after failure ({} passed, {} failed)",
                                passed, failed
                            );
                            return 1;
                        }
                    }
                }
            }
        }
    }

    eprintln!("{} tasks, {} passed, {} failed", total, passed, failed);
    if failed > 0 {
        1
    } else {
        0
    }
}

fn interpolate_prompt(prompt: &str, eval: &Evaluator) -> String {
    let scope = eval.current_scope.clone();
    Interpolation::resolve(
        prompt,
        move |name| {
            scope
                .lock()
                .unwrap()
                .get(name)
                .map(|v| format!("{}", v))
        },
        move |cmd| crate::executor::Executor::new().run(cmd).map(|r| r.stdout),
    )
    .unwrap_or_else(|_| prompt.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestDir {
        dir: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir()
                .join(format!("ash-tree-test-{}-{}", std::process::id(), id));
            fs::create_dir_all(&dir).unwrap();
            TestDir { dir }
        }

        fn file(&self, name: &str, content: &str) {
            fs::write(self.dir.join(name), content).unwrap();
        }

        fn subdir(&self, name: &str) -> PathBuf {
            let d = self.dir.join(name);
            fs::create_dir_all(&d).unwrap();
            d
        }

        fn path(&self) -> &Path {
            &self.dir
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn test_extract_numeric_prefix_basic() {
        assert_eq!(extract_numeric_prefix("01-token"), Some(1));
        assert_eq!(extract_numeric_prefix("02-value"), Some(2));
        assert_eq!(extract_numeric_prefix("123-abc"), Some(123));
        assert_eq!(extract_numeric_prefix("0-prefix"), Some(0));
    }

    #[test]
    fn test_extract_numeric_prefix_no_digits() {
        assert_eq!(extract_numeric_prefix("readme"), None);
        assert_eq!(extract_numeric_prefix("abc-01"), None);
        assert_eq!(extract_numeric_prefix(""), None);
    }

    #[test]
    fn test_is_task_file() {
        assert!(is_task_file("01-task.md"));
        assert!(is_task_file("123-task.md"));
        assert!(!is_task_file("task.md"));
        assert!(!is_task_file("01-task.txt"));
        assert!(!is_task_file("01-task.md.bak"));
    }

    #[test]
    fn test_is_task_file_ash() {
        assert!(is_task_file("01-task.ash"));
        assert!(is_task_file("42-build.ash"));
        assert!(!is_task_file("task.ash"));
        assert!(!is_task_file("01-task.ash.bak"));
    }

    #[test]
    fn test_file_ext() {
        assert_eq!(file_ext("01-task.md"), Some(".md"));
        assert_eq!(file_ext("01-task.ash"), Some(".ash"));
        assert_eq!(file_ext("01-task.txt"), None);
        assert_eq!(file_ext("readme"), None);
    }

    #[test]
    fn test_parse_ash_shebang() {
        let (engine, model) = parse_ash_shebang("#!opencode:1.0");
        assert_eq!(engine.as_deref(), Some("opencode"));
        assert!(model.is_none());

        let (engine, model) = parse_ash_shebang("#!opencode:1.2.0:sonnet");
        assert_eq!(engine.as_deref(), Some("opencode"));
        assert_eq!(model.as_deref(), Some("sonnet"));

        let (engine, model) = parse_ash_shebang("#!claude-code:2.0.0:claude-sonnet-4");
        assert_eq!(engine.as_deref(), Some("claude-code"));
        assert_eq!(model.as_deref(), Some("claude-sonnet-4"));

        let (engine, model) = parse_ash_shebang("X = 42\nprint X\n");
        assert!(engine.is_none());
        assert!(model.is_none());

        let (engine, model) = parse_ash_shebang("not a shebang");
        assert!(engine.is_none());
        assert!(model.is_none());
    }

    #[test]
    fn test_read_task_ash_basic() {
        let content = "#!opencode:1.0\nX = 42\nprint X\n";
        let task = read_task(Path::new("01-setup.ash"), content).unwrap();
        assert_eq!(task.kind, TaskKind::Ash);
        assert_eq!(task.content, content);
        assert_eq!(task.agent.as_deref(), Some("opencode"));
        assert!(task.model.is_none());
        assert_eq!(task.prompt, "");
        assert_eq!(task.on_fail, "stop");
    }

    #[test]
    fn test_read_task_ash_empty() {
        assert!(read_task(Path::new("01-empty.ash"), "").is_none());
        assert!(read_task(Path::new("01-empty.ash"), "   \n").is_none());
    }

    #[test]
    fn test_read_task_md_has_correct_kind() {
        let content = "---\nagent: opencode\n---\n\nImplement the feature";
        let task = read_task(Path::new("01-feature.md"), content).unwrap();
        assert_eq!(task.kind, TaskKind::Markdown);
        assert_eq!(task.prompt, "Implement the feature");
    }

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(".git"));
        assert!(is_hidden(".hidden"));
        assert!(!is_hidden("visible"));
        assert!(!is_hidden("01-task.md"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Title\n\nSome content";
        let (fm, body) = parse_frontmatter(content);
        assert!(fm.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_with_fields() {
        let content = "---\nagent: opencode\nmodel: sonnet\ncompact: on\non_fail: continue\n---\n\n# Title\n\nSome content";
        let (fm_opt, body) = parse_frontmatter(content);
        let fm = fm_opt.unwrap();
        assert_eq!(fm.agent.as_deref(), Some("opencode"));
        assert_eq!(fm.model.as_deref(), Some("sonnet"));
        assert_eq!(fm.compact.as_deref(), Some("on"));
        assert_eq!(fm.on_fail.as_deref(), Some("continue"));
        assert_eq!(body.trim(), "# Title\n\nSome content");
    }

    #[test]
    fn test_parse_frontmatter_empty() {
        let content = "---\n---\n\n# Content";
        let (fm_opt, body) = parse_frontmatter(content);
        assert!(fm_opt.is_some());
        assert_eq!(body.trim(), "# Content");
    }

    #[test]
    fn test_parse_frontmatter_partial_fields() {
        let content = "---\nagent: claude-code\n---\n\n# Content";
        let (fm_opt, body) = parse_frontmatter(content);
        let fm = fm_opt.unwrap();
        assert_eq!(fm.agent.as_deref(), Some("claude-code"));
        assert!(fm.model.is_none());
        assert!(fm.compact.is_none());
        assert!(fm.on_fail.is_none());
        assert_eq!(body.trim(), "# Content");
    }

    #[test]
    fn test_read_task_full() {
        let content = "---\nagent: opencode\nmodel: sonnet\non_fail: continue\n---\n\nImplement the feature";
        let task = read_task(Path::new("01-feature.md"), content).unwrap();
        assert_eq!(task.agent.as_deref(), Some("opencode"));
        assert_eq!(task.model.as_deref(), Some("sonnet"));
        assert_eq!(task.on_fail, "continue");
        assert_eq!(task.prompt, "Implement the feature");
    }

    #[test]
    fn test_read_task_no_frontmatter() {
        let content = "# Task\nJust a task";
        let task = read_task(Path::new("01-task.md"), content).unwrap();
        assert!(task.agent.is_none());
        assert!(task.model.is_none());
        assert_eq!(task.on_fail, "stop");
        assert_eq!(task.prompt, "# Task\nJust a task");
    }

    #[test]
    fn test_read_task_empty_returns_none() {
        let content = "---\n---\n\n   ";
        assert!(read_task(Path::new("01-empty.md"), content).is_none());
        assert!(read_task(Path::new("01-empty.md"), "").is_none());
    }

    #[test]
    fn test_walk_dir_ordering() {
        let d = TestDir::new();
        fs::write(d.subdir("02-sub").join("01-a.md"), "# A").unwrap();
        fs::write(d.subdir("02-sub").join("02-b.md"), "# B").unwrap();
        d.file("01-root.md", "# Root");
        fs::write(d.subdir("03-last").join("01-c.md"), "# C").unwrap();

        let tasks = walk_dir(d.path()).unwrap();
        let names: Vec<&str> = tasks.iter().map(|t| {
            t.path.file_name().unwrap().to_str().unwrap()
        }).collect();
        assert_eq!(names, vec!["01-root.md", "01-a.md", "02-b.md", "01-c.md"]);
    }

    #[test]
    fn test_walk_dir_skips_non_md() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("02-other.txt", "not a markdown");

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_skips_no_prefix() {
        let d = TestDir::new();
        d.file("01-valid.md", "# Valid");
        d.file("readme.md", "# Readme");
        d.file("02-also.md", "# Also");

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-valid.md");
        assert_eq!(tasks[1].path.file_name().unwrap().to_str().unwrap(), "02-also.md");
    }

    #[test]
    fn test_walk_dir_skips_hidden() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        let hidden_dir = d.subdir(".hidden");
        fs::write(hidden_dir.join("01-secret.md"), "# Secret").unwrap();

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_skips_empty() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("02-empty.md", "---\n---\n\n  \n");

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_conflict_detection() {
        let d = TestDir::new();
        d.file("01-foo.md", "# Foo");
        d.file("01-bar.md", "# Bar");

        let result = walk_dir(d.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("numbering conflict"));
        assert!(err.contains("prefix 1"));
        assert!(err.contains("01-bar.md"));
        assert!(err.contains("01-foo.md"));
    }

    #[test]
    fn test_walk_dir_conflict_file_and_dir() {
        let d = TestDir::new();
        d.file("01-foo.md", "# Foo");
        d.subdir("01-sub");

        let result = walk_dir(d.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("numbering conflict"));
        assert!(err.contains("01-sub/"));
        assert!(err.contains("01-foo.md"));
    }

    #[test]
    fn test_walk_dir_no_conflict_different_levels() {
        let d = TestDir::new();
        d.file("01-root.md", "# Root");
        let sub = d.subdir("02-sub");
        fs::write(sub.join("01-inner.md"), "# Inner").unwrap();

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_walk_dir_picks_up_ash_files() {
        let d = TestDir::new();
        d.file("01-setup.ash", "X = 42\n");
        d.file("02-build.md", "# Build");
        d.file("03-test.ash", "print X\n");

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].kind, TaskKind::Ash);
        assert_eq!(tasks[1].kind, TaskKind::Markdown);
        assert_eq!(tasks[2].kind, TaskKind::Ash);
        assert_eq!(
            tasks[0].path.file_name().unwrap().to_str().unwrap(),
            "01-setup.ash"
        );
        assert_eq!(
            tasks[1].path.file_name().unwrap().to_str().unwrap(),
            "02-build.md"
        );
        assert_eq!(
            tasks[2].path.file_name().unwrap().to_str().unwrap(),
            "03-test.ash"
        );
    }

    #[test]
    fn test_walk_dir_skips_ash_no_prefix() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("readme.ash", "X = 42\n");

        let tasks = walk_dir(d.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_mixed_sorting() {
        let d = TestDir::new();
        d.file("01-first.ash", "#!echo:1.0\n");
        d.file("02-second.md", "# Second");
        d.file("03-third.ash", "#!echo:1.0\n");

        let tasks = walk_dir(d.path()).unwrap();
        let names: Vec<&str> = tasks
            .iter()
            .map(|t| t.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["01-first.ash", "02-second.md", "03-third.ash"]);
    }

    #[test]
    fn test_walk_dir_conflict_md_and_ash_same_prefix() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("01-task.ash", "X = 42\n");

        let result = walk_dir(d.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("numbering conflict"));
        assert!(err.contains("prefix 1"));
        assert!(err.contains("01-task.md"));
        assert!(err.contains("01-task.ash"));
    }
}
