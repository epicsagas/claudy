pub(crate) fn extract_text(content: &serde_json::Value) -> Option<String> {
    if let Some(s) = content.as_str()
        && !s.is_empty()
    {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for block in arr {
            if block["type"].as_str() == Some("text")
                && let Some(text) = block["text"].as_str()
                && !text.is_empty()
            {
                return Some(text.to_string());
            }
        }
    }
    None
}

pub(crate) fn find_session_jsonl(
    base: &std::path::Path,
    session_id: &str,
) -> Option<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(base) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let jsonl = path.join(format!("{}.jsonl", session_id));
        if jsonl.exists() {
            return Some(jsonl);
        }
    }
    None
}
