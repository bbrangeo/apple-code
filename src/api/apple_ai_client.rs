//! Apple Intelligence code generation client (sync bridge via block_on).

use crate::api::CodeGenClient;
use crate::error::ApiError;
use crate::types::{GenerateRequest, GenerateResponse};

use apple_ai::{AppleAiClient, AppleAiError, GenerationOptions, Message};
use tokio::runtime::Runtime;

/// Code generation client backed by on-device Apple Intelligence.
pub struct AppleAiCodeGenClient {
    client: AppleAiClient,
    runtime: Runtime,
}

impl AppleAiCodeGenClient {
    /// Create a new client. Fails if Apple Intelligence is unavailable.
    pub fn new() -> Result<Self, ApiError> {
        let runtime = Runtime::new().map_err(|e| ApiError::Connection(e.to_string()))?;
        let client = runtime.block_on(async { AppleAiClient::new() }).map_err(map_apple_error)?;
        Ok(Self { client, runtime })
    }
}

impl CodeGenClient for AppleAiCodeGenClient {
    fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse, ApiError> {
        let mut messages = Vec::new();

        let mut system_parts = vec!["You are a code generator. Output only code, no explanation.".to_string()];
        if let Some(lang) = request.language.as_ref() {
            system_parts.push(format!("Language: {}", lang));
        }
        if let Some(ctx) = request.context.as_ref() {
            system_parts.push(format!("Context: {}", ctx));
        }
        messages.push(Message::system(system_parts.join(" ")));
        messages.push(Message::user(request.prompt.clone()));

        let options = GenerationOptions {
            temperature: request.temperature as f64,
            max_tokens: request.max_tokens as i32,
            ..Default::default()
        };

        let response = self
            .runtime
            .block_on(async { self.client.generate(messages, options).await })
            .map_err(map_apple_error)?;

        Ok(GenerateResponse {
            code: response.text,
            language: request.language.clone(),
        })
    }
}

fn map_apple_error(e: AppleAiError) -> ApiError {
    match e {
        AppleAiError::DeviceNotEligible => ApiError::Connection("Device not eligible for Apple Intelligence".into()),
        AppleAiError::NotEnabled => ApiError::Connection("Apple Intelligence not enabled in System Settings".into()),
        AppleAiError::ModelNotReady => ApiError::Connection("On-device model not ready (still downloading)".into()),
        AppleAiError::Unavailable(reason) => ApiError::Connection(format!("Apple Intelligence unavailable: {}", reason)),
        AppleAiError::Bridge(msg) => ApiError::Request(msg),
        _ => ApiError::Request(e.to_string()),
    }
}
