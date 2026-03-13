//! Apple Foundation Models client and code generation trait

mod mock;

#[cfg(feature = "apple_ai")]
mod apple_ai_client;

use crate::types::{GenerateRequest, GenerateResponse};

/// Client for code generation (Apple Foundation or mock).
pub trait CodeGenClient: Send + Sync {
    /// Generate code from the given request.
    fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse, crate::error::ApiError>;
}

/// Create the default client (Apple AI when available, otherwise mock).
#[cfg(feature = "apple_ai")]
pub fn default_client() -> Box<dyn CodeGenClient> {
    match apple_ai_client::AppleAiCodeGenClient::new() {
        Ok(client) => {
            log::info!("Using Apple AI client.");
            Box::new(client)
        }
        Err(e) => {
            log::warn!("Apple AI client unavailable: {}. Falling back to mock client.", e);
            Box::new(mock::MockClient::new())
        }
    }
}

#[cfg(not(feature = "apple_ai"))]
pub fn default_client() -> Box<dyn CodeGenClient> {
    log::info!("Apple AI feature disabled. Using mock client.");
    Box::new(mock::MockClient::new())
}

