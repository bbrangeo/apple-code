//! Configuration loading (env + config file)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

const CONFIG_DIR_NAME: &str = "apple-code-assistant";
const CONFIG_FILE_NAME: &str = "config.toml";

/// Prompt template configuration (inspired by smartcat's prompts.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    /// Optional override for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional override for temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional system prompt snippet to prepend to the default system behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Name of the default prompt template (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_prompt: Option<String>,
    /// Prompt templates loaded from config file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<HashMap<String, PromptConfig>>,
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: None,
            default_language: Some("typescript".to_string()),
            theme: Some("dark".to_string()),
            max_tokens: Some(4000),
            temperature: Some(0.7),
            default_prompt: None,
            prompts: None,
            config_file: None,
        }
    }
}

impl Config {
    /// Default config file path under user config dir.
    pub fn default_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME).join(CONFIG_FILE_NAME))
    }

    /// Load config: first .env (if present), then config file (if path given or default exists), then defaults.
    pub fn load(config_file_override: Option<&str>) -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        let mut config = Self::default();

        let config_path: Option<PathBuf> = config_file_override
            .map(PathBuf::from)
            .or_else(Self::default_config_path);

        if let Some(ref path) = config_path {
            if path.exists() {
                let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e))?;
                let file_config: ConfigFile = toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
                config.merge_file(file_config);
                config.config_file = Some(path.clone());
            }
        }

        config.merge_env();
        Ok(config)
    }

    fn merge_file(&mut self, f: ConfigFile) {
        if f.model.is_some() {
            self.model = f.model;
        }
        if f.default_language.is_some() {
            self.default_language = f.default_language;
        }
        if f.theme.is_some() {
            self.theme = f.theme;
        }
        if f.max_tokens.is_some() {
            self.max_tokens = f.max_tokens;
        }
        if f.temperature.is_some() {
            self.temperature = f.temperature;
        }
        if f.default_prompt.is_some() {
            self.default_prompt = f.default_prompt;
        }
        if let Some(prompts) = f.prompts {
            // Simple overwrite for now; could be merged per key later if needed.
            self.prompts = Some(prompts);
        }
    }

    fn merge_env(&mut self) {
        if let Ok(v) = std::env::var("APPLE_FOUNDATION_MODEL") {
            if !v.is_empty() {
                self.model = Some(v);
            }
        }
        if let Ok(v) = std::env::var("APPLE_CODE_DEFAULT_LANGUAGE") {
            if !v.is_empty() {
                self.default_language = Some(v);
            }
        }
        if let Ok(v) = std::env::var("APPLE_CODE_THEME") {
            if !v.is_empty() {
                self.theme = Some(v);
            }
        }
        if let Ok(v) = std::env::var("APPLE_CODE_MAX_TOKENS") {
            if let Ok(n) = v.parse::<u32>() {
                self.max_tokens = Some(n);
            }
        }
        if let Ok(v) = std::env::var("APPLE_CODE_TEMPERATURE") {
            if let Ok(n) = v.parse::<f32>() {
                self.temperature = Some(n);
            }
        }
    }

    /// Save current config to the given path (or default path).
    pub fn save(&self, path: Option<&Path>) -> Result<(), ConfigError> {
        let path = path
            .map(PathBuf::from)
            .or_else(Self::default_config_path)
            .ok_or_else(|| ConfigError::Invalid("no config path".to_string()))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let file = ConfigFile {
            model: self.model.clone(),
            default_language: self.default_language.clone(),
            theme: self.theme.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            default_prompt: self.default_prompt.clone(),
            prompts: self.prompts.clone(),
        };
        let toml = toml::to_string_pretty(&file).map_err(|e| ConfigError::Invalid(e.to_string()))?;
        std::fs::write(&path, toml).map_err(ConfigError::Io)?;
        Ok(())
    }

    /// Get a value by key name (for --get).
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "model" | "APPLE_FOUNDATION_MODEL" => self.model.clone(),
            "default_language" | "APPLE_CODE_DEFAULT_LANGUAGE" => self.default_language.clone(),
            "theme" | "APPLE_CODE_THEME" => self.theme.clone(),
            "max_tokens" | "APPLE_CODE_MAX_TOKENS" => self.max_tokens.map(|n| n.to_string()),
            "temperature" | "APPLE_CODE_TEMPERATURE" => self.temperature.map(|n| n.to_string()),
            _ => None,
        }
    }

    /// Set a value by key (for --set key=value).
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), ConfigError> {
        match key {
            "model" | "APPLE_FOUNDATION_MODEL" => self.model = Some(value.to_string()),
            "default_language" | "APPLE_CODE_DEFAULT_LANGUAGE" => self.default_language = Some(value.to_string()),
            "theme" | "APPLE_CODE_THEME" => self.theme = Some(value.to_string()),
            "max_tokens" | "APPLE_CODE_MAX_TOKENS" => {
                self.max_tokens = Some(value.parse().map_err(|_| ConfigError::Invalid(format!("invalid number: {}", value)))?);
            }
            "temperature" | "APPLE_CODE_TEMPERATURE" => {
                self.temperature = Some(value.parse().map_err(|_| ConfigError::Invalid(format!("invalid number: {}", value)))?);
            }
            _ => return Err(ConfigError::Invalid(format!("unknown key: {}", key))),
        }
        Ok(())
    }

    /// All keys for --list.
    pub fn keys() -> &'static [&'static str] {
        &["model", "default_language", "theme", "max_tokens", "temperature"]
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    model: Option<String>,
    default_language: Option<String>,
    theme: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
     /// Optional default prompt template name.
    default_prompt: Option<String>,
    /// Optional map of prompt templates.
    prompts: Option<HashMap<String, PromptConfig>>,
}

impl Config {
    /// Resolve a prompt template either by explicit name or by using default_prompt.
    pub fn resolve_prompt<'a>(&'a self, name: Option<&str>) -> Option<(&'a str, &'a PromptConfig)> {
        let prompts = self.prompts.as_ref()?;
        let key = if let Some(name) = name {
            name
        } else if let Some(default) = self.default_prompt.as_deref() {
            default
        } else {
            return None;
        };
        prompts
            .get_key_value(key)
            .map(|(k, v)| (k.as_str(), v))
    }
}
