//! Mock code generation client for development without Apple API

use super::CodeGenClient;
use crate::error::ApiError;
use crate::types::{GenerateRequest, GenerateResponse};

/// Mock client that returns a static snippet.
#[derive(Debug)]
pub struct MockClient;

impl MockClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenClient for MockClient {
    fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse, ApiError> {
        let lang = request.language.as_deref().unwrap_or("rust");
        let code = format!(
            "// Mock generated code for: {}\n// Language: {}\n\nfn main() {{\n    println!(\"Hello from mock\");\n}}\n",
            request.prompt,
            lang
        );
        Ok(GenerateResponse {
            code,
            language: request.language.clone(),
        })
    }
}
