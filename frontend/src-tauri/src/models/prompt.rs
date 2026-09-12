use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}
