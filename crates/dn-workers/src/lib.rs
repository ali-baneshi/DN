use std::path::PathBuf;

pub fn discover_python() -> Option<PathBuf> {
    for candidate in ["python3", "python"] {
        if let Ok(path) = which::which(candidate) {
            return Some(path);
        }
    }

    None
}

pub fn require_python() -> anyhow::Result<PathBuf> {
    discover_python().ok_or_else(|| anyhow::anyhow!("python interpreter not found in PATH"))
}
