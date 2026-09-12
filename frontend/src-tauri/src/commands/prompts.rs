use tauri::State;

use crate::contracts::PromptDto;
use crate::error::AppResult;
use crate::models::PromptFileLink;
use crate::services;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn list_prompts(state: State<'_, AppState>) -> AppResult<Vec<PromptDto>> {
    services::list_prompts(&state.db)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_prompt(
    state: State<'_, AppState>,
    name: String,
    content: String,
) -> AppResult<PromptDto> {
    services::create_prompt(&state.db, name, content)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_prompt(
    state: State<'_, AppState>,
    id: String,
    name: String,
    content: String,
) -> AppResult<PromptDto> {
    services::update_prompt(&state.db, &id, name, content)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn duplicate_prompt(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
) -> AppResult<PromptDto> {
    services::duplicate_prompt(&state.db, &id, name)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_prompt(state: State<'_, AppState>, id: String) -> AppResult<()> {
    services::delete_prompt(&state.db, &id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_prompt_file(
    state: State<'_, AppState>,
    file_path: String,
    name: Option<String>,
) -> AppResult<PromptDto> {
    services::import_prompt_file(&state.db, file_path, name)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_prompt_file_link(
    state: State<'_, AppState>,
    prompt_id: String,
    file_path: String,
    write_back_enabled: Option<bool>,
) -> AppResult<PromptFileLink> {
    services::create_prompt_file_link(&state.db, &prompt_id, file_path, write_back_enabled)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn unlink_prompt_file(state: State<'_, AppState>, link_id: String) -> AppResult<()> {
    services::unlink_prompt_file(&state.db, &link_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn refresh_prompt_file_link(
    state: State<'_, AppState>,
    link_id: String,
) -> AppResult<PromptDto> {
    services::refresh_prompt_file_link(&state.db, &link_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn write_prompt_to_file(
    state: State<'_, AppState>,
    link_id: String,
    force: Option<bool>,
) -> AppResult<PromptFileLink> {
    services::write_prompt_to_file(&state.db, &link_id, force)
}
