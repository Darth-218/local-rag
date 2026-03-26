use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatData {
    pub chats: Vec<Chat>,
    pub messages: Vec<Message>,
}

impl Default for ChatData {
    fn default() -> Self {
        Self {
            chats: Vec::new(),
            messages: Vec::new(),
        }
    }
}

pub fn load_chat_data(chats_dir: &std::path::Path) -> Result<ChatData, String> {
    let data_path = chats_dir.join("data.json");
    if !data_path.exists() {
        return Ok(ChatData::default());
    }
    let content = fs::read_to_string(&data_path)
        .map_err(|e| format!("Failed to read chat data: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse chat data: {}", e))
}

pub fn save_chat_data(chats_dir: &std::path::Path, data: &ChatData) -> Result<(), String> {
    fs::create_dir_all(chats_dir)
        .map_err(|e| format!("Failed to create chats directory: {}", e))?;
    let data_path = chats_dir.join("data.json");
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize chat data: {}", e))?;
    fs::write(&data_path, json)
        .map_err(|e| format!("Failed to write chat data: {}", e))
}

pub fn create_new_chat(data: &mut ChatData, title: Option<String>) -> Chat {
    let now = Utc::now();
    let chat = Chat {
        id: Uuid::new_v4().to_string(),
        title: title.unwrap_or_else(|| format!("New Chat")),
        created_at: now,
        updated_at: now,
    };
    data.chats.insert(0, chat.clone());
    chat
}

pub fn delete_chat_by_id(data: &mut ChatData, chat_id: &str) -> bool {
    let initial_len = data.chats.len();
    data.chats.retain(|c| c.id != chat_id);
    data.messages.retain(|m| m.chat_id != chat_id);
    data.chats.len() < initial_len
}

pub fn rename_chat(data: &mut ChatData, chat_id: &str, new_title: &str) -> Option<Chat> {
    if let Some(chat) = data.chats.iter_mut().find(|c| c.id == chat_id) {
        chat.title = new_title.to_string();
        chat.updated_at = Utc::now();
        return Some(chat.clone());
    }
    None
}

pub fn get_chat_messages(data: &ChatData, chat_id: &str) -> Vec<Message> {
    let mut messages: Vec<Message> = data.messages
        .iter()
        .filter(|m| m.chat_id == chat_id)
        .cloned()
        .collect();
    messages.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    messages
}

pub fn add_message(data: &mut ChatData, chat_id: &str, role: MessageRole, content: &str) -> Message {
    let message = Message {
        id: Uuid::new_v4().to_string(),
        chat_id: chat_id.to_string(),
        role,
        content: content.to_string(),
        created_at: Utc::now(),
    };
    data.messages.push(message.clone());
    
    if let Some(chat) = data.chats.iter_mut().find(|c| c.id == chat_id) {
        chat.updated_at = Utc::now();
    }
    
    message
}
