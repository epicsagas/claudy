use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;

use super::ledger::RequestRecord;
use super::policy::SettingsPolicy;
use super::scan::RegexScanner;
use crate::config::registry::GuardSettings;
use crate::ports::guard_ports::{ContentScanner, Finding, GuardAction, GuardPolicy};

/// 256 MiB — generous ceiling for multi-MB context payloads with images.
const MAX_BODY: usize = 256 * 1024 * 1024;

const HOP_BY_HOP_REQ: &[&str] = &[
    "host",
    "content-length",
    "transfer-encoding",
    "connection",
    "keep-alive",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "upgrade",
];

const HOP_BY_HOP_RESP: &[&str] = &[
    "content-length",
    "transfer-encoding",
    "connection",
    "keep-alive",
    "te",
    "trailer",
    "upgrade",
];

pub(crate) struct GuardState {
    upstream: String,
    upstream_host: String,
    client: reqwest::Client,
    scanner: RegexScanner,
    policy: SettingsPolicy,
    provider_id: String,
    ledger: String,
    reroute_notified: AtomicBool,
}

pub(crate) fn build_router(
    upstream: String,
    settings: GuardSettings,
    provider_id: String,
    ledger: String,
) -> anyhow::Result<Router> {
    let client = reqwest::Client::builder()
        // A followed redirect would move egress to a destination this layer
        // never inspected — DLP must see the real target.
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(15))
        // No overall timeout: SSE streams run for minutes.
        .build()?;
    let upstream_host = reqwest::Url::parse(&upstream)
        .ok()
        .and_then(|u| u.host_str().map(str::to_string))
        .unwrap_or_else(|| upstream.clone());
    let state = Arc::new(GuardState {
        upstream,
        upstream_host,
        client,
        scanner: RegexScanner::new(&settings),
        policy: SettingsPolicy::new(settings),
        provider_id,
        ledger,
        reroute_notified: AtomicBool::new(false),
    });
    Ok(Router::new().fallback(proxy_handler).with_state(state))
}

async fn proxy_handler(State(state): State<Arc<GuardState>>, req: Request) -> Response {
    let (parts, body) = req.into_parts();
    let method = parts.method;
    let method_str = method.as_str().to_string();
    let path = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = match axum::body::to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => {
            return error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "invalid_request_error",
                "claudy-guard: request body too large".to_string(),
            );
        }
    };
    let bytes_in = body.len() as u64;

    let report = state.scanner.scan(&body, &content_type);

    if report
        .findings
        .iter()
        .any(|f| f.action == GuardAction::Block)
    {
        let kinds: Vec<String> = report.findings.iter().map(|f| f.kind.clone()).collect();
        super::ledger::log_request(
            &state.ledger,
            RequestRecord {
                method: &method_str,
                path: &path,
                upstream_host: &state.upstream_host,
                status: 400,
                bytes_in,
                bytes_out: 0,
                findings: &report.findings,
            },
        );
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            format!(
                "claudy-guard: blocked request containing {}",
                kinds.join(", ")
            ),
        );
    }

    maybe_reroute_advisory(&state, &report.findings);

    let out_body = report.redacted_body.unwrap_or_else(|| body.to_vec());

    let url = format!("{}{}", state.upstream, path);
    let mut fwd_headers = HeaderMap::new();
    for (name, value) in &parts.headers {
        if !HOP_BY_HOP_REQ.contains(&name.as_str()) {
            fwd_headers.insert(name.clone(), value.clone());
        }
    }

    let resp = match state
        .client
        .request(method, &url)
        .headers(fwd_headers)
        .body(out_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            super::ledger::log_upstream_error(
                &state.ledger,
                &path,
                &state.upstream_host,
                &e.to_string(),
            );
            return error_response(
                StatusCode::BAD_GATEWAY,
                "api_error",
                format!(
                    "claudy-guard: upstream {} unreachable: {}",
                    state.upstream_host, e
                ),
            );
        }
    };

    let status = resp.status();
    let mut out_headers = HeaderMap::new();
    for (name, value) in resp.headers() {
        if !HOP_BY_HOP_RESP.contains(&name.as_str()) {
            out_headers.insert(name.clone(), value.clone());
        }
    }

    let ctx = CountCtx {
        ledger: state.ledger.clone(),
        method: method_str,
        path,
        host: state.upstream_host.clone(),
        status: status.as_u16(),
        bytes_in,
        findings: report.findings,
    };
    let stream = CountingStream::new(resp.bytes_stream(), ctx);

    let mut builder = Response::builder().status(status);
    for (name, value) in out_headers.iter() {
        builder = builder.header(name, value);
    }
    builder
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

/// One-time stderr + ledger advisory when sensitive findings occur on an
/// untrusted provider. Mid-session provider swap is impossible, so re-route
/// stays advisory in the MVP.
fn maybe_reroute_advisory(state: &GuardState, findings: &[Finding]) {
    let sensitive = findings
        .iter()
        .any(|f| !matches!(f.kind.as_str(), "non_json" | "unparseable_json"));
    if !sensitive
        || state.reroute_notified.swap(true, Ordering::Relaxed)
        || state.policy.is_trusted(&state.provider_id)
    {
        return;
    }
    eprintln!(
        "[claudy] guard: sensitive content detected on untrusted provider '{}' — consider re-routing to a trusted provider",
        state.provider_id
    );
    let payload = serde_json::json!({"provider": state.provider_id});
    crate::adapters::channel::audit::log_event(&state.ledger, "guard_reroute_suggestion", &payload);
}

fn error_response(status: StatusCode, error_type: &str, message: String) -> Response {
    let body = serde_json::json!({
        "type": "error",
        "error": {"type": error_type, "message": message},
    });
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

struct CountCtx {
    ledger: String,
    method: String,
    path: String,
    host: String,
    status: u16,
    bytes_in: u64,
    findings: Vec<Finding>,
}

/// Passes upstream chunks through untouched while counting bytes; appends the
/// ledger entry when the stream terminates (also the only place mid-stream
/// upstream failures become visible in the ledger).
struct CountingStream {
    inner: Pin<Box<dyn futures_util::Stream<Item = reqwest::Result<Bytes>> + Send>>,
    ctx: Option<CountCtx>,
    bytes: u64,
}

impl CountingStream {
    fn new(
        inner: impl futures_util::Stream<Item = reqwest::Result<Bytes>> + Send + 'static,
        ctx: CountCtx,
    ) -> Self {
        CountingStream {
            inner: Box::pin(inner),
            ctx: Some(ctx),
            bytes: 0,
        }
    }

    fn finalize(&mut self, error: Option<String>) {
        if let Some(ctx) = self.ctx.take() {
            if let Some(err) = error {
                let payload = serde_json::json!({
                    "path": ctx.path,
                    "upstream_host": ctx.host,
                    "error": err,
                    "bytes_out": self.bytes,
                });
                crate::adapters::channel::audit::log_event(
                    &ctx.ledger,
                    "guard_stream_error",
                    &payload,
                );
                return;
            }
            super::ledger::log_request(
                &ctx.ledger,
                RequestRecord {
                    method: &ctx.method,
                    path: &ctx.path,
                    upstream_host: &ctx.host,
                    status: ctx.status,
                    bytes_in: ctx.bytes_in,
                    bytes_out: self.bytes,
                    findings: &ctx.findings,
                },
            );
        }
    }
}

impl futures_util::Stream for CountingStream {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.bytes += chunk.len() as u64;
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => {
                this.finalize(Some(e.to_string()));
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                this.finalize(None);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use axum::routing::any;
    use std::sync::Mutex;

    fn guard_settings(on_secret: crate::config::registry::SecretPolicy) -> GuardSettings {
        GuardSettings {
            strip_images: true,
            on_secret,
            trusted_providers: vec!["native".to_string()],
        }
    }

    async fn spawn_app(router: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        });
        format!("http://{addr}")
    }

    async fn spawn_guard(upstream: &str, settings: GuardSettings, ledger: &str) -> String {
        let router = build_router(
            upstream.to_string(),
            settings,
            "zai".to_string(),
            ledger.to_string(),
        )
        .expect("guard router");
        spawn_app(router).await
    }

    #[tokio::test]
    async fn image_block_stripped_before_reaching_upstream() {
        let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
        let cap = captured.clone();
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(move |body: Bytes| {
                let cap = cap.clone();
                async move {
                    *cap.lock().expect("lock") = Some(serde_json::from_slice(&body).expect("json"));
                    Json(serde_json::json!({"ok": true}))
                }
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        let body = serde_json::json!({
            "messages": [{"role": "user", "content": [
                {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "aGk="}}
            ]}]
        });
        let resp = reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 200);

        let seen = captured.lock().expect("lock").clone().expect("captured");
        let content = seen["messages"][0]["content"].as_array().expect("array");
        assert_eq!(content[0]["type"], "text");
        assert!(
            content[0]["text"]
                .as_str()
                .expect("text")
                .contains("image block removed")
        );
    }

    #[tokio::test]
    async fn sse_response_passthrough_byte_identical() {
        let chunks: Vec<Bytes> = vec![
            Bytes::from_static(b"event: message_start\ndata: {\"a\":1}\n\n"),
            Bytes::from_static(b"event: content_block_delta\ndata: {\"b\":2}\n\n"),
        ];
        let expected: Vec<u8> = chunks.concat().to_vec();
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(move || {
                let chunks = chunks.clone();
                async move {
                    let stream = futures_util::stream::iter(
                        chunks.into_iter().map(Ok::<Bytes, std::io::Error>),
                    );
                    Response::builder()
                        .status(200)
                        .header("content-type", "text/event-stream")
                        .body(Body::from_stream(stream))
                        .expect("resp")
                }
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        let resp = reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .body(r#"{"messages":[]}"#)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 200);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("text/event-stream")
        );
        let bytes = resp.bytes().await.expect("body");
        assert_eq!(bytes.to_vec(), expected);
    }

    #[tokio::test]
    async fn auth_and_anthropic_headers_forwarded() {
        let captured: Arc<Mutex<Option<HeaderMap>>> = Arc::new(Mutex::new(None));
        let cap = captured.clone();
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(move |headers: HeaderMap| {
                let cap = cap.clone();
                async move {
                    *cap.lock().expect("lock") = Some(headers);
                    Json(serde_json::json!({"ok": true}))
                }
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .header("authorization", "Bearer testtoken1234567890")
            .header("x-api-key", "sk-test-abcdef123456")
            .header("anthropic-version", "2023-06-01")
            .body(r#"{"messages":[]}"#)
            .send()
            .await
            .expect("send");

        let seen = captured.lock().expect("lock").clone().expect("captured");
        assert_eq!(
            seen.get("authorization").and_then(|v| v.to_str().ok()),
            Some("Bearer testtoken1234567890")
        );
        assert_eq!(
            seen.get("x-api-key").and_then(|v| v.to_str().ok()),
            Some("sk-test-abcdef123456")
        );
        assert_eq!(
            seen.get("anthropic-version").and_then(|v| v.to_str().ok()),
            Some("2023-06-01")
        );
    }

    #[tokio::test]
    async fn upstream_unreachable_returns_502_anthropic_error_shape() {
        // Bind then drop to get a guaranteed-closed port.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        drop(listener);
        let dead_upstream = format!("http://{addr}");

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &dead_upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        let resp = reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .body(r#"{"messages":[]}"#)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 502);
        let body: serde_json::Value = resp.json().await.expect("json");
        assert_eq!(body["type"], "error");
        assert_eq!(body["error"]["type"], "api_error");
    }

    #[tokio::test]
    async fn redirect_from_upstream_not_followed() {
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(|| async {
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header("location", "https://elsewhere.example/v1/messages")
                    .body(Body::empty())
                    .expect("resp")
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        // The test client must not follow the redirect either, or the
        // assertion would observe the post-redirect result.
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");
        let resp = client
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .body(r#"{"messages":[]}"#)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), StatusCode::FOUND);
    }

    #[tokio::test]
    async fn non_json_body_passthrough_with_no_upstream_mutation() {
        let captured: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let cap = captured.clone();
        let upstream = spawn_app(Router::new().route(
            "/upload",
            any(move |body: Bytes| {
                let cap = cap.clone();
                async move {
                    *cap.lock().expect("lock") = body.to_vec();
                    StatusCode::OK
                }
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            ledger.to_str().expect("path"),
        )
        .await;

        let raw = b"\x00\x01binary-not-json\xff".to_vec();
        let resp = reqwest::Client::new()
            .post(format!("{guard}/upload"))
            .header("content-type", "application/octet-stream")
            .body(raw.clone())
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 200);
        assert_eq!(*captured.lock().expect("lock"), raw);
    }

    #[tokio::test]
    async fn block_policy_returns_400_and_never_contacts_upstream() {
        let contacted = Arc::new(AtomicBool::new(false));
        let hit = contacted.clone();
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(move || {
                let hit = hit.clone();
                async move {
                    hit.store(true, Ordering::SeqCst);
                    Json(serde_json::json!({"ok": true}))
                }
            }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger = ledger_dir.path().join("l.jsonl");
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Block),
            ledger.to_str().expect("path"),
        )
        .await;

        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "leak: Bearer abcdefghijklmnop123456"}]
        });
        let resp = reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 400);
        assert!(
            !contacted.load(Ordering::SeqCst),
            "upstream must not be contacted"
        );
    }

    #[tokio::test]
    async fn ledger_records_redacted_preview_only() {
        let upstream = spawn_app(Router::new().route(
            "/v1/messages",
            any(|| async { Json(serde_json::json!({"ok": true})) }),
        ))
        .await;

        let ledger_dir = tempfile::tempdir().expect("tempdir");
        let ledger_path = ledger_dir.path().join("l.jsonl");
        let ledger = ledger_path.to_str().expect("path").to_string();
        let guard = spawn_guard(
            &upstream,
            guard_settings(crate::config::registry::SecretPolicy::Redact),
            &ledger,
        )
        .await;

        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "leak: sk-ant-api03-abcdefghij1234567890AB"}]
        });
        let resp = reqwest::Client::new()
            .post(format!("{guard}/v1/messages"))
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .expect("send");
        assert_eq!(resp.status(), 200);

        let content = std::fs::read_to_string(&ledger_path).expect("ledger");
        assert!(content.contains("guard_request"));
        assert!(
            !content.contains("sk-ant-api03-abcdefghij1234567890AB"),
            "raw secret must never be written"
        );
    }
}
