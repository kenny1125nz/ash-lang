use std::path::Path;

/// Load the highest offset from marker files named `<event_path>.offset.<N>`.
/// Returns 0 if no markers exist.
pub fn load_offset(event_path: &Path) -> u64 {
    let dir = event_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = event_path.file_name().unwrap_or_default().to_string_lossy();
    let prefix = format!("{}.offset.", stem);
    let mut max = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(rest) = name.strip_prefix(&prefix) {
                if let Ok(n) = rest.parse::<u64>() {
                    max = max.max(n);
                }
            }
        }
    }
    max
}

/// Save offset by creating a marker file `<event_path>.offset.<offset>`.
/// Removes any previous markers for the same event path.
pub fn save_offset(event_path: &Path, offset: u64) {
    let dir = event_path.parent().unwrap_or_else(|| Path::new("."));
    let stem = event_path.file_name().unwrap_or_default().to_string_lossy();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{}.offset.", stem)) {
                let _ = std::fs::remove_file(dir.join(&name));
            }
        }
    }
    let marker = dir.join(format!("{}.offset.{}", stem, offset));
    let _ = std::fs::File::create(&marker);
}
