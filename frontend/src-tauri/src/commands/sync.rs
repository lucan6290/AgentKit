use tauri::State;

use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::{ScopePreference, SkillTarget};
use crate::repositories::{
    RecentProjectsRepository, ScopePreferencesRepository, SkillTargetsRepository, SkillsRepository,
};
use crate::state::AppState;
use crate::tools::adapter::{
    self, effective_tool_adapters, resolve_default_path, resolve_project_path,
};
use crate::utils::path_safety;

fn safe_sync_target_path(
    base: &str,
    name: &str,
    fallback: &str,
    label: &str,
) -> AppResult<std::path::PathBuf> {
    let dir_name = path_safety::safe_dir_name_with_fallback(Some(name), fallback);
    path_safety::safe_child_path(base, &dir_name, label).map_err(AppError::PathError)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_skill_to_tool(
    state: State<'_, AppState>,
    source_path: String,
    skill_id: String,
    tool: String,
    name: Option<String>,
    scope: Option<String>,
    project_path: Option<String>,
    overwrite_if_same_content: Option<bool>,
) -> AppResult<()> {
    let scope = scope.unwrap_or_else(|| "global".to_string());
    let overwrite = overwrite_if_same_content.unwrap_or(true);

    let adapters = effective_tool_adapters(&state.db);
    let adapter = adapter::adapter_by_key(&adapters, &tool)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown tool: {}", tool)))?;

    let target_dir = if scope == "project" {
        let pp = project_path.as_deref().ok_or_else(|| {
            AppError::InvalidInput("project_path required for project scope".into())
        })?;
        resolve_project_path(adapter, pp)
    } else {
        resolve_default_path(adapter)
    };

    let skill_name = name.unwrap_or_else(|| {
        std::path::Path::new(&source_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string())
    });

    let target_path_buf = safe_sync_target_path(&target_dir, &skill_name, "skill", "skill name")?;
    let target_path = target_path_buf.to_string_lossy().to_string();

    // Ensure parent directory exists
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;

    // Perform sync
    crate::skills::sync_engine::sync_dir_for_tool_with_overwrite(
        &tool,
        &source_path,
        &target_path_buf,
        overwrite,
        adapter.force_copy,
    )
    .map_err(|e| {
        log::warn!(
            "[SYNC_ERROR] sync_skill_to_tool failed | skill_id={} tool={} scope={}",
            skill_id,
            tool,
            scope
        );
        AppError::FileSystemError(e)
    })?;

    // Record target in database
    let now = now_ms();
    let target = SkillTarget {
        id: uuid::Uuid::new_v4().to_string(),
        skill_id: skill_id.clone(),
        tool: tool.clone(),
        scope,
        project_path,
        target_path,
        mode: if adapter.force_copy {
            "copy".to_string()
        } else {
            "auto".to_string()
        },
        status: "ok".to_string(),
        synced_at: Some(now),
        ..Default::default()
    };

    let targets_repo = SkillTargetsRepository::new(&state.db);
    targets_repo.upsert(&target).map_err(|e| {
        log::warn!(
            "[DB_ERROR] sync_skill_to_tool: upsert target failed | skill_id={} tool={}",
            skill_id,
            tool
        );
        AppError::DatabaseError(e.to_string())
    })?;

    // Update skill last_sync_at
    state
        .db
        .with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET last_sync_at = ?1, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, skill_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| {
            log::warn!(
                "[DB_ERROR] sync_skill_to_tool: update last_sync_at failed | skill_id={}",
                skill_id
            );
            AppError::DatabaseError(e.to_string())
        })?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn unsync_skill_from_tool(
    state: State<'_, AppState>,
    skill_id: String,
    tool: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> AppResult<()> {
    let scope = scope.unwrap_or_else(|| "global".to_string());

    let targets_repo = SkillTargetsRepository::new(&state.db);
    let target = targets_repo
        .get(&skill_id, &tool, &scope, project_path.as_deref())
        .map_err(|e| {
            log::warn!(
                "[DB_ERROR] unsync_skill_from_tool: get target failed | skill_id={} tool={}",
                skill_id,
                tool
            );
            AppError::DatabaseError(e.to_string())
        })?;

    if let Some(t) = target {
        // Remove the synced path
        let _ = crate::skills::sync_engine::unsync_target(&t.target_path);
        // Delete from DB
        targets_repo
            .delete(&skill_id, &tool, &scope, project_path.as_deref())
            .map_err(|e| {
                log::warn!(
                    "[DB_ERROR] unsync_skill_from_tool: delete target failed | skill_id={} tool={}",
                    skill_id,
                    tool
                );
                AppError::DatabaseError(e.to_string())
            })?;
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn sync_suite_to_tool(
    state: State<'_, AppState>,
    source_path: String,
    skill_id: String,
    tool: String,
    name: Option<String>,
    scope: Option<String>,
    project_path: Option<String>,
    overwrite_if_same_content: Option<bool>,
) -> AppResult<()> {
    // Suite sync: sync each sub-skill directory
    let scope = scope.unwrap_or_else(|| "global".to_string());
    let overwrite = overwrite_if_same_content.unwrap_or(true);

    let adapters = effective_tool_adapters(&state.db);
    let adapter = adapter::adapter_by_key(&adapters, &tool)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown tool: {}", tool)))?;

    let target_base = if scope == "project" {
        let pp = project_path.as_deref().ok_or_else(|| {
            AppError::InvalidInput("project_path required for project scope".into())
        })?;
        resolve_project_path(adapter, pp)
    } else {
        resolve_default_path(adapter)
    };

    let suite_name = name.unwrap_or_else(|| {
        std::path::Path::new(&source_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "suite".to_string())
    });

    // Scan sub-directories of the suite
    let source_dir = std::path::Path::new(&source_path);
    if !source_dir.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "source is not a directory: {}",
            source_path
        )));
    }

    let entries = std::fs::read_dir(source_dir)
        .map_err(|e| AppError::FileSystemError(format!("failed to read suite dir: {}", e)))?;

    let now = now_ms();
    let targets_repo = SkillTargetsRepository::new(&state.db);

    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let sub_path = entry.path();
        if !sub_path.join("SKILL.md").exists() {
            continue;
        }

        let sub_name = entry.file_name().to_string_lossy().to_string();
        let target_path_buf =
            safe_sync_target_path(&target_base, &sub_name, "skill", "suite sub-skill name")?;
        let target_path_str = target_path_buf.to_string_lossy().to_string();

        std::fs::create_dir_all(&target_base)
            .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;

        crate::skills::sync_engine::sync_dir_for_tool_with_overwrite(
            &tool,
            &sub_path,
            &target_path_buf,
            overwrite,
            adapter.force_copy,
        )
        .map_err(|e| {
            log::warn!("[SYNC_ERROR] sync_suite_to_tool: sub-skill sync failed | skill_id={} tool={} sub={}", skill_id, tool, sub_name);
            AppError::FileSystemError(e)
        })?;

        let target = SkillTarget {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id: skill_id.clone(),
            tool: tool.clone(),
            scope: scope.clone(),
            project_path: project_path.clone(),
            target_path: target_path_str,
            mode: if adapter.force_copy {
                "copy".to_string()
            } else {
                "auto".to_string()
            },
            status: "ok".to_string(),
            synced_at: Some(now),
            suite_skill_id: Some(skill_id.clone()),
            ..Default::default()
        };
        targets_repo.upsert(&target).map_err(|e| {
            log::warn!(
                "[DB_ERROR] sync_suite_to_tool: upsert sub-target failed | skill_id={} tool={}",
                skill_id,
                tool
            );
            AppError::DatabaseError(e.to_string())
        })?;
    }

    // Also record the suite-level target
    let suite_target_path =
        safe_sync_target_path(&target_base, &suite_name, "suite", "suite name")?;
    let suite_target = SkillTarget {
        id: uuid::Uuid::new_v4().to_string(),
        skill_id: skill_id.clone(),
        tool: tool.clone(),
        scope,
        project_path,
        target_path: suite_target_path.to_string_lossy().to_string(),
        mode: "suite".to_string(),
        status: "ok".to_string(),
        synced_at: Some(now),
        ..Default::default()
    };
    targets_repo
        .upsert(&suite_target)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    state
        .db
        .with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET last_sync_at = ?1, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![now, skill_id],
            )?;
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn unsync_suite_from_tool(
    state: State<'_, AppState>,
    skill_id: String,
    tool: String,
    scope: Option<String>,
    project_path: Option<String>,
) -> AppResult<()> {
    let scope = scope.unwrap_or_else(|| "global".to_string());

    let targets_repo = SkillTargetsRepository::new(&state.db);
    let deleted = targets_repo
        .delete_suite_targets(&skill_id, &tool, &scope, project_path.as_deref())
        .map_err(|e| {
            log::warn!("[DB_ERROR] unsync_suite_from_tool: delete suite targets failed | skill_id={} tool={}", skill_id, tool);
            AppError::DatabaseError(e.to_string())
        })?;

    for t in &deleted {
        let _ = crate::skills::sync_engine::unsync_target(&t.target_path);
    }

    // Also delete the suite-level target
    let _ = targets_repo.delete(&skill_id, &tool, &scope, project_path.as_deref());

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_scope_preferences(state: State<'_, AppState>) -> AppResult<Vec<ScopePreference>> {
    let repo = ScopePreferencesRepository::new(&state.db);
    repo.list_all()
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_scope_preference(
    state: State<'_, AppState>,
    skill_id: String,
    scope: String,
    project_paths: String,
) -> AppResult<()> {
    let repo = ScopePreferencesRepository::new(&state.db);
    repo.set(&skill_id, &scope, &project_paths)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_recent_projects(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let repo = RecentProjectsRepository::new(&state.db);
    repo.list(8)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn save_recent_project(
    state: State<'_, AppState>,
    project_path: String,
) -> AppResult<Vec<String>> {
    let repo = RecentProjectsRepository::new(&state.db);
    repo.touch(&project_path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    repo.list(8)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_suite_sub_skills(
    state: State<'_, AppState>,
    skill_id: String,
) -> AppResult<Vec<serde_json::Value>> {
    let skills_repo = SkillsRepository::new(&state.db);
    let skill = skills_repo
        .get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))?;

    let community_path = &skill.community_path;
    let source_dir = std::path::Path::new(community_path);

    if !source_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut subs = Vec::new();
    let entries = std::fs::read_dir(source_dir)
        .map_err(|e| AppError::FileSystemError(format!("failed to read dir: {}", e)))?;

    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let sub_path = entry.path();
        if !sub_path.join("SKILL.md").exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        subs.push(serde_json::json!({
            "name": name,
            "path": sub_path.to_string_lossy().to_string(),
        }));
    }

    Ok(subs)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn bulk_sync_skills(
    state: State<'_, AppState>,
    skill_ids: Vec<String>,
) -> AppResult<serde_json::Value> {
    let skills_repo = SkillsRepository::new(&state.db);
    let targets_repo = SkillTargetsRepository::new(&state.db);
    let adapters = effective_tool_adapters(&state.db);

    let mut synced = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for skill_id in &skill_ids {
        let skill = match skills_repo
            .get_by_id(skill_id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
        {
            Some(skill) => skill,
            None => {
                skipped += 1;
                continue;
            }
        };

        if !skill.enabled {
            skipped += 1;
            continue;
        }

        let targets = targets_repo
            .list_by_skill(skill_id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for target in &targets {
            let Some(adapter) = adapter::adapter_by_key(&adapters, &target.tool) else {
                continue;
            };
            let target_path = std::path::PathBuf::from(&target.target_path);
            match crate::skills::sync_engine::sync_dir_for_tool_with_overwrite(
                &target.tool,
                &skill.community_path,
                &target_path,
                true,
                adapter.force_copy,
            ) {
                Ok(_) => synced += 1,
                Err(e) => errors.push(format!("{}: {}", skill.name, e)),
            }
        }
    }

    Ok(serde_json::json!({
        "synced": synced,
        "skipped": skipped,
        "errors": errors,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_sync_target_path_sanitizes_windows_invalid_name_chars() {
        let base = std::env::temp_dir().join("skills_hub_sync_target_path_test");
        let base_str = base.to_string_lossy().to_string();

        let target = safe_sync_target_path(
            &base_str,
            "tasteskill: Anti-Slop/Frontend Skill",
            "skill",
            "skill name",
        )
        .unwrap();

        assert_eq!(
            target.file_name().unwrap().to_string_lossy(),
            "tasteskill- Anti-Slop-Frontend Skill"
        );
        assert!(path_safety::is_path_within(&target, &base));
    }

    #[test]
    fn safe_sync_target_path_uses_custom_fallback() {
        let base = std::env::temp_dir().join("skills_hub_sync_suite_path_test");
        let base_str = base.to_string_lossy().to_string();

        let target = safe_sync_target_path(&base_str, "...", "suite", "suite name").unwrap();

        assert_eq!(target.file_name().unwrap().to_string_lossy(), "suite");
        assert!(path_safety::is_path_within(&target, &base));
    }
}
