//! ConversationManager: create/load session, add message, persist

use std::fs;
use std::path::PathBuf;

use serde_json;
use uuid::Uuid;

use super::{Message, Role, Session};

const SESSIONS_DIR: &str = "apple-code-assistant/sessions";

pub struct ConversationManager {
    sessions_dir: PathBuf,
    current: Option<Session>,
}

impl ConversationManager {
    pub fn new() -> Self {
        let sessions_dir = dirs::config_dir()
            .map(|d| d.join(SESSIONS_DIR))
            .unwrap_or_else(|| PathBuf::from(".").join(".sessions"));
        Self {
            sessions_dir,
            current: None,
        }
    }

    fn ensure_dir(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.sessions_dir)
    }

    pub fn create_session(&mut self) -> &Session {
        let id = Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            messages: Vec::new(),
            created_at: chrono_iso(),
        };
        self.current = Some(session);
        self.current.as_ref().unwrap()
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.current.as_ref()
    }

    pub fn add_user_message(&mut self, content: &str) -> std::io::Result<()> {
        if self.current.is_none() {
            self.create_session();
        }
        if let Some(s) = self.current.as_mut() {
            s.messages.push(Message {
                role: Role::User,
                content: content.to_string(),
            });
            self.save_current()?;
        }
        Ok(())
    }

    pub fn add_assistant_message(&mut self, content: &str) -> std::io::Result<()> {
        if let Some(s) = self.current.as_mut() {
            s.messages.push(Message {
                role: Role::Assistant,
                content: content.to_string(),
            });
            self.save_current()?;
        }
        Ok(())
    }

    fn save_current(&self) -> std::io::Result<()> {
        if let Some(ref s) = self.current {
            self.ensure_dir()?;
            let path = self.sessions_dir.join(format!("{}.json", s.id));
            let json = serde_json::to_string_pretty(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> std::io::Result<Vec<String>> {
        self.ensure_dir()?;
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Some(stem) = path.file_stem() {
                    ids.push(stem.to_string_lossy().to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn load_session(&mut self, id: &str) -> std::io::Result<Option<&Session>> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.current = Some(session);
        Ok(self.current.as_ref().into())
    }

    pub fn history(&self) -> &[Message] {
        self.current
            .as_ref()
            .map(|s| s.messages.as_slice())
            .unwrap_or(&[])
    }
}

fn chrono_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("{}", t)
}
