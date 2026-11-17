//! Shared types for CLI args, requests, and responses

/// Code generation request (prompt + options).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
    pub language: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub model: Option<String>,
    pub context: Option<String>,
    /// When true, behave as a raw CLI tool (no markdown/explanations).
    pub tool_mode: bool,
}

/// Code generation response (generated code).
#[derive(Debug, Clone)]
pub struct GenerateResponse {
    pub code: String,
    pub language: Option<String>,
}
