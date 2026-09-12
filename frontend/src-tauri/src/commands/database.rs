use tauri::State;

use crate::contracts::{DbMaintenanceResult, DbOverview, DbTableData, DbTableInfo, OkResponse};
use crate::error::{AppError, AppResult};
use crate::repositories::MaintenanceRepository;
use crate::state::AppState;

fn table_display_name(table: &str) -> &str {
    match table {
        "skills" => "Skills",
        "skill_targets" => "Sync Targets",
        "skill_tags" => "Tags",
        "skill_tag_links" => "Tag Links",
        "settings" => "Settings",
        "discovered_skills" => "Discovered Skills",
        "tool_scan_state" => "Tool Scan State",
        "tool_skill_cache" => "Tool Skill Cache",
        "tool_adapter_configs" => "Tool Adapter Configs",
        "skill_scope_preference" => "Scope Preferences",
        "recent_projects" => "Recent Projects",
        "skill_usage" => "Skill Usage",
        "prompts" => "Prompts",
        "prompt_file_links" => "Prompt File Links",
        _ => table,
    }
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_overview(state: State<'_, AppState>) -> AppResult<DbOverview> {
    let db_path = crate::config::default_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();

    let file_meta = crate::filesystem::metadata(&db_path).ok();
    let file_size = file_meta
        .as_ref()
        .map(|meta| meta.len() as i64)
        .unwrap_or(0);
    let last_modified = file_meta
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);

    let maint = MaintenanceRepository::new(&state.db);
    let (sqlite_version, page_size, page_count, freelist_count) = maint
        .get_database_stats()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let free_size = freelist_count * page_size;
    let fragmentation_pct = if page_count > 0 {
        (freelist_count as f64 / page_count as f64) * 100.0
    } else {
        0.0
    };

    let maint = MaintenanceRepository::new(&state.db);
    let overview = maint
        .get_overview()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let tables: Vec<DbTableInfo> = overview
        .tables
        .iter()
        .map(|(name, count)| DbTableInfo {
            table_name: name.clone(),
            display_name: table_display_name(name).to_string(),
            row_count: *count,
            size_bytes: 0, // Per-table size estimation is expensive; skip for now
            size_human: String::new(),
        })
        .collect();

    Ok(DbOverview {
        db_path: db_path_str,
        file_size,
        file_size_human: format_size(file_size),
        last_modified,
        sqlite_version,
        page_size,
        page_count,
        freelist_count,
        free_size,
        free_size_human: format_size(free_size),
        fragmentation_pct,
        tables,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_table_data(
    state: State<'_, AppState>,
    table_name: String,
    page: Option<i64>,
    page_size: Option<i64>,
    sort_col: Option<String>,
    sort_dir: Option<String>,
    filter_text: Option<String>,
) -> AppResult<DbTableData> {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).clamp(1, 200);
    let mut data = MaintenanceRepository::new(&state.db)
        .get_table_data(
            &table_name,
            page,
            page_size,
            sort_col.as_deref(),
            sort_dir.as_deref(),
            filter_text.as_deref(),
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    data.display_name = table_display_name(&table_name).to_string();
    Ok(data)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_maintenance(
    state: State<'_, AppState>,
    action: String,
) -> AppResult<DbMaintenanceResult> {
    let maint = MaintenanceRepository::new(&state.db);

    match action.as_str() {
        "vacuum" => {
            maint
                .vacuum()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "vacuum".to_string(),
                message: "VACUUM completed".to_string(),
                integrity_result: None,
            })
        }
        "analyze" => {
            maint
                .analyze()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "analyze".to_string(),
                message: "ANALYZE completed".to_string(),
                integrity_result: None,
            })
        }
        "integrity_check" => {
            let results = maint
                .integrity_check()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            let result_str = results.join(", ");
            Ok(DbMaintenanceResult {
                ok: result_str == "ok",
                action: "integrity_check".to_string(),
                message: result_str.clone(),
                integrity_result: Some(result_str),
            })
        }
        "clear_cache" => {
            maint
                .clear_cache()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_cache".to_string(),
                message: "Cache cleared".to_string(),
                integrity_result: None,
            })
        }
        "clear_discovered" => {
            maint
                .clear_discovered()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_discovered".to_string(),
                message: "Discovered skills cleared".to_string(),
                integrity_result: None,
            })
        }
        "wal_checkpoint" => {
            maint
                .wal_checkpoint()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "wal_checkpoint".to_string(),
                message: "WAL checkpoint completed".to_string(),
                integrity_result: None,
            })
        }
        "reindex" => {
            maint
                .reindex()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "reindex".to_string(),
                message: "REINDEX completed".to_string(),
                integrity_result: None,
            })
        }
        "optimize" => {
            maint
                .optimize()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "optimize".to_string(),
                message: "PRAGMA optimize completed".to_string(),
                integrity_result: None,
            })
        }
        "clear_usage" => {
            maint
                .clear_usage()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_usage".to_string(),
                message: "Usage records cleared".to_string(),
                integrity_result: None,
            })
        }
        _ => Err(AppError::InvalidInput(format!(
            "unknown maintenance action: {}",
            action
        ))),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_reset(state: State<'_, AppState>, confirm_text: String) -> AppResult<OkResponse> {
    if confirm_text != "RESET" && confirm_text != "reset" {
        return Err(AppError::InvalidInput(
            "confirmation text must be 'RESET'".into(),
        ));
    }

    MaintenanceRepository::new(&state.db)
        .reset()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(OkResponse {
        ok: true,
        message: "database has been reset".to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_export(app: tauri::AppHandle, state: State<'_, AppState>) -> AppResult<OkResponse> {
    use tauri_plugin_dialog::DialogExt;

    let started = std::time::Instant::now();
    tracing::info!(target: crate::logging::app_target(), event = "database.export.started", layer = "backend", area = "database", outcome = "started", "database export started");
    let maint = MaintenanceRepository::new(&state.db);
    maint.wal_checkpoint().map_err(|e| {
        tracing::warn!(target: crate::logging::app_target(), event = "database.export.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "WAL checkpoint failed before export");
        AppError::DatabaseError(e.to_string())
    })?;

    let db_path = crate::config::default_db_path();
    let default_name = format!("skills_hub_backup_{}.db", local_timestamp());

    let file_path = app
        .dialog()
        .file()
        .set_title("Export Database Backup")
        .set_file_name(&default_name)
        .add_filter("SQLite Database", &["db"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let dest = path.as_path().unwrap_or_else(|| std::path::Path::new(""));
            if dest.as_os_str().is_empty() {
                tracing::info!(target: crate::logging::app_target(), event = "database.export.completed", layer = "backend", area = "database", outcome = "canceled", duration_ms = started.elapsed().as_millis() as u64, "database export canceled");
                return Ok(OkResponse {
                    ok: false,
                    message: "No file selected".to_string(),
                });
            }
            // Ensure .db extension
            let dest = if dest.extension().is_none() {
                dest.with_extension("db")
            } else {
                dest.to_path_buf()
            };

            crate::filesystem::copy_file(&db_path, &dest).map_err(|e| {
                tracing::warn!(target: crate::logging::app_target(), event = "database.export.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to copy database export");
                AppError::FileSystemError(format!("Failed to copy database: {}", e))
            })?;

            tracing::info!(target: crate::logging::app_target(), event = "database.export.completed", layer = "backend", area = "database", outcome = "success", duration_ms = started.elapsed().as_millis() as u64, "database export completed");
            Ok(OkResponse {
                ok: true,
                message: format!("Database exported to: {}", dest.display()),
            })
        }
        None => {
            tracing::info!(target: crate::logging::app_target(), event = "database.export.completed", layer = "backend", area = "database", outcome = "canceled", duration_ms = started.elapsed().as_millis() as u64, "database export canceled");
            Ok(OkResponse {
                ok: false,
                message: "Export cancelled".to_string(),
            })
        }
    }
}

fn local_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    // Offset to local timezone (UTC+8 for China; adjust if needed)
    let local_secs = now.as_secs() + 8 * 3600;
    let days = local_secs / 86400;
    let time_of_day = local_secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let (year, month, day) = epoch_days_to_ymd(days);
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn epoch_days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_import(app: tauri::AppHandle, state: State<'_, AppState>) -> AppResult<OkResponse> {
    use tauri_plugin_dialog::DialogExt;

    let started = std::time::Instant::now();
    tracing::info!(target: crate::logging::app_target(), event = "database.import.started", layer = "backend", area = "database", outcome = "started", "database import started");
    let file_path = app
        .dialog()
        .file()
        .set_title("Import Database Backup")
        .add_filter("SQLite Database", &["db"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let src = path.as_path().unwrap_or_else(|| std::path::Path::new(""));
            if src.as_os_str().is_empty() {
                tracing::info!(target: crate::logging::app_target(), event = "database.import.completed", layer = "backend", area = "database", outcome = "canceled", duration_ms = started.elapsed().as_millis() as u64, "database import canceled");
                return Ok(OkResponse {
                    ok: false,
                    message: "No file selected".to_string(),
                });
            }

            // Validate it's a valid SQLite file by checking magic bytes
            let data = crate::filesystem::read_bytes(src).map_err(|e| {
                tracing::warn!(target: crate::logging::app_target(), event = "database.import.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to read database backup");
                AppError::FileSystemError(format!("Failed to read backup file: {}", e))
            })?;
            if data.len() < 16 || &data[..16] != b"SQLite format 3\0" {
                tracing::warn!(target: crate::logging::app_target(), event = "database.import.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, "database backup validation failed");
                return Err(AppError::InvalidInput(
                    "Selected file is not a valid SQLite database".into(),
                ));
            }

            let db_path = crate::config::default_db_path();

            // Step 1: WAL checkpoint to flush all pending writes to the main .db file
            let maint = MaintenanceRepository::new(&state.db);
            maint.wal_checkpoint().map_err(|e| {
                tracing::warn!(target: crate::logging::app_target(), event = "database.import.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "WAL checkpoint failed before import");
                AppError::DatabaseError(e.to_string())
            })?;

            // Step 2: Backup current database (including WAL/SHM if they exist)
            let backup_path = db_path.with_extension("db.pre_import_backup");
            if db_path.exists() {
                crate::filesystem::copy_file(&db_path, &backup_path).map_err(|e| {
                    tracing::warn!(target: crate::logging::app_target(), event = "database.import.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to back up current database before import");
                    AppError::FileSystemError(format!("Failed to backup current database: {}", e))
                })?;
            }

            // Step 3: Remove stale WAL and SHM files to prevent data inconsistency
            let wal_path = db_path.with_extension("db-wal");
            let shm_path = db_path.with_extension("db-shm");
            for stale_path in [&wal_path, &shm_path] {
                crate::filesystem::remove_file(stale_path).map_err(|err| {
                    tracing::warn!(target: crate::logging::app_target(), event = "database.import.stale_file_remove.failed", layer = "backend", area = "database", outcome = "failed", path = %stale_path.display(), duration_ms = started.elapsed().as_millis() as u64, error = %err, "failed to remove stale database sidecar file");
                    AppError::FileSystemError(format!(
                        "Failed to remove {}: {}",
                        stale_path.display(),
                        err
                    ))
                })?;
            }

            // Step 4: Write the imported file over the current database
            crate::filesystem::write_file(&db_path, &data).map_err(|e| {
                tracing::warn!(target: crate::logging::app_target(), event = "database.import.failed", layer = "backend", area = "database", outcome = "failed", duration_ms = started.elapsed().as_millis() as u64, error = %e, "failed to write imported database");
                AppError::FileSystemError(format!("Failed to write imported database: {}", e))
            })?;

            tracing::info!(target: crate::logging::app_target(), event = "database.import.completed", layer = "backend", area = "database", outcome = "success", duration_ms = started.elapsed().as_millis() as u64, "database import completed");
            Ok(OkResponse {
                ok: true,
                message: format!(
                    "Database imported from: {}. Previous database backed up to: {}. Restart the app to apply changes.",
                    src.display(),
                    backup_path.display()
                ),
            })
        }
        None => {
            tracing::info!(target: crate::logging::app_target(), event = "database.import.completed", layer = "backend", area = "database", outcome = "canceled", duration_ms = started.elapsed().as_millis() as u64, "database import canceled");
            Ok(OkResponse {
                ok: false,
                message: "Import cancelled".to_string(),
            })
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_open_folder() -> AppResult<OkResponse> {
    let db_path = crate::config::default_db_path();
    let folder = db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    if !folder.exists() {
        crate::filesystem::create_dir_all(folder).map_err(AppError::FileSystemError)?;
    }

    crate::filesystem::open_folder(folder).map_err(|e| AppError::FileSystemError(e))?;

    Ok(OkResponse {
        ok: true,
        message: "opened".to_string(),
    })
}
