use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::Prompt;

pub struct PromptsRepository<'a> {
    db: &'a Database,
}

impl<'a> PromptsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<Prompt>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, content, created_at, updated_at FROM prompts ORDER BY updated_at DESC, name",
            )?;
            let rows = stmt.query_map([], map_prompt)?;
            rows.collect()
        })
    }

    pub fn get(&self, id: &str) -> AppResult<Option<Prompt>> {
        self.db.with_conn(|conn| {
            let result = conn.query_row(
                "SELECT id, name, content, created_at, updated_at FROM prompts WHERE id = ?1",
                [id],
                map_prompt,
            );
            match result {
                Ok(prompt) => Ok(Some(prompt)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(error) => Err(error),
            }
        })
    }

    pub fn create(&self, prompt: &Prompt) -> AppResult<()> {
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "INSERT INTO prompts (id, name, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![prompt.id, prompt.name, prompt.content, prompt.created_at, prompt.updated_at],
            )?;
            Ok(())
        })
    }

    pub fn update(&self, id: &str, name: &str, content: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "UPDATE prompts SET name = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![name, content, now, id],
            )?;
            Ok(())
        })
    }

    pub fn update_content(&self, id: &str, content: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn_mut(|conn| {
            conn.execute(
                "UPDATE prompts SET content = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![content, now, id],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.db.with_conn_mut(|conn| {
            conn.execute("DELETE FROM prompts WHERE id = ?1", [id])?;
            Ok(())
        })
    }
}

fn map_prompt(row: &rusqlite::Row<'_>) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: row.get(0)?,
        name: row.get(1)?,
        content: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}
