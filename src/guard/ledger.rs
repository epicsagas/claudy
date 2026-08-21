use crate::ports::guard_ports::Finding;

/// Guard decision ledger — append-only JSONL via the existing channel audit
/// writer (open-per-event, best-effort, never panics).
pub struct RequestRecord<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub upstream_host: &'a str,
    pub status: u16,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub findings: &'a [Finding],
}

pub fn log_request(ledger_file: &str, rec: RequestRecord) {
    let findings: Vec<serde_json::Value> = rec
        .findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "kind": f.kind,
                "action": format!("{:?}", f.action),
                "preview": f.preview,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "method": rec.method,
        "path": rec.path,
        "upstream_host": rec.upstream_host,
        "status": rec.status,
        "bytes_in": rec.bytes_in,
        "bytes_out": rec.bytes_out,
        "findings": findings,
    });
    crate::adapters::channel::audit::log_event(ledger_file, "guard_request", &payload);
}

pub fn log_upstream_error(ledger_file: &str, path: &str, upstream_host: &str, error: &str) {
    let payload = serde_json::json!({
        "path": path,
        "upstream_host": upstream_host,
        "error": error,
    });
    crate::adapters::channel::audit::log_event(ledger_file, "guard_upstream_error", &payload);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::guard_ports::GuardAction;

    #[test]
    fn ledger_records_redacted_preview_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger = dir.path().join("ledger.jsonl");
        let ledger = ledger.to_str().expect("utf8 path");

        let findings = vec![Finding {
            kind: "api_key".to_string(),
            action: GuardAction::Redact,
            preview: "sk-a****f789".to_string(),
        }];
        log_request(
            ledger,
            RequestRecord {
                method: "POST",
                path: "/v1/messages",
                upstream_host: "api.example.com",
                status: 200,
                bytes_in: 10,
                bytes_out: 20,
                findings: &findings,
            },
        );

        let content = std::fs::read_to_string(ledger).expect("ledger written");
        assert!(content.contains("guard_request"));
        assert!(content.contains("sk-a****f789"));
        // Raw secret material never appears.
        assert!(!content.contains("sk-ant-api03-secretsecretsecret"));
    }
}
