use std::path::Path;

use crate::domain::channel_session::DeliveryAttempt;

pub fn append_attempt(log_file: &str, attempt: &DeliveryAttempt) -> anyhow::Result<()> {
    let path = Path::new(log_file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(attempt)? + "\n";
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}
