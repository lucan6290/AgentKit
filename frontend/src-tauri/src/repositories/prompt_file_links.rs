use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::PromptFileLink;

pub struct PromptFileLinksRepository<'a> {
    db: &'a Database,
}

impl<'a> PromptFileLinksRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list_by_prompt(&self, prompt_id: &str) -> AppResult<Vec<PromptFileLink>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, prompt_id, file_path, write_back_enabled, content_hash, exists_on_disk, last_synced_at, created_at, updated_at FROM prompt_file_links WHERE prompt_id = ?1 ORDER BY created_at, id",
            )?;
            let rows = stmt.query_map([prompt_id], map_link)?;
            rows.collect()
        })
    }

    pub fn get(&self, id: &str) -> AppResult<Option<PromptFileLink>> {
        self.db.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT id, prompt_id, file_path, write_back_enabled, content_hash, exists_on_disk, last_synced_at, created_at, updated_at FROM prompt_file_links WHERE id = ?1",
                [id],
                map_link,
            );
            match result {
                Ok(link) => Ok(Some(link)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(error) => Err(error),
            }
        })
    }

    pub fn get_by_file_path(&self, file_path: &str) -> AppResult<Option<PromptFileLink>> {
        self.db.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT id, prompt_id, file_path, write_back_enabled, content_hash, exists_on_disk, last_synced_at, created_at, updated_at FROM prompt_file_links WHERE file_path = ?1",
                [file_path],
                map_link,
            );
            match result {
                Ok(link) => Ok(Some(link)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(error) => Err(error),
            }
        })
    }

    pub fn create(&self, link: &PromptFileLink) -> AppResult<()> {
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO prompt_file_links (id, prompt_id, file_path, write_back_enabled, content_hash, exists_on_disk, last_synced_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![link.id, link.prompt_id, link.file_path, link.write_back_enabled as i64, link.content_hash, link.exists_on_disk as i64, link.last_synced_at, link.created_at, link.updated_at],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.db.with_conn_mut(|conn| {
            conn.execute("DELETE FROM prompt_file_links WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    pub fn update_sync_state(
        &self,
        id: &str,
        content_hash: Option<&str>,
        exists_on_disk: bool,
        last_synced_at: Option<i64>,
    ) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "UPDATE prompt_file_links SET content_hash = ?1, exists_on_disk = ?2, last_synced_at = ?3, updated_at = ?4 WHERE id = ?5",
                rusqlite::params![content_hash, exists_on_disk as i64, last_synced_at, now, id],
            )?;
            Ok(())
        })
    }
}

fn map_link(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptFileLink> {
    Ok(PromptFileLink {
        id: row.get(0)?,
        prompt_id: row.get(1)?,
        file_path: row.get(2)?,
        write_back_enabled: row.get::<_, i64>(3)? != 0,
        content_hash: row.get(4)?,
        exists_on_disk: row.get::<_, i64>(5)? != 0,
        last_synced_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}
