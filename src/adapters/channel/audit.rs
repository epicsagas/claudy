use std::path::Path;

pub fn init_audit_log(log_file: &str) -> anyhow::Result<()> {
    let path = Path::new(log_file);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    Ok(())
}

pub fn log_event(log_file: &str, event_type: &str, payload: &serde_json::Value) {
    let entry = serde_json::json!({
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "event": event_type,
        "payload": payload,
    });
    let line = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(_) => return,
    };
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
    {
        let _ = file.write_all(format!("{}\n", line).as_bytes());
    }
}
