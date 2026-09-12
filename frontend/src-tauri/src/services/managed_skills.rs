use crate::contracts::ManagedSkillDto;
use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::repositories::{
    SkillTargetsRepository, SkillUsageRepository, SkillsRepository, TagsRepository,
};
use crate::tools::adapter::{self, effective_tool_adapters};

pub fn get_managed_skills(
    db: &Database,
    refresh: bool,
    source_type: Option<&str>,
    sort: &str,
) -> AppResult<Vec<ManagedSkillDto>> {
    if refresh {
        crate::repo::scanner::sync_all_repo_registries(db).map_err(AppError::FileSystemError)?;
    }

    let skills_repo = SkillsRepository::new(db);
    let tags_repo = TagsRepository::new(db);
    let targets_repo = SkillTargetsRepository::new(db);
    let usage_repo = SkillUsageRepository::new(db);
    let skills = skills_repo.list(sort)?;

    skills
        .into_iter()
        .filter(|skill| match source_type {
            Some("custom") => {
                crate::repo::scanner::normalize_source_type(&skill.source_type) == "custom"
            }
            Some(_) => {
                crate::repo::scanner::normalize_source_type(&skill.source_type) == "community"
            }
            None => true,
        })
        .map(|skill| {
            Ok(ManagedSkillDto {
                tags: tags_repo.get_skill_tags(&skill.id)?,
                targets: targets_repo.list_by_skill(&skill.id)?,
                usage: usage_repo.get_by_skill(&skill.id)?,
                is_suite: crate::repo::scanner::has_sub_skills(std::path::Path::new(
                    &skill.community_path,
                )),
                skill,
            })
        })
        .collect()
}

pub fn get_managed_skill_by_id(db: &Database, skill_id: &str) -> AppResult<Option<ManagedSkillDto>> {
    let skills_repo = SkillsRepository::new(db);
    let tags_repo = TagsRepository::new(db);
    let targets_repo = SkillTargetsRepository::new(db);
    let usage_repo = SkillUsageRepository::new(db);

    let Some(skill) = skills_repo.get_by_id(skill_id)? else {
        return Ok(None);
    };

    Ok(Some(ManagedSkillDto {
        tags: tags_repo.get_skill_tags(&skill.id)?,
        targets: targets_repo.list_by_skill(&skill.id)?,
        usage: usage_repo.get_by_skill(&skill.id)?,
        is_suite: crate::repo::scanner::has_sub_skills(std::path::Path::new(
            &skill.community_path,
        )),
        skill,
    }))
}

pub fn bulk_sync_skills(db: &Database, skill_ids: &[String]) -> AppResult<serde_json::Value> {
    let skills_repo = SkillsRepository::new(db);
    let targets_repo = SkillTargetsRepository::new(db);
    let adapters = effective_tool_adapters(db);
    let started = std::time::Instant::now();
    tracing::info!(target: crate::logging::app_target(), event = "sync.bulk.started", layer = "backend", area = "sync", outcome = "started", requested_skill_count = skill_ids.len(), "bulk sync started");

    let mut synced = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();

    for skill_id in skill_ids {
        let skill = match skills_repo.get_by_id(skill_id)? {
            Some(skill) => skill,
            None => {
                tracing::warn!(target: crate::logging::app_target(), event = "sync.bulk.skill.skipped", layer = "backend", area = "sync", outcome = "skipped", skill_id = %skill_id, reason = "missing", "skipping missing skill during bulk sync");
                skipped += 1;
                continue;
            }
        };

        if !skill.enabled {
            tracing::warn!(target: crate::logging::app_target(), event = "sync.bulk.skill.skipped", layer = "backend", area = "sync", outcome = "skipped", skill_id = %skill_id, reason = "disabled", "skipping disabled skill during bulk sync");
            skipped += 1;
            continue;
        }

        for target in targets_repo.list_by_skill(skill_id)? {
            let Some(adapter) = adapter::adapter_by_key(&adapters, &target.tool) else {
                tracing::warn!(target: crate::logging::app_target(), event = "sync.bulk.target.skipped", layer = "backend", area = "sync", outcome = "skipped", skill_id = %skill_id, tool = %target.tool, reason = "unknown_tool", "skipping target with unknown tool during bulk sync");
                skipped += 1;
                continue;
            };

            match crate::skills::sync_engine::sync_dir_for_tool_with_overwrite(
                &target.tool,
                &skill.community_path,
                std::path::Path::new(&target.target_path),
                true,
                adapter.force_copy,
            ) {
                Ok(_) => synced += 1,
                Err(error) => {
                    tracing::warn!(target: crate::logging::app_target(), event = "sync.bulk.item.failed", layer = "backend", area = "sync", outcome = "failed", skill_id = %skill_id, tool = %target.tool, target_path = %target.target_path, error = %error, "bulk sync item failed");
                    errors.push(format!("{}: {}", skill.name, error));
                }
            }
        }
    }

    tracing::info!(target: crate::logging::app_target(), event = "sync.bulk.completed", layer = "backend", area = "sync", outcome = "success", synced, skipped, error_count = errors.len(), duration_ms = started.elapsed().as_millis() as u64, "bulk sync completed");
    Ok(serde_json::json!({ "synced": synced, "skipped": skipped, "errors": errors }))
}
