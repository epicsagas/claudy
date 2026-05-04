use std::path::Path;

pub fn write(pid_file: &str) -> anyhow::Result<()> {
    let pid = std::process::id();
    let path = Path::new(pid_file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::config::atomic::write_atomic(pid_file, pid.to_string().as_bytes(), 0o600)?;
    Ok(())
}

pub fn remove(pid_file: &str) {
    std::fs::remove_file(pid_file).ok();
}
