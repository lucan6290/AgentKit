use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

use crate::error::{AppError, AppResult};
use crate::update::{self, CheckUpdateResponse, PerformUpdateResponse};

/// Progress payload sent to the frontend during download.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct UpdateProgress {
    chunk_length: usize,
    content_length: Option<u64>,
}

/// Check for updates via the GitHub Releases API (for display: version + release notes).
///
/// The actual install path is handled by the native `tauri-plugin-updater` in `do_update`.
#[tauri::command(rename_all = "snake_case")]
pub async fn check_update(app: AppHandle) -> AppResult<CheckUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    let install_mode = detect_install_mode();

    let result = tokio_or_spawn(move || update::check_for_update(&current_version, &install_mode));

    Ok(result)
}

/// Download and install the latest update via the native `tauri-plugin-updater`.
///
/// Emits `update://progress` events during download: `{ chunk_length, content_length }`.
/// On completion the app restarts automatically.
#[tauri::command(rename_all = "snake_case")]
pub async fn do_update(app: AppHandle) -> AppResult<PerformUpdateResponse> {
    let updater = app.updater().map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "update.perform.updater_unavailable",
            layer = "backend",
            area = "update",
            outcome = "failed",
            error = %e,
            "updater is not configured"
        );
        AppError::UpdateError(format!("updater 未配置: {e}"))
    })?;

    let update = updater.check().await.map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "update.perform.check.failed",
            layer = "backend",
            area = "update",
            outcome = "failed",
            error = %e,
            "native update check failed"
        );
        AppError::UpdateError(format!("检查更新失败: {e}"))
    })?;

    let Some(update) = update else {
        return Ok(PerformUpdateResponse {
            ok: false,
            message: "当前已是最新版本".to_string(),
        });
    };

    let app_for_cb = app.clone();
    update
        .download_and_install(
            move |chunk_length, content_length| {
                let _ = app_for_cb.emit(
                    "update://progress",
                    UpdateProgress {
                        chunk_length,
                        content_length,
                    },
                );
            },
            || {
                // download finished callback
            },
        )
        .await
        .map_err(|e| {
            tracing::error!(
                target: crate::logging::app_target(),
                event = "update.perform.install.failed",
                layer = "backend",
                area = "update",
                outcome = "failed",
                version = %update.version,
                error = %e,
                "update download and install failed"
            );
            AppError::UpdateError(format!("下载/安装更新失败: {e}"))
        })?;

    app.restart()
}

fn detect_install_mode() -> String {
    // Simple heuristic: check if running from a portable directory or installed location
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let portable_flag = exe_dir.join("portable.flag");
        if portable_flag.exists() {
            return "portable".to_string();
        }
        // Check if in Program Files (installed via setup)
        let exe_str = exe.to_string_lossy().to_lowercase();
        if exe_str.contains("program files") {
            return "setup".to_string();
        }
    }
    "dev".to_string()
}

/// Run a blocking closure on a separate thread and wait for the result.
fn tokio_or_spawn<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(f)
        .join()
        .expect("background thread panicked")
}
