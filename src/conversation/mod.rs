//! Conversation and session management

mod manager;

pub use manager::ConversationManager;

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        })
    }
}

pub fn format_history(history: &[Message]) -> String {
    history
        .iter()
        .map(|message| format!("[{}] {}\n", message.role, message.content))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::{format_history, Message, Role};

    #[test]
    fn history_keeps_roles_multiline_content_and_trailing_newlines() {
        assert_eq!(format_history(&[]), "", "empty history");
        let history = [
            Message {
                role: Role::User,
                content: "first\nsecond".into(),
            },
            Message {
                role: Role::Assistant,
                content: "é漢字\n".into(),
            },
        ];
        assert_eq!(
            format_history(&history),
            "[user] first\nsecond\n[assistant] é漢字\n\n",
            "history formatting must remain compatible with existing prompts"
        );
    }
}
