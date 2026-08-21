pub mod ledger;
pub mod policy;
pub mod proxy;
pub mod scan;

use crate::config::registry::GuardSettings;
use crate::launcher::env_schema::EnvMap;

/// Spawn the guard proxy on a daemon thread owning its own tokio runtime,
/// bound to an ephemeral localhost port. Returns the assigned port after a
/// successful bind, so proxy failures surface before claude launches.
///
/// The thread lives for the process lifetime — main exits via
/// `std::process::exit` after the claude session, killing it.
pub fn start_guard_proxy(
    upstream: &str,
    settings: GuardSettings,
    provider_id: &str,
    ledger_file: &str,
) -> anyhow::Result<u16> {
    if let Some(parent) = std::path::Path::new(ledger_file).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (tx, rx) = std::sync::mpsc::channel::<Option<u16>>();
    let upstream = upstream.to_string();
    let provider_id = provider_id.to_string();
    let ledger_file = ledger_file.to_string();
    std::thread::Builder::new()
        .name("claudy-guard".to_string())
        .spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(_) => {
                    let _ = tx.send(None);
                    return;
                }
            };
            rt.block_on(async move {
                let app = match proxy::build_router(upstream, settings, provider_id, ledger_file) {
                    Ok(app) => app,
                    Err(_) => {
                        let _ = tx.send(None);
                        return;
                    }
                };
                let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
                    Ok(l) => l,
                    Err(_) => {
                        let _ = tx.send(None);
                        return;
                    }
                };
                let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
                let _ = tx.send(Some(port));
                let _ = axum::serve(listener, app).await;
            });
        })?;
    match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Some(port)) if port != 0 => Ok(port),
        _ => anyhow::bail!("failed to start guard proxy on 127.0.0.1"),
    }
}

/// Rewrite (or set, for the empty-base-url native provider) the base URL to
/// point at the local guard proxy.
pub fn apply_guard_env(env: &[String], port: u16) -> Vec<String> {
    let mut map = EnvMap::from_env_slice_lenient(env);
    map.set("ANTHROPIC_BASE_URL", &format!("http://127.0.0.1:{port}"));
    map.to_env_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_guard_env_sets_missing_base_url() {
        // Native provider: envkit's set_if_not_empty leaves the var absent.
        let env = vec!["ANTHROPIC_MODEL=claude-x".to_string()];
        let out = apply_guard_env(&env, 45678);
        assert!(out.contains(&"ANTHROPIC_BASE_URL=http://127.0.0.1:45678".to_string()));
    }

    #[test]
    fn apply_guard_env_replaces_existing_base_url() {
        let env = vec![
            "ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic".to_string(),
            "ANTHROPIC_AUTH_TOKEN=redacted".to_string(),
        ];
        let out = apply_guard_env(&env, 45678);
        assert!(out.contains(&"ANTHROPIC_BASE_URL=http://127.0.0.1:45678".to_string()));
        assert!(
            !out.iter()
                .any(|e| e.starts_with("ANTHROPIC_BASE_URL=https://api.z.ai"))
        );
        // Other vars survive.
        assert!(out.contains(&"ANTHROPIC_AUTH_TOKEN=redacted".to_string()));
    }
}
