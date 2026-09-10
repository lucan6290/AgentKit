use tauri::State;

use crate::contracts::FrontendLogPayload;
use crate::error::AppResult;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn write_frontend_log(
    _state: State<'_, AppState>,
    payload: FrontendLogPayload,
) -> AppResult<()> {
    crate::logging::emit_frontend_log(payload);
    Ok(())
}
