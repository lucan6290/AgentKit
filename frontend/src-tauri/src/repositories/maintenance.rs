use crate::db::Database;
use crate::error::{AppError, AppResult};

pub struct MaintenanceRepository<'a> {
    db: &'a Database,
}

impl<'a> MaintenanceRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn vacuum(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("VACUUM")?;
            Ok(())
        })
    }

    pub fn analyze(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("ANALYZE")?;
            Ok(())
        })
    }

    pub fn clear_cache(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("DELETE FROM tool_skill_cache; DELETE FROM tool_scan_state;")?;
            Ok(())
        })
    }

    pub fn clear_discovered(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("DELETE FROM discovered_skills;")?;
            Ok(())
        })
    }

    pub fn integrity_check(&self) -> AppResult<Vec<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare("PRAGMA integrity_check")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn wal_checkpoint(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            Ok(())
        })
    }

    pub fn reindex(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("REINDEX")?;
            Ok(())
        })
    }

    pub fn optimize(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("PRAGMA optimize")?;
            Ok(())
        })
    }

    pub fn clear_usage(&self) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute_batch("DELETE FROM skill_usage;")?;
            Ok(())
        })
    }

    pub fn get_table_names(&self) -> AppResult<Vec<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn table_exists(&self, table: &str) -> AppResult<bool> {
        let count = self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get::<_, i64>(0),
            )
        })?;
        Ok(count > 0)
    }

    pub fn get_table_count(&self, table: &str) -> AppResult<i64> {
        if !self.table_exists(table)? {
            return Err(AppError::Unexpected(format!(
                "Invalid table name: {}",
                table
            )));
        }

        self.db.with_conn(|conn| {
            let sql = format!("SELECT COUNT(*) FROM {}", table);
            conn.query_row(&sql, [], |row| row.get(0))
                .map_err(|e| e.into())
        })
    }

    pub fn get_database_stats(&self) -> AppResult<(String, i64, i64, i64)> {
        self.db.with_conn(|conn| {
            let version = conn
                .query_row("SELECT sqlite_version()", [], |row| row.get(0))
                .unwrap_or_default();
            let page_size = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .unwrap_or(4096);
            let page_count = conn
                .query_row("PRAGMA page_count", [], |row| row.get(0))
                .unwrap_or(0);
            let freelist_count = conn
                .query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .unwrap_or(0);
            Ok((version, page_size, page_count, freelist_count))
        })
    }

    pub fn reset(&self) -> AppResult<()> {
        const RESET_DELETE_ORDER: &[&str] = &[
            "skill_tag_links",
            "skill_targets",
            "skill_usage",
            "skill_scope_preference",
            "tool_skill_cache",
            "tool_scan_state",
            "discovered_skills",
            "recent_projects",
            "skill_tags",
            "skills",
            "settings",
            "tool_adapter_configs",
        ];

        self.db.with_conn(|conn| {
            for table in RESET_DELETE_ORDER {
                conn.execute_batch(&format!("DELETE FROM {}", table))?;
            }
            conn.execute_batch("VACUUM")?;

            let mut order = 1.0;
            let now = crate::db::now_ms();
            for (key, cfg) in crate::config::default_tool_adapters() {
                conn.execute(
                    "INSERT OR REPLACE INTO tool_adapter_configs
                     (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                      supports_symlink, supports_junction, force_copy, supports_project_scope,
                      is_custom, enabled, sort_order, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 1, ?10, ?11)",
                    rusqlite::params![
                        key,
                        cfg.display_name,
                        cfg.skills_dir,
                        cfg.detect_dir,
                        cfg.project_skills_dir,
                        cfg.supports_symlink as i32,
                        cfg.supports_junction as i32,
                        cfg.force_copy as i32,
                        cfg.supports_project_scope.map(|value| value as i32),
                        order,
                        now,
                    ],
                )?;
                order += 1.0;
            }
            Ok(())
        })
    }

    pub fn get_table_data(
        &self,
        table_name: &str,
        page: i64,
        page_size: i64,
        sort_col: Option<&str>,
        sort_dir: Option<&str>,
        filter_text: Option<&str>,
    ) -> AppResult<crate::contracts::DbTableData> {
        use crate::contracts::{DbColumnInfo, DbTableData};

        if !self.table_exists(table_name)? {
            return Err(AppError::InvalidInput(format!(
                "Invalid table name: {}",
                table_name
            )));
        }

        let columns = self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
            let rows = stmt.query_map([], |row| {
                Ok(DbColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    col_type: row.get(2)?,
                    notnull: row.get::<_, i32>(3)? != 0,
                    default: row.get(4)?,
                    pk: row.get::<_, i32>(5)? != 0,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })?;
        let (where_clause, params) = match filter_text.filter(|filter| !filter.is_empty()) {
            Some(filter) => {
                let text_cols: Vec<_> = columns
                    .iter()
                    .filter(|column| {
                        column.col_type.to_uppercase().contains("TEXT")
                            || column.col_type.is_empty()
                    })
                    .map(|column| column.name.as_str())
                    .collect();
                if text_cols.is_empty() {
                    (String::new(), vec![])
                } else {
                    (
                        format!(
                            " WHERE {}",
                            text_cols
                                .iter()
                                .map(|column| format!("{} LIKE ?1", column))
                                .collect::<Vec<_>>()
                                .join(" OR ")
                        ),
                        vec![format!("%{}%", filter)],
                    )
                }
            }
            None => (String::new(), vec![]),
        };
        let total = self.db.with_conn(|conn| {
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
                .iter()
                .map(|value| value as &dyn rusqlite::types::ToSql)
                .collect();
            conn.query_row(
                &format!("SELECT COUNT(*) FROM {}{}", table_name, where_clause),
                params_refs.as_slice(),
                |row| row.get(0),
            )
        })?;
        let order_by = sort_col
            .filter(|column| columns.iter().any(|item| item.name == *column))
            .map(|column| {
                format!(
                    " ORDER BY {} {}",
                    column,
                    if sort_dir == Some("desc") {
                        "DESC"
                    } else {
                        "ASC"
                    }
                )
            })
            .unwrap_or_default();
        let offset = (page - 1) * page_size;
        let rows = self.db.with_conn(|conn| {
            let sql = format!(
                "SELECT * FROM {}{}{} LIMIT ?{} OFFSET ?{}",
                table_name,
                where_clause,
                order_by,
                params.len() + 1,
                params.len() + 2
            );
            let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = params
                .iter()
                .map(|value| Box::new(value.clone()) as Box<dyn rusqlite::types::ToSql>)
                .collect();
            values.push(Box::new(page_size));
            values.push(Box::new(offset));
            let refs: Vec<&dyn rusqlite::types::ToSql> =
                values.iter().map(|value| value.as_ref()).collect();
            let names: Vec<_> = columns.iter().map(|column| column.name.clone()).collect();
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt
                .query_map(refs.as_slice(), |row| {
                    let mut map = serde_json::Map::new();
                    for (index, name) in names.iter().enumerate() {
                        let value = match row.get::<_, rusqlite::types::Value>(index) {
                            Ok(rusqlite::types::Value::Null) => serde_json::Value::Null,
                            Ok(rusqlite::types::Value::Integer(value)) => {
                                serde_json::Value::Number(value.into())
                            }
                            Ok(rusqlite::types::Value::Real(value)) => {
                                serde_json::Number::from_f64(value)
                                    .map(serde_json::Value::Number)
                                    .unwrap_or(serde_json::Value::Null)
                            }
                            Ok(rusqlite::types::Value::Text(value)) => {
                                serde_json::Value::String(value)
                            }
                            Ok(rusqlite::types::Value::Blob(value)) => {
                                serde_json::Value::String(format!("<blob {} bytes>", value.len()))
                            }
                            Err(_) => serde_json::Value::Null,
                        };
                        map.insert(name.clone(), value);
                    }
                    Ok(serde_json::Value::Object(map))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        Ok(DbTableData {
            table: table_name.to_string(),
            display_name: table_name.to_string(),
            columns,
            rows,
            total,
            page,
            page_size,
            total_pages: (total as f64 / page_size as f64).ceil() as i64,
        })
    }

    pub fn get_overview(&self) -> AppResult<DatabaseOverview> {
        let table_names = self.get_table_names()?;

        let mut counts = Vec::new();
        for name in &table_names {
            let count = self.get_table_count(name)?;
            counts.push((name.clone(), count));
        }

        Ok(DatabaseOverview { tables: counts })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseOverview {
    pub tables: Vec<(String, i64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maintenance_operations() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        repo.analyze().unwrap();
        repo.vacuum().unwrap();
        repo.wal_checkpoint().unwrap();
        repo.reindex().unwrap();
        repo.optimize().unwrap();
        repo.clear_usage().unwrap();

        let results = repo.integrity_check().unwrap();
        assert_eq!(results, vec!["ok"]);
    }

    #[test]
    fn test_get_overview() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        let overview = repo.get_overview().unwrap();
        assert_eq!(overview.tables.len(), 14);
    }

    #[test]
    fn test_invalid_table_name() {
        let db = Database::new_in_memory().unwrap();
        let repo = MaintenanceRepository::new(&db);

        let result = repo.get_table_count("invalid_table");
        assert!(result.is_err());
    }
}
