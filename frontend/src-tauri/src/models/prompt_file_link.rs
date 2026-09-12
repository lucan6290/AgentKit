use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PromptFileLink {
    pub id: String,
    pub prompt_id: String,
    pub file_path: String,
    pub write_back_enabled: bool,
    pub content_hash: Option<String>,
    pub exists_on_disk: bool,
    pub last_synced_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}
