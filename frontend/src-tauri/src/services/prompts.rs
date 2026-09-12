use std::path::Path;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::contracts::PromptDto;
use crate::db::{now_ms, Database};
use crate::error::{AppError, AppResult};
use crate::models::{Prompt, PromptFileLink};
use crate::repositories::{PromptFileLinksRepository, PromptsRepository};

pub fn list_prompts(db: &Database) -> AppResult<Vec<PromptDto>> {
    let prompts = PromptsRepository::new(db).list()?;
    prompts
        .into_iter()
        .map(|prompt| to_dto(db, prompt))
        .collect()
}

pub fn create_prompt(db: &Database, name: String, content: String) -> AppResult<PromptDto> {
    let now = now_ms();
    let prompt = Prompt {
        id: Uuid::new_v4().to_string(),
        name,
        content,
        created_at: now,
        updated_at: now,
    };
    PromptsRepository::new(db).create(&prompt)?;
    to_dto(db, prompt)
}

pub fn update_prompt(
    db: &Database,
    id: &str,
    name: String,
    content: String,
) -> AppResult<PromptDto> {
    let repo = PromptsRepository::new(db);
    require_prompt(&repo, id)?;
    repo.update(id, &name, &content)?;
    get_prompt_dto(db, id)
}

pub fn duplicate_prompt(db: &Database, id: &str, name: Option<String>) -> AppResult<PromptDto> {
    let source = require_prompt(&PromptsRepository::new(db), id)?;
    create_prompt(
        db,
        name.unwrap_or_else(|| format!("{} Copy", source.name)),
        source.content,
    )
}

pub fn delete_prompt(db: &Database, id: &str) -> AppResult<()> {
    let repo = PromptsRepository::new(db);
    require_prompt(&repo, id)?;
    repo.delete(id)
}

pub fn import_prompt_file(
    db: &Database,
    file_path: String,
    name: Option<String>,
) -> AppResult<PromptDto> {
    let path = Path::new(&file_path);
    let content = crate::filesystem::read_file(path).map_err(AppError::FileSystemError)?;
    let prompt_name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Untitled prompt")
            .to_string()
    });
    let prompt = create_prompt(db, prompt_name, content)?;
    let prompt_id = prompt.prompt.id.clone();
    create_prompt_file_link(db, &prompt_id, file_path, Some(false))?;
    get_prompt_dto(db, &prompt_id)
}

pub fn create_prompt_file_link(
    db: &Database,
    prompt_id: &str,
    file_path: String,
    write_back_enabled: Option<bool>,
) -> AppResult<PromptFileLink> {
    require_prompt(&PromptsRepository::new(db), prompt_id)?;
    let path = Path::new(&file_path);
    let exists_on_disk = crate::filesystem::exists(path);
    let content_hash = if exists_on_disk {
        Some(hash_file(path)?)
    } else {
        None
    };
    let now = now_ms();
    let link = PromptFileLink {
        id: Uuid::new_v4().to_string(),
        prompt_id: prompt_id.to_string(),
        file_path,
        write_back_enabled: write_back_enabled.unwrap_or(false),
        content_hash,
        exists_on_disk,
        last_synced_at: None,
        created_at: now,
        updated_at: now,
    };
    PromptFileLinksRepository::new(db).create(&link)?;
    Ok(link)
}

pub fn unlink_prompt_file(db: &Database, link_id: &str) -> AppResult<()> {
    let repo = PromptFileLinksRepository::new(db);
    require_link(&repo, link_id)?;
    repo.delete(link_id)
}

pub fn refresh_prompt_file_link(db: &Database, link_id: &str) -> AppResult<PromptDto> {
    let links = PromptFileLinksRepository::new(db);
    let link = require_link(&links, link_id)?;
    let path = Path::new(&link.file_path);
    if !crate::filesystem::exists(path) {
        links.update_sync_state(link_id, None, false, None)?;
        return Err(AppError::NotFound(format!(
            "prompt file not found: {}",
            link.file_path
        )));
    }

    let content = crate::filesystem::read_file(path).map_err(AppError::FileSystemError)?;
    let hash = hash_content(&content);
    let prompts = PromptsRepository::new(db);
    require_prompt(&prompts, &link.prompt_id)?;
    prompts.update_content(&link.prompt_id, &content)?;
    links.update_sync_state(link_id, Some(&hash), true, Some(now_ms()))?;
    get_prompt_dto(db, &link.prompt_id)
}

pub fn write_prompt_to_file(
    db: &Database,
    link_id: &str,
    force: Option<bool>,
) -> AppResult<PromptFileLink> {
    let links = PromptFileLinksRepository::new(db);
    let link = require_link(&links, link_id)?;
    if !link.write_back_enabled {
        return Err(AppError::InvalidInput(format!(
            "write-back is disabled for prompt file link: {}",
            link_id
        )));
    }

    let prompt = require_prompt(&PromptsRepository::new(db), &link.prompt_id)?;
    let path = Path::new(&link.file_path);
    let exists_on_disk = crate::filesystem::exists(path);
    if exists_on_disk && !force.unwrap_or(false) {
        let current_hash = hash_file(path)?;
        if link.content_hash.as_deref() != Some(current_hash.as_str()) {
            return Err(AppError::InvalidInput(format!(
                "prompt file conflict: {}",
                link.file_path
            )));
        }
    }

    crate::filesystem::write_file(path, prompt.content.as_bytes())
        .map_err(AppError::FileSystemError)?;
    let hash = hash_content(&prompt.content);
    let now = now_ms();
    links.update_sync_state(link_id, Some(&hash), true, Some(now))?;
    require_link(&links, link_id)
}

fn get_prompt_dto(db: &Database, id: &str) -> AppResult<PromptDto> {
    let prompt = require_prompt(&PromptsRepository::new(db), id)?;
    to_dto(db, prompt)
}

fn to_dto(db: &Database, prompt: Prompt) -> AppResult<PromptDto> {
    Ok(PromptDto {
        file_links: PromptFileLinksRepository::new(db).list_by_prompt(&prompt.id)?,
        prompt,
    })
}

fn require_prompt(repo: &PromptsRepository<'_>, id: &str) -> AppResult<Prompt> {
    repo.get(id)?
        .ok_or_else(|| AppError::NotFound(format!("prompt not found: {}", id)))
}

fn require_link(repo: &PromptFileLinksRepository<'_>, id: &str) -> AppResult<PromptFileLink> {
    repo.get(id)?
        .ok_or_else(|| AppError::NotFound(format!("prompt file link not found: {}", id)))
}

fn hash_file(path: &Path) -> AppResult<String> {
    let content = crate::filesystem::read_bytes(path).map_err(AppError::FileSystemError)?;
    Ok(hash_bytes(&content))
}

fn hash_content(content: &str) -> String {
    hash_bytes(content.as_bytes())
}

fn hash_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_dto_includes_an_empty_file_links_array() {
        let db = Database::new_in_memory().unwrap();
        let prompt = create_prompt(&db, "prompt".to_string(), "content".to_string()).unwrap();

        assert!(prompt.file_links.is_empty());
        assert!(list_prompts(&db).unwrap()[0].file_links.is_empty());
    }

    #[test]
    fn imported_prompt_dto_includes_its_file_link() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prompt.md");
        crate::filesystem::write_file(&path, b"from file").unwrap();

        let prompt = import_prompt_file(&db, path.to_string_lossy().to_string(), None).unwrap();

        assert_eq!(prompt.file_links.len(), 1);
        assert_eq!(prompt.file_links[0].file_path, path.to_string_lossy());
    }

    #[test]
    fn prompt_edits_stay_in_database_until_write_back() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prompt.md");
        crate::filesystem::write_file(&path, b"from file").unwrap();

        let prompt = import_prompt_file(&db, path.to_string_lossy().to_string(), None).unwrap();
        let link = create_prompt_file_link(
            &db,
            &prompt.prompt.id,
            dir.path()
                .join("write-back.md")
                .to_string_lossy()
                .to_string(),
            Some(true),
        )
        .unwrap();
        let updated = update_prompt(
            &db,
            &prompt.prompt.id,
            "renamed".to_string(),
            "database only".to_string(),
        )
        .unwrap();

        assert_eq!(crate::filesystem::read_file(&path).unwrap(), "from file");
        write_prompt_to_file(&db, &link.id, None).unwrap();
        assert_eq!(
            crate::filesystem::read_file(dir.path().join("write-back.md")).unwrap(),
            updated.prompt.content
        );
    }

    #[test]
    fn write_back_detects_external_file_changes() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prompt.md");
        crate::filesystem::write_file(&path, b"original").unwrap();
        let prompt = create_prompt(&db, "prompt".to_string(), "database".to_string()).unwrap();
        let link = create_prompt_file_link(
            &db,
            &prompt.prompt.id,
            path.to_string_lossy().to_string(),
            Some(true),
        )
        .unwrap();
        crate::filesystem::write_file(&path, b"external").unwrap();

        assert!(write_prompt_to_file(&db, &link.id, None).is_err());
        write_prompt_to_file(&db, &link.id, Some(true)).unwrap();
        assert_eq!(crate::filesystem::read_file(&path).unwrap(), "database");
    }
}
