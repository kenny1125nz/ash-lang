use std::fmt;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use log::{debug, info, warn};

use crate::engine::{self, ExecuteRequest, ExecuteResponse};
use crate::lang::ast::Script;
use crate::lang::parser::parse_str;
use crate::runtime::interpolation::Interpolation;
use crate::runtime::scope::ScopeRef;
use crate::AshError;

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

/// A group of tasks sharing the same numeric prefix at the same directory level.
/// Single-file groups execute sequentially; multi-file groups execute in parallel.
/// If files share the prefix with a subdirectory, the subdirectory is walked
/// and its groups run on a separate thread alongside the files.
#[derive(Debug, Clone)]
pub struct TaskGroup {
    pub files: Vec<Task>,
    pub subdir: Option<Vec<TaskGroup>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParallelMode {
    /// Run parallel groups without prompting
    Allow,
    /// Prompt for confirmation if interactive; error if non-interactive
    Prompt,
}

#[derive(Debug, Clone)]
pub struct WalkConfig {
    pub root: PathBuf,
    pub dry_run: bool,
    pub continue_on_error: bool,
    pub default_agent: String,
    pub default_model: String,
    pub parallel: ParallelMode,
}

impl Default for WalkConfig {
    fn default() -> Self {
        WalkConfig {
            root: PathBuf::new(),
            dry_run: false,
            continue_on_error: false,
            default_agent: "echo".to_string(),
            default_model: String::new(),
            parallel: ParallelMode::Prompt,
        }
    }
}

#[derive(Debug)]
pub enum ExecError {
    Exit(i32),
    Msg(String),
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::Exit(code) => write!(f, "exit code {}", code),
            ExecError::Msg(s) => write!(f, "{}", s),
        }
    }
}

/// Trait that abstracts over the concrete Evaluator, breaking the
/// circular dependency between runtime (tree walker) and eval.
pub trait TaskExecutor: Send + Sync {
    fn fork(&self) -> Box<dyn TaskExecutor + Send>;
    fn eval_script(&mut self, script: &Script) -> Result<(), ExecError>;
    fn set_default_agent(&mut self, name: &str);
    fn set_default_model(&mut self, name: &str);
    fn set_source_path(&mut self, path: Option<PathBuf>);
    fn current_scope(&self) -> ScopeRef;
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

pub fn parse_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
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
pub struct Frontmatter {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub compact: Option<String>,
    pub on_fail: Option<String>,
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
    match crate::lang::lexer::parse_shebang(first_line) {
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

fn walk_dir(dir: &Path) -> Result<Vec<TaskGroup>, AshError> {
    let mut groups: Vec<TaskGroup> = Vec::new();
    walk_dir_into(dir, &mut groups, dir)?;
    Ok(groups)
}

fn walk_dir_into(dir: &Path, groups: &mut Vec<TaskGroup>, root: &Path) -> Result<(), AshError> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut file_entries: Vec<PathBuf> = Vec::new();
    let mut dir_entries: Vec<PathBuf> = Vec::new();
    let mut skipped: Vec<(String, String)> = Vec::new();
    let mut file_prefix_map: std::collections::HashMap<u64, Vec<PathBuf>> = std::collections::HashMap::new();
    let mut dir_prefix_set: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut prefix_names: std::collections::HashMap<u64, Vec<String>> = std::collections::HashMap::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if is_hidden(&name_str) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            if let Some(prefix) = extract_numeric_prefix(&name_str) {
                dir_prefix_set.insert(prefix);
                prefix_names.entry(prefix).or_default().push(format!("{}/", name_str));
            }
            dir_entries.push(path);
        } else if path.is_file() {
            if let Some(ext) = file_ext(&name_str) {
                if !is_task_file(&name_str) {
                    skipped.push(("no-prefix".to_string(), name_str));
                } else {
                    let stem = name_str.strip_suffix(ext).unwrap_or(&name_str);
                    if let Some(prefix) = extract_numeric_prefix(stem) {
                        file_prefix_map.entry(prefix).or_default().push(path.clone());
                        prefix_names.entry(prefix).or_default().push(name_str.clone());
                    }
                    file_entries.push(path);
                }
            } else {
                skipped.push(("non-task".to_string(), name_str));
            }
        }
    }

    // Track which prefixes have mixed files+dirs (these become combined parallel groups)
    let mixed_prefixes: std::collections::HashSet<u64> = file_prefix_map
        .keys()
        .copied()
        .filter(|p| dir_prefix_set.contains(p))
        .collect();

    for (reason, name) in &skipped {
        let rel = dir.strip_prefix(root).unwrap_or(dir);
        if rel.as_os_str().is_empty() {
            println!("[skip] {}: {}", reason, name);
        } else {
            println!("[skip] {}: {}/{}", reason, rel.display(), name);
        }
    }

    // Sort file entries by prefix
    file_entries.sort_by_key(|p| {
        let name = p.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        extract_numeric_prefix(&name).unwrap_or(u64::MAX)
    });

    // Sort directory entries by prefix
    dir_entries.sort_by_key(|p| {
        let name = p.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        extract_numeric_prefix(&name).unwrap_or(u64::MAX)
    });

    // Merge file groups and directory groups in prefix order
    let mut all_keys: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for p in file_prefix_map.keys() { all_keys.insert(*p); }
    for p in &dir_prefix_set { all_keys.insert(*p); }

    for prefix in all_keys {
        let is_mixed = mixed_prefixes.contains(&prefix);

        // Process file groups for this prefix
        if let Some(files) = file_prefix_map.remove(&prefix) {
            let mut group_tasks: Vec<Task> = Vec::new();
            for f in files {
                debug!("tree — reading file {}", f.display());
                let content = match fs::read_to_string(&f) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Some(task) = read_task(&f, &content) {
                    debug!("tree — loaded task {}", f.display());
                    group_tasks.push(task);
                } else {
                    let rel = f.strip_prefix(root).unwrap_or(&f);
                    if rel.as_os_str().is_empty() {
                        println!("[skip] empty: {}", f.file_name().map(|n| n.to_string_lossy()).unwrap_or_default());
                    } else {
                        println!("[skip] empty: {}", rel.display());
                    }
                }
            }
            if !group_tasks.is_empty() {
                group_tasks.sort_by_key(|t| {
                    t.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
                });
                if is_mixed {
                    // Walk the subdirectory and embed its groups
                    let dirs_at_prefix: Vec<PathBuf> = dir_entries
                        .iter()
                        .filter(|d| {
                            let name = d.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                            extract_numeric_prefix(&name) == Some(prefix)
                        })
                        .cloned()
                        .collect();
                    let mut subdir_groups: Vec<TaskGroup> = Vec::new();
                    for d in dirs_at_prefix {
                        walk_dir_into(&d, &mut subdir_groups, root)?;
                    }
                    groups.push(TaskGroup {
                        files: group_tasks,
                        subdir: Some(subdir_groups),
                    });
                } else {
                    groups.push(TaskGroup {
                        files: group_tasks,
                        subdir: None,
                    });
                }
            }
        }

        // Process directory-only prefixes (no file at this prefix)
        if !is_mixed && dir_prefix_set.contains(&prefix) {
            let dirs_at_prefix: Vec<PathBuf> = dir_entries
                .iter()
                .filter(|d| {
                    let name = d.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    extract_numeric_prefix(&name) == Some(prefix)
                })
                .cloned()
                .collect();
            for d in dirs_at_prefix {
                walk_dir_into(&d, groups, root)?;
            }
        }
    }

    Ok(())
}

fn confirm_parallel(group_count: usize) -> bool {
    if !std::io::stdin().is_terminal() {
        return false;
    }
    print!(
        "\n{} parallel group(s) detected. Run them in parallel? [y/N] ",
        group_count,
    );
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}

fn execute_md_task(task: &Task, config: &WalkConfig, eval: &mut dyn TaskExecutor) -> bool {
    let agent_name = task.agent.as_deref().unwrap_or(&config.default_agent);
    let agent = engine::get(agent_name);
    let model = task.model.as_deref().unwrap_or(&config.default_model);
    let prompt = interpolate_prompt(&task.prompt, eval);

    let req = ExecuteRequest {
        prompt,
        model: model.to_string(),
        dir: String::new(),
        session: false,
        yes: false,
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
        eprintln!("  [ok]");
        true
    } else {
        eprintln!("  [fail] exit={} {}", resp.exit_code, resp.stderr.trim());
        false
    }
}

fn execute_ash_task(task: &Task, _config: &WalkConfig, eval: &mut dyn TaskExecutor) -> bool {
    let script = match parse_str(&task.content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  [fail] parse error: {}", e);
            return false;
        }
    };

    if let Some(ref shebang) = script.shebang {
        eval.set_default_agent(&shebang.engine);
        if !shebang.model.is_empty() {
            eval.set_default_model(&shebang.model);
        }
    }

    eval.set_source_path(Some(task.path.clone()));

    let result = match eval.eval_script(&script) {
        Ok(()) => {
            eprintln!("  [ok]");
            true
        }
        Err(ExecError::Exit(0)) => {
            eprintln!("  [ok]");
            true
        }
        Err(ExecError::Exit(code)) => {
            eprintln!("  [fail] exit={}", code);
            false
        }
        Err(ExecError::Msg(e)) => {
            eprintln!("  [fail] {}", e);
            false
        }
    };

    eval.set_source_path(None);

    result
}

fn execute_task(task: &Task, config: &WalkConfig, eval: &mut dyn TaskExecutor) -> bool {
    let rel_path = task.path.strip_prefix(&config.root).unwrap_or(&task.path);
    debug!("tree — executing {} (kind={:?})", rel_path.display(), task.kind);
    match task.kind {
        TaskKind::Markdown => execute_md_task(task, config, eval),
        TaskKind::Ash => execute_ash_task(task, config, eval),
    }
}

fn execute_task_isolated(task: &Task, config: &WalkConfig, parent_eval: &dyn TaskExecutor) -> bool {
    let mut eval = parent_eval.fork();
    execute_task(task, config, &mut *eval)
}

fn dry_run_group(group: &TaskGroup, index: usize, total: usize, config: &WalkConfig) {
    let file_count = group.files.len();
    let has_subdir = group.subdir.is_some();
    let label = if file_count > 1 && !has_subdir {
        format!("[parallel: {} tasks]", file_count)
    } else if has_subdir && file_count > 0 {
        format!("[parallel: {} files + subdir]", file_count)
    } else if has_subdir {
        String::new()
    } else {
        String::new()
    };

    for (j, task) in group.files.iter().enumerate() {
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
                if j == 0 && !label.is_empty() {
                    println!("[{}/{}] {} | type=md agent={} model={} on_fail={} {}",
                        index + 1, total, task.path.display(), agent, model_display, task.on_fail, label);
                } else {
                    println!("[{}/{}] {} | type=md agent={} model={} on_fail={}",
                        index + 1, total, task.path.display(), agent, model_display, task.on_fail);
                }
                println!("       {}", preview);
            }
            TaskKind::Ash => {
                let preview: String = task.content.chars().take(80).collect();
                let preview = if task.content.len() > 80 {
                    format!("{}...", preview)
                } else {
                    preview
                };
                if j == 0 && !label.is_empty() {
                    println!("[{}/{}] {} | type=ash agent={} model={} {}",
                        index + 1, total, task.path.display(), agent, model_display, label);
                } else {
                    println!("[{}/{}] {} | type=ash agent={} model={}",
                        index + 1, total, task.path.display(), agent, model_display);
                }
                println!("       {}", preview);
            }
        }
    }

    if has_subdir {
        let subdir_groups = group.subdir.as_ref().unwrap();
        println!("       [subdirectory with {} group(s)]", subdir_groups.len());
    }
}

fn count_tasks_in_groups(groups: &[TaskGroup]) -> usize {
    let mut count = 0;
    for g in groups {
        count += g.files.len();
        if let Some(ref sub) = g.subdir {
            count += count_tasks_in_groups(sub);
        }
    }
    count
}

fn has_parallel_work(group: &TaskGroup) -> bool {
    group.files.len() > 1 || group.subdir.is_some()
}

fn execute_group_parallel<'a>(
    group: &'a TaskGroup,
    config: &WalkConfig,
    parent_eval: &dyn TaskExecutor,
) -> Vec<(bool, Option<&'a Task>)> {
    std::thread::scope(|s| {
        let mut handles: Vec<std::thread::ScopedJoinHandle<'_, (bool, Option<&'a Task>)>> = Vec::new();

        for task in &group.files {
            let handle = s.spawn(move || {
                let ok = execute_task_isolated(task, config, parent_eval);
                (ok, Some(task))
            });
            handles.push(handle);
        }

        if let Some(ref subdir_groups) = group.subdir {
            let handle = s.spawn(move || {
                let mut sub_results = Vec::new();
                execute_groups_isolated(subdir_groups, config, parent_eval, &mut sub_results);
                (sub_results.iter().all(|(ok, _)| *ok), None)
            });
            handles.push(handle);
        }

        handles.into_iter().map(|h| h.join().unwrap_or((false, None))).collect()
    })
}

fn execute_groups_isolated<'a>(
    groups: &'a [TaskGroup],
    config: &WalkConfig,
    parent_eval: &dyn TaskExecutor,
    result: &mut Vec<(bool, Option<&'a Task>)>,
) {
    let mut eval = parent_eval.fork();

    for group in groups {
        if group.files.is_empty() && group.subdir.is_none() {
            continue;
        }

        if group.files.len() == 1 && group.subdir.is_none() {
            let ok = execute_task(&group.files[0], config, &mut *eval);
            result.push((ok, Some(&group.files[0])));
        } else if has_parallel_work(group) {
            let results = execute_group_parallel(group, config, parent_eval);
            for (ok, maybe_task) in results {
                result.push((ok, maybe_task));
            }
        }
    }
}

pub fn run_tree(config: WalkConfig, eval: &mut dyn TaskExecutor) -> i32 {
    info!("engine — walking task tree at {}", config.root.display());

    let groups = match walk_dir(&config.root) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };

    if groups.is_empty() {
        warn!("No tasks found in {}", config.root.display());
        return 0;
    }

    let total_groups = groups.len();
    let total_tasks = count_tasks_in_groups(&groups);
    info!("engine — dispatching {} groups ({} tasks)", total_groups, total_tasks);

    if config.dry_run {
        for (i, group) in groups.iter().enumerate() {
            dry_run_group(group, i, total_groups, &config);
        }
        println!("{} tasks (dry-run)", total_tasks);
        return 0;
    }

    // Check if there are parallel groups and handle confirmation
    let parallel_groups: Vec<&TaskGroup> = groups.iter().filter(|g| has_parallel_work(g)).collect();
    let parallel_ok = if parallel_groups.is_empty() {
        true
    } else if config.parallel == ParallelMode::Allow {
        true
    } else if confirm_parallel(parallel_groups.len()) {
        true
    } else {
        eprintln!(
            "error: {} parallel group(s) detected. Use --yes to allow parallel execution.",
            parallel_groups.len()
        );
        return 1;
    };

    if !parallel_ok {
        return 1;
    }

    let mut passed = 0;
    let mut failed = 0;

    for (i, group) in groups.iter().enumerate() {
        let file_count = group.files.len();
        let has_subdir = group.subdir.is_some();
        info!("engine — dispatching group {}/{} ({} files, subdir={})", i + 1, total_groups, file_count, has_subdir);

        if file_count == 1 && !has_subdir {
            // Sequential execution
            let task = &group.files[0];
            let rel_path = task.path.strip_prefix(&config.root).unwrap_or(&task.path);
            eprintln!("[{}] {}", i + 1, rel_path.display());
            let ok = execute_task(task, &config, eval);
            if ok {
                passed += 1;
            } else {
                failed += 1;
                if task.on_fail == "stop" && !config.continue_on_error {
                    eprintln!("stopping after failure ({} passed, {} failed)", passed, failed);
                    return 1;
                }
            }
        } else if has_parallel_work(group) {
            // Parallel execution
            eprintln!("[{}] parallel: {} files{}",
                i + 1, file_count,
                if has_subdir { " + subdir" } else { "" });
            for task in &group.files {
                let rel_path = task.path.strip_prefix(&config.root).unwrap_or(&task.path);
                eprintln!("  ├─ {}", rel_path.display());
            }
            if has_subdir {
                eprintln!("  └─ [subdirectory]");
            }

            if group.files.iter().any(|t| t.on_fail == "stop" && !config.continue_on_error) {
                warn!("parallel group has tasks with on_fail=stop — failures will stop the entire run");
            }

            let results = execute_group_parallel(group, &config, eval);

            for (ok, maybe_task) in results {
                if ok {
                    passed += 1;
                } else {
                    failed += 1;
                    if let Some(task) = maybe_task {
                        if task.on_fail == "stop" && !config.continue_on_error {
                            eprintln!("stopping after failure ({} passed, {} failed)", passed, failed);
                            return 1;
                        }
                    } else {
                        if !config.continue_on_error {
                            eprintln!("stopping after failure ({} passed, {} failed)", passed, failed);
                            return 1;
                        }
                    }
                }
            }
        }
    }

    eprintln!("{} tasks, {} passed, {} failed", total_tasks, passed, failed);
    if failed > 0 {
        1
    } else {
        0
    }
}

fn interpolate_prompt(prompt: &str, eval: &dyn TaskExecutor) -> String {
    let scope = eval.current_scope();
    Interpolation::resolve(
        prompt,
        move |name| {
            scope
                .lock()
                .unwrap()
                .get(name)
                .map(|v| format!("{}", v))
        },
        move |cmd| crate::runtime::executor::Executor::new().run(cmd).map(|r| r.stdout),
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

    fn flatten_tasks(groups: &[TaskGroup]) -> Vec<&Task> {
        let mut result = Vec::new();
        for g in groups {
            result.extend(g.files.iter());
            if let Some(ref sub) = g.subdir {
                result.extend(flatten_tasks(sub));
            }
        }
        result
    }

    fn group_sizes(groups: &[TaskGroup]) -> Vec<usize> {
        groups.iter().map(|g| {
            let sub_count = g.subdir.as_ref().map(|s| s.len()).unwrap_or(0);
            g.files.len() + sub_count
        }).collect()
    }

    #[test]
    fn test_walk_dir_ordering() {
        let d = TestDir::new();
        fs::write(d.subdir("02-sub").join("01-a.md"), "# A").unwrap();
        fs::write(d.subdir("02-sub").join("02-b.md"), "# B").unwrap();
        d.file("01-root.md", "# Root");
        fs::write(d.subdir("03-last").join("01-c.md"), "# C").unwrap();

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
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

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_skips_no_prefix() {
        let d = TestDir::new();
        d.file("01-valid.md", "# Valid");
        d.file("readme.md", "# Readme");
        d.file("02-also.md", "# Also");

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
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

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_skips_empty() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("02-empty.md", "---\n---\n\n  \n");

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_parallel_group() {
        let d = TestDir::new();
        d.file("01-a.md", "# A");
        d.file("01-b.md", "# B");
        d.file("02-c.md", "# C");

        let groups = walk_dir(d.path()).unwrap();
        assert_eq!(group_sizes(&groups), vec![2, 1]);
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-a.md");
        assert_eq!(tasks[1].path.file_name().unwrap().to_str().unwrap(), "01-b.md");
        assert_eq!(tasks[2].path.file_name().unwrap().to_str().unwrap(), "02-c.md");
    }

    #[test]
    fn test_walk_dir_parallel_three_tasks() {
        let d = TestDir::new();
        d.file("01-a.md", "# A");
        d.file("01-b.md", "# B");
        d.file("01-c.md", "# C");
        d.file("02-d.md", "# D");

        let groups = walk_dir(d.path()).unwrap();
        assert_eq!(group_sizes(&groups), vec![3, 1]);
    }

    #[test]
    fn test_walk_dir_file_and_dir_same_prefix_parallel() {
        let d = TestDir::new();
        d.file("01-foo.md", "# Foo");
        let sub = d.subdir("01-sub");
        fs::write(sub.join("01-inner.md"), "# Inner").unwrap();
        d.file("02-bar.md", "# Bar");

        let groups = walk_dir(d.path()).unwrap();
        // Group 1: 1 file + subdir (combined parallel group)
        // Group 2: 1 file (sequential)
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].files.len(), 1);
        assert!(groups[0].subdir.is_some());
        assert_eq!(groups[1].files.len(), 1);
        assert!(groups[1].subdir.is_none());

        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-foo.md");
        assert_eq!(tasks[1].path.file_name().unwrap().to_str().unwrap(), "01-inner.md");
        assert_eq!(tasks[2].path.file_name().unwrap().to_str().unwrap(), "02-bar.md");
    }

    #[test]
    fn test_walk_dir_no_conflict_different_levels() {
        let d = TestDir::new();
        d.file("01-root.md", "# Root");
        let sub = d.subdir("02-sub");
        fs::write(sub.join("01-inner.md"), "# Inner").unwrap();

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_walk_dir_picks_up_ash_files() {
        let d = TestDir::new();
        d.file("01-setup.ash", "X = 42\n");
        d.file("02-build.md", "# Build");
        d.file("03-test.ash", "print X\n");

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
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

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].path.file_name().unwrap().to_str().unwrap(), "01-task.md");
    }

    #[test]
    fn test_walk_dir_mixed_sorting() {
        let d = TestDir::new();
        d.file("01-first.ash", "#!echo:1.0\n");
        d.file("02-second.md", "# Second");
        d.file("03-third.ash", "#!echo:1.0\n");

        let groups = walk_dir(d.path()).unwrap();
        let tasks = flatten_tasks(&groups);
        let names: Vec<&str> = tasks
            .iter()
            .map(|t| t.path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["01-first.ash", "02-second.md", "03-third.ash"]);
    }

    #[test]
    fn test_walk_dir_parallel_md_and_ash() {
        let d = TestDir::new();
        d.file("01-task.md", "# Task");
        d.file("01-task.ash", "X = 42\n");
        d.file("02-other.md", "# Other");

        let groups = walk_dir(d.path()).unwrap();
        assert_eq!(group_sizes(&groups), vec![2, 1]);
        let tasks = flatten_tasks(&groups);
        assert_eq!(tasks.len(), 3);
        // Sorted by filename within group: ".ash" < ".md"
        assert_eq!(tasks[0].kind, TaskKind::Ash);
        assert_eq!(tasks[1].kind, TaskKind::Markdown);
    }
}
