//! Shared types for CLI args, requests, and responses

/// Code generation request (prompt + options).
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
    pub language: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub model: Option<String>,
    pub context: Option<String>,
}

/// Code generation response (generated code).
#[derive(Debug, Clone)]
pub struct GenerateResponse {
    pub code: String,
    pub language: Option<String>,
}

/// Placeholder for conversation message (session/streaming).
#[derive(Debug)]
pub struct ConversationMessage;
