//! OpenAI-compatible text chat over Axum, using the native bridge owned by apple_ai.

use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use apple_ai::{AppleAiClient, AppleAiError, GenerationOptions};
use axum::extract::{rejection::JsonRejection, DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use tokio::sync::{mpsc, Semaphore};
use tokio::time::{self, Instant};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

const MODEL: &str = "apple-intelligence";
// Bounds protect the local model from unbounded network input and stalled generations.
const BODY_LIMIT: usize = 256 * 1024;
const GENERATION_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) fn run(address: SocketAddr) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let client = Arc::new(AppleAiClient::new()?);
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .with_context(|| format!("binding HTTP server to {address}"))?;
        if !address.ip().is_loopback() {
            eprintln!(
                "Warning: this API has no authentication or TLS. Use only on a trusted network."
            );
        }
        eprintln!("Apple Intelligence API listening on http://{address}");
        axum::serve(listener, router(client))
            .with_graceful_shutdown(shutdown())
            .await?;
        Ok(())
    })
}

async fn shutdown() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = terminate.recv() => {},
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

#[derive(Clone)]
struct AppState {
    client: Arc<AppleAiClient>,
    slots: Arc<Semaphore>,
}

fn router(client: Arc<AppleAiClient>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat))
        .fallback(|| async {
            ApiError::new(StatusCode::NOT_FOUND, "not_found", "Unknown endpoint")
        })
        .method_not_allowed_fallback(|| async {
            ApiError::new(
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "Method not allowed",
            )
        })
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(AppState {
            client,
            // ponytail: one inference at a time; increase only after measuring model contention.
            slots: Arc::new(Semaphore::new(1)),
        })
}

async fn health() -> Result<Json<Value>, ApiError> {
    AppleAiClient::check_availability().map_err(ApiError::from)?;
    Ok(Json(json!({"status": "ok", "model": MODEL})))
}

async fn models() -> Json<Value> {
    Json(json!({"object": "list", "data": [{
        "id": MODEL, "object": "model", "created": 0, "owned_by": "apple"
    }]}))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(default)]
    stream: bool,
    temperature: Option<f64>,
    #[serde(alias = "max_completion_tokens")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Role {
    System,
    User,
    Assistant,
}

impl ChatRequest {
    fn validate(&self) -> Result<(), ApiError> {
        if self.model != MODEL {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "model_not_found",
                "Only apple-intelligence is available",
            ));
        }
        if self.messages.is_empty() || self.messages.len() > 256 {
            return Err(ApiError::invalid(
                "messages must contain between 1 and 256 entries",
            ));
        }
        let mut expected = Role::User;
        let mut in_history = false;
        for message in &self.messages {
            if message.content.trim().is_empty() {
                return Err(ApiError::invalid("Message content must not be empty"));
            }
            if !in_history && message.role == Role::System {
                continue;
            }
            in_history = true;
            if message.role != expected {
                return Err(ApiError::invalid("After initial system messages, roles must alternate user/assistant and end with user"));
            }
            expected = if expected == Role::User {
                Role::Assistant
            } else {
                Role::User
            };
        }
        if !in_history || expected != Role::Assistant {
            return Err(ApiError::invalid("The last message must have role user"));
        }
        if self
            .temperature
            .is_some_and(|t| !t.is_finite() || !(0.0..=2.0).contains(&t))
        {
            return Err(ApiError::invalid("temperature must be between 0 and 2"));
        }
        if self.max_tokens.is_some_and(|n| !(1..=8192).contains(&n)) {
            return Err(ApiError::invalid("max_tokens must be between 1 and 8192"));
        }
        Ok(())
    }
}

async fn chat(
    State(state): State<AppState>,
    body: Result<Json<ChatRequest>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) = body.map_err(|e| {
        let status = if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            StatusCode::PAYLOAD_TOO_LARGE
        } else {
            StatusCode::BAD_REQUEST
        };
        ApiError::new(status, "invalid_request_error", &e.body_text())
    })?;
    request.validate()?;
    let permit = state.slots.clone().try_acquire_owned().map_err(|_| {
        ApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "server_busy",
            "Another generation is running; retry later",
        )
    })?;
    let streaming = request.stream;
    let mut generation = Generation::start(&state.client, request)?;
    // Catch native errors before committing HTTP 200 and SSE headers.
    let first = generation.next().await?;
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4().simple());
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(ApiError::internal)?
        .as_secs();
    if !streaming {
        let mut text = String::new();
        let mut event = first;
        while let BridgeEvent::Delta(delta) = event {
            text.push_str(&delta);
            event = generation.next().await?;
        }
        return Ok(Json(json!({
            "id": id, "object": "chat.completion", "created": created, "model": MODEL,
            "choices": [{"index": 0, "message": {"role": "assistant", "content": text}, "finish_reason": "stop"}]
        })).into_response());
    }

    let (sender, receiver) = mpsc::channel::<Result<Event, Infallible>>(8);
    tokio::spawn(async move {
        let _permit = permit;

        let result = time::timeout_at(generation.deadline, async {
            send(
                &sender,
                chunk(&id, created, json!({"role": "assistant"}), Value::Null),
            )
            .await?;
            let mut event = first;
            loop {
                match event {
                    BridgeEvent::Delta(delta) => {
                        send(
                            &sender,
                            chunk(&id, created, json!({"content": delta}), Value::Null),
                        )
                        .await?;
                    }
                    BridgeEvent::Done => {
                        send(&sender, chunk(&id, created, json!({}), json!("stop"))).await?;
                        return send(&sender, "[DONE]".into()).await;
                    }
                }
                event = tokio::select! {
                    _ = sender.closed() => return Ok(()),
                    event = generation.next() => event?,
                };
            }
        })
        .await
        .unwrap_or_else(|_| Err(ApiError::timeout()));
        if let Err(error) = result {
            // Once SSE starts, failures are events, not a fabricated successful completion.
            let _ = time::timeout(
                Duration::from_secs(1),
                send(&sender, error.body().to_string()),
            )
            .await;
        }
    });
    Ok(Sse::new(ReceiverStream::new(receiver))
        .keep_alive(KeepAlive::default())
        .into_response())
}

fn chunk(id: &str, created: u64, delta: Value, finish_reason: Value) -> String {
    json!({"id": id, "object": "chat.completion.chunk", "created": created, "model": MODEL,
        "choices": [{"index": 0, "delta": delta, "finish_reason": finish_reason}]})
    .to_string()
}

async fn send(
    sender: &mpsc::Sender<Result<Event, Infallible>>,
    data: String,
) -> Result<(), ApiError> {
    sender
        .send(Ok(Event::default().data(data)))
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::BAD_REQUEST,
                "client_disconnected",
                "Client disconnected",
            )
        })
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: String,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &str, message: &str) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
        }
    }

    fn invalid(message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request_error", message)
    }

    fn internal(error: impl std::fmt::Display) -> Self {
        log::error!("Chat backend error: {error}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Internal backend error",
        )
    }

    fn timeout() -> Self {
        Self::new(
            StatusCode::GATEWAY_TIMEOUT,
            "generation_timeout",
            "Apple Intelligence request timed out",
        )
    }

    fn body(&self) -> Value {
        let kind = if self.status.is_client_error() {
            "invalid_request_error"
        } else {
            "server_error"
        };
        json!({"error": {"message": self.message, "type": kind, "param": null, "code": self.code}})
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body())).into_response()
    }
}

impl From<AppleAiError> for ApiError {
    fn from(error: AppleAiError) -> Self {
        let (status, code) = match &error {
            AppleAiError::ContextLengthExceeded(_) => {
                (StatusCode::BAD_REQUEST, "context_length_exceeded")
            }
            AppleAiError::InvalidMessages(_) | AppleAiError::InvalidOptions(_) => {
                (StatusCode::BAD_REQUEST, "invalid_request_error")
            }
            AppleAiError::DeviceNotEligible
            | AppleAiError::NotEnabled
            | AppleAiError::ModelNotReady
            | AppleAiError::Unavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "model_unavailable")
            }
            AppleAiError::Busy => (StatusCode::TOO_MANY_REQUESTS, "server_busy"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "generation_error"),
        };
        Self::new(status, code, &error.to_string())
    }
}

enum BridgeEvent {
    Delta(String),
    Done,
}

struct Generation {
    stream: Pin<Box<dyn Stream<Item = Result<String, AppleAiError>> + Send>>,
    deadline: Instant,
}

impl Generation {
    fn start(client: &AppleAiClient, request: ChatRequest) -> Result<Self, ApiError> {
        let messages = request
            .messages
            .into_iter()
            .map(|message| match message.role {
                Role::System => apple_ai::Message::system(message.content),
                Role::User => apple_ai::Message::user(message.content),
                Role::Assistant => apple_ai::Message::assistant(message.content),
            })
            .collect();
        let options = GenerationOptions::default()
            .temperature(request.temperature.unwrap_or(0.7))
            .max_tokens(request.max_tokens.unwrap_or(512) as i32);
        // Buffer this same native stream for non-streaming HTTP responses too.
        let stream = client.generate_stream_result_async(messages, options)?;
        Ok(Self {
            stream: Box::pin(stream),
            deadline: Instant::now() + GENERATION_TIMEOUT,
        })
    }

    async fn next(&mut self) -> Result<BridgeEvent, ApiError> {
        match time::timeout_at(self.deadline, self.stream.next()).await {
            Ok(Some(Ok(text))) => Ok(BridgeEvent::Delta(text)),
            Ok(Some(Err(error))) => Err(error.into()),
            Ok(None) => Ok(BridgeEvent::Done),
            Err(_) => Err(ApiError::timeout()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    fn app() -> Router {
        // Invalid requests and model listing never invoke the native backend.
        router(Arc::new(AppleAiClient))
    }

    fn request(value: Value) -> ChatRequest {
        serde_json::from_value(value).expect("valid test request")
    }

    #[test]
    fn validates_roles_parameters_and_unsupported_features() {
        let valid = json!({"model": MODEL, "messages": [
            {"role":"system", "content":"Be brief"},
            {"role":"user", "content":"Hello"},
            {"role":"assistant", "content":"Hi"},
            {"role":"user", "content":"Explain Rust"}
        ]});
        assert!(request(valid.clone()).validate().is_ok());
        for patch in [
            json!({"model":"other"}),
            json!({"messages":[]}),
            json!({"temperature": 2.1}),
            json!({"max_tokens":0}),
            json!({"messages":[{"role":"assistant","content":"Hi"}]}),
            json!({"messages":[{"role":"user","content":"Hi"},{"role":"system","content":"Rule"},{"role":"user","content":"Hi"}]}),
            json!({"messages":[{"role":"user","content":" "}]}),
            json!({"messages":[{"role":"user","content":"Hi"},{"role":"user","content":"Again"}]}),
        ] {
            let mut input = valid.clone();
            for (key, value) in patch.as_object().expect("object patch") {
                input[key] = value.clone();
            }
            assert!(request(input).validate().is_err(), "patch {patch}");
        }
        let mut tools = valid;
        tools["tools"] = json!([]);
        assert!(serde_json::from_value::<ChatRequest>(tools).is_err());
    }

    #[tokio::test]
    async fn routes_models_and_openai_errors_without_model_hardware() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/v1/models")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), BODY_LIMIT)
            .await
            .expect("body");
        let models: Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(models["data"][0]["id"], MODEL);
        for (path, body, expected) in [
            ("/v1/chat/completions", "{", StatusCode::BAD_REQUEST),
            (
                "/v1/chat/completions",
                r#"{"model":"other","messages":[]}"#,
                StatusCode::NOT_FOUND,
            ),
            ("/missing", "{}", StatusCode::NOT_FOUND),
        ] {
            let response = app()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(path)
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), expected);
            let bytes = to_bytes(response.into_body(), BODY_LIMIT)
                .await
                .expect("body");
            let error: Value = serde_json::from_slice(&bytes).expect("JSON error");
            assert!(error["error"]["message"].is_string());
        }
    }

    #[tokio::test]
    #[ignore = "requires an available Apple Intelligence model"]
    async fn native_backend_serves_health_json_and_sse() {
        let app = router(Arc::new(AppleAiClient::new().expect("available model")));
        let health = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(health.status(), StatusCode::OK);
        for streaming in [false, true] {
            let body = json!({"model": MODEL, "messages": [
                {"role": "system", "content": "Reply briefly in French."},
                {"role": "user", "content": "Say hello."}
            ], "stream": streaming, "max_tokens": 64});
            let response = time::timeout(
                Duration::from_secs(60),
                app.clone().oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/v1/chat/completions")
                        .header("content-type", "application/json")
                        .body(Body::from(body.to_string()))
                        .expect("request"),
                ),
            )
            .await
            .expect("response timeout")
            .expect("response");
            assert_eq!(response.status(), StatusCode::OK);
            let content_type = response.headers()["content-type"]
                .to_str()
                .expect("content type")
                .to_owned();
            let bytes = time::timeout(
                Duration::from_secs(60),
                to_bytes(response.into_body(), BODY_LIMIT),
            )
            .await
            .expect("body timeout")
            .expect("body");
            if !streaming {
                assert!(content_type.starts_with("application/json"));
                let response: Value = serde_json::from_slice(&bytes).expect("completion JSON");
                assert_eq!(response["object"], "chat.completion");
                assert_eq!(response["choices"][0]["finish_reason"], "stop");
                assert!(!response["choices"][0]["message"]["content"]
                    .as_str()
                    .expect("text")
                    .is_empty());
                assert!(response.get("usage").is_none(), "no invented token counts");
            } else {
                assert!(content_type.starts_with("text/event-stream"));
                let text = std::str::from_utf8(&bytes).expect("UTF-8 SSE");
                let data: Vec<_> = text
                    .lines()
                    .filter_map(|line| line.strip_prefix("data: "))
                    .collect();
                assert_eq!(data.last(), Some(&"[DONE]"));
                let chunks: Vec<Value> = data[..data.len() - 1]
                    .iter()
                    .map(|line| serde_json::from_str(line).expect("chunk JSON"))
                    .collect();
                assert_eq!(chunks[0]["choices"][0]["delta"]["role"], "assistant");
                let id = chunks[0]["id"].clone();
                let mut content = String::new();
                for chunk in &chunks {
                    assert!(chunk.get("error").is_none(), "unexpected error: {chunk}");
                    assert_eq!(chunk["id"], id);
                    assert_eq!(chunk["object"], "chat.completion.chunk");
                    if let Some(delta) = chunk["choices"][0]["delta"]["content"].as_str() {
                        content.push_str(delta);
                    }
                }
                assert!(!content.trim().is_empty());
                assert_eq!(
                    chunks.last().expect("stop chunk")["choices"][0]["finish_reason"],
                    "stop"
                );
            }
        }
    }

    #[test]
    fn backend_errors_map_to_http_statuses() {
        assert_eq!(
            ApiError::from(AppleAiError::ContextLengthExceeded("too long".into())).status,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::from(AppleAiError::NotEnabled).status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            ApiError::from(AppleAiError::Busy).status,
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn streaming_chunks_preserve_unicode_and_completion_identity() {
        let value: Value = serde_json::from_str(&chunk(
            "chatcmpl-test",
            42,
            json!({"content":"été 🦀"}),
            Value::Null,
        ))
        .expect("chunk JSON");
        assert_eq!(value["choices"][0]["delta"]["content"], "été 🦀");
        assert_eq!(value["id"], "chatcmpl-test");
        assert_eq!(value["object"], "chat.completion.chunk");
        assert!(value["choices"][0]["finish_reason"].is_null());
    }
}
