use std::time::Instant;

use tauri::State;

use crate::contracts::ManagedSkillDto;
use crate::error::{AppError, AppResult};
use crate::models::Skill;
use crate::repositories::SkillsRepository;
use crate::services::install::{
    install_local_skill_from_selection, list_local_skills, upsert_skill_from_install,
    LocalSkillCandidate,
};
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_managed_skills(
    state: State<'_, AppState>,
    refresh: Option<bool>,
    source_type: Option<String>,
    sort: Option<String>,
) -> AppResult<Vec<ManagedSkillDto>> {
    crate::services::managed_skills::get_managed_skills(
        &state.db,
        refresh.unwrap_or(false),
        source_type.as_deref(),
        &sort.unwrap_or_else(|| "manual".to_string()),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_skill_enabled(
    state: State<'_, AppState>,
    skill_id: String,
    enabled: bool,
) -> AppResult<()> {
    let repo = SkillsRepository::new(&state.db);
    repo.set_enabled(&skill_id, enabled)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

fn delete_skill_cascade(state: &AppState, skill_id: &str) -> AppResult<()> {
    let repo = SkillsRepository::new(&state.db);
    // Also remove targets and tag links
    state
        .db
        .with_conn(|conn| {
            conn.execute("DELETE FROM skill_targets WHERE skill_id = ?1", [skill_id])?;
            conn.execute(
                "DELETE FROM skill_tag_links WHERE skill_id = ?1",
                [skill_id],
            )?;
            conn.execute(
                "DELETE FROM skill_scope_preference WHERE skill_id = ?1",
                [skill_id],
            )?;
            conn.execute("DELETE FROM skill_usage WHERE skill_id = ?1", [skill_id])?;
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "skills.delete.cascade.failed",
                layer = "backend",
                area = "skills",
                outcome = "failed",
                skill_id = %skill_id,
                error = %e,
                "failed to delete related skill records"
            );
            AppError::DatabaseError(e.to_string())
        })?;

    repo.delete(skill_id).map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "skills.delete.failed",
            layer = "backend",
            area = "skills",
            outcome = "failed",
            skill_id = %skill_id,
            error = %e,
            "failed to delete skill"
        );
        AppError::DatabaseError(e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_managed_skill(state: State<'_, AppState>, skill_id: String) -> AppResult<()> {
    delete_skill_cascade(&state, &skill_id)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_managed_skills(
    state: State<'_, AppState>,
    skill_ids: Vec<String>,
) -> AppResult<serde_json::Value> {
    for id in &skill_ids {
        delete_skill_cascade(&state, id)?;
    }
    Ok(serde_json::json!({ "removed": skill_ids.len() }))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_skill_source_url(
    state: State<'_, AppState>,
    skill_id: String,
    source_url: Option<String>,
) -> AppResult<Skill> {
    let repo = SkillsRepository::new(&state.db);
    repo.update_source_url(&skill_id, source_url.as_deref())
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    repo.get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_existing_skill(
    state: State<'_, AppState>,
    source_path: String,
    name: Option<String>,
    source_type: Option<String>,
) -> AppResult<serde_json::Value> {
    let source_type = source_type.unwrap_or_else(|| "community".to_string());
    let started = Instant::now();
    tracing::info!(target: crate::logging::app_target(), event = "skills.import.started", layer = "backend", area = "skills", outcome = "started", source = %source_path, source_type = %source_type, "skill import started");
    let path = std::path::Path::new(&source_path);

    if !path.is_dir() {
        tracing::warn!(target: crate::logging::app_target(), event = "skills.import.failed", layer = "backend", area = "skills", outcome = "failed", source = %source_path, source_type = %source_type, duration_ms = started.elapsed().as_millis() as u64, "skill import source is not a directory");
        return Err(AppError::InvalidInput(format!(
            "source path is not a directory: {}",
            source_path
        )));
    }

    let result = crate::services::install::install_local_skill(
        &state.db,
        path,
        name.as_deref(),
        None,
        &source_type,
    )
    .map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "skills.import.install.failed",
            layer = "backend",
            area = "skills",
            outcome = "failed",
            source_path = %source_path,
            source_type = %source_type,
            duration_ms = started.elapsed().as_millis() as u64,
            error = %e,
            "failed to install imported skill"
        );
        AppError::FileSystemError(e)
    })?;

    upsert_skill_from_install(&state.db, &result, &source_path, &source_type).map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "skills.import.upsert.failed",
            layer = "backend",
            area = "skills",
            outcome = "failed",
            source_path = %source_path,
            source_type = %source_type,
            error = %e,
            "failed to persist imported skill"
        );
        AppError::DatabaseError(e)
    })?;

    tracing::info!(target: crate::logging::app_target(), event = "skills.import.completed", layer = "backend", area = "skills", outcome = "success", source = %source_path, target = %result.community_path, skill_id = %result.skill_id, source_type = %source_type, file_count = ?result.skill_file_count, dir_size = ?result.skill_dir_size, duration_ms = started.elapsed().as_millis() as u64, "skill import completed");
    Ok(serde_json::json!({
        "skill_id": result.skill_id,
        "name": result.name,
        "community_path": result.community_path,
    }))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_local_skills_cmd(base_path: String) -> AppResult<Vec<LocalSkillCandidate>> {
    let path = std::path::Path::new(&base_path);
    list_local_skills(path).map_err(|e| AppError::FileSystemError(e))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn install_local_selection(
    state: State<'_, AppState>,
    base_path: String,
    subpath: String,
    name: Option<String>,
    source_type: Option<String>,
) -> AppResult<serde_json::Value> {
    let source_type = source_type.unwrap_or_else(|| "custom".to_string());
    let started = Instant::now();
    let full_source = std::path::Path::new(&base_path).join(&subpath);
    tracing::info!(target: crate::logging::app_target(), event = "skills.local_install.started", layer = "backend", area = "skills", outcome = "started", source = %full_source.display(), source_type = %source_type, "local skill installation started");
    let base = std::path::Path::new(&base_path);

    let result = install_local_skill_from_selection(
        &state.db,
        base,
        &subpath,
        name.as_deref(),
        None,
        &source_type,
    )
    .map_err(|e| {
        tracing::warn!(target: crate::logging::app_target(), event = "skills.local_install.failed", layer = "backend", area = "skills", outcome = "failed", source = %full_source.display(), source_type = %source_type, duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to install selected local skill");
        AppError::FileSystemError(e)
    })?;

    upsert_skill_from_install(
        &state.db,
        &result,
        &full_source.to_string_lossy(),
        &source_type,
    )
    .map_err(|e| {
        tracing::warn!(target: crate::logging::app_target(), event = "skills.local_install.failed", layer = "backend", area = "skills", outcome = "failed", source = %full_source.display(), skill_id = %result.skill_id, source_type = %source_type, duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to persist selected local skill");
        AppError::DatabaseError(e)
    })?;

    tracing::info!(target: crate::logging::app_target(), event = "skills.local_install.completed", layer = "backend", area = "skills", outcome = "success", source = %full_source.display(), target = %result.community_path, skill_id = %result.skill_id, source_type = %source_type, file_count = ?result.skill_file_count, dir_size = ?result.skill_dir_size, duration_ms = started.elapsed().as_millis() as u64, "local skill installation completed");
    Ok(serde_json::json!({
        "skill_id": result.skill_id,
        "name": result.name,
        "community_path": result.community_path,
        "description": result.description,
        "content_hash": result.content_hash,
    }))
}
