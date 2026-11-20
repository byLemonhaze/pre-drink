use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Conversation {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub created_at: Option<f64>,
    pub updated_at: Option<f64>,
    pub private: bool,
    pub sensitive: bool,
    pub message_count: i64,
    pub preview: String,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: Option<f64>,
    pub branch_id: Option<String>,
    pub parent_id: Option<String>,
    pub is_active: bool,
    pub sensitive: bool,
    pub position: i64,
}

#[derive(Debug)]
pub struct ParsedConversation {
    pub platform: String,
    pub id: String,
    pub title: String,
    pub created_at: Option<f64>,
    pub updated_at: Option<f64>,
    pub messages: Vec<Message>,
    pub raw_json: String,
    pub sensitive: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct ImportReport {
    pub conversations: usize,
    pub messages: usize,
    pub new_conversations: usize,
    pub updated_conversations: usize,
    pub skipped_messages: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub conversations: i64,
    pub messages: i64,
    pub private: i64,
    pub sensitive: i64,
    pub database_path: String,
}
