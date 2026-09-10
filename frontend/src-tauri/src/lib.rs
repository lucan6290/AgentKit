pub mod commands;
pub mod config;
pub mod contracts;
pub mod db;
pub mod error;
pub mod filesystem;
pub mod logging;
pub mod models;
pub mod platform;
pub mod repo;
pub mod repositories;
pub mod services;
pub mod skills;
pub mod state;
pub mod tasks;
pub mod tools;
pub mod update;
pub mod utils;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

pub fn run() {
    let log_dir = crate::config::resolve_data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    // Install a panic hook so that crashes are captured in the dedicated error
    // log file. Without this, panics only print to stderr and are lost in a
    // GUI application.
    let error_log_path = log_dir.join("skills-hub-error.log");
    std::panic::set_hook(Box::new(move |panic_info| {
        use std::io::Write;
        let now = time::OffsetDateTime::now_utc();
        let timestamp = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        // force_capture() ignores the RUST_BACKTRACE env var, which is
        // typically unset in a desktop GUI application.
        let backtrace = std::backtrace::Backtrace::force_capture();
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_log_path)
            .and_then(|mut f| {
                writeln!(f, "[{}] [PANIC] {}", timestamp, panic_info)?;
                writeln!(f, "Backtrace:\n{}", backtrace)?;
                writeln!(f, "--------------------------------------------------")?;
                Ok(())
            });
        // Also attempt to route through the regular logger (may fail if the
        // panic originated inside the logger itself).
        tracing::error!(
            target: crate::logging::app_target(),
            event = "app.panic",
            layer = "backend",
            area = "app",
            outcome = "failed",
            "application panic"
        );
    }));

    // Read log level from DB before Tauri initializes (use a temporary connection).
    let log_level = {
        let db_path = crate::config::default_db_path();
        if let Ok(db) = crate::db::Database::new(&db_path) {
            let repo = crate::repositories::SettingsRepository::new(&db);
            repo.get("log_level").ok().flatten().unwrap_or_else(|| "info".to_string())
        } else {
            "info".to_string()
        }
    };
    crate::logging::init(&log_dir, &log_level);

    tauri::Builder::default()
        // single-instance must be the first plugin; the deep-link feature forwards
        // scheme URLs from a second process to the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            tracing::info!(
                target: crate::logging::app_target(),
                event = "app.instance.forwarded",
                layer = "backend",
                area = "app",
                outcome = "success",
                args = ?args,
                "second app instance forwarded"
            );
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        tracing::debug!(
                            target: crate::logging::app_target(),
                            event = "shortcut.triggered",
                            layer = "backend",
                            area = "shortcut",
                            outcome = "started",
                            shortcut = ?shortcut,
                            "global shortcut triggered"
                        );
                        show_main_window(app);
                    }
                })
                .build(),
        )
        .manage(state::AppState::default())
        .setup(move |app| {
            tracing::info!(
                target: crate::logging::app_target(),
                event = "app.start",
                layer = "backend",
                area = "app",
                outcome = "started",
                log_level = %log_level,
                "Skills Hub starting"
            );
            tracing::info!(
                target: crate::logging::app_target(),
                event = "app.setup.started",
                layer = "backend",
                area = "app",
                outcome = "started",
                "Tauri setup started"
            );
            build_tray(app.handle())?;

            // Intercept the close button based on user setting:
            // "minimize_to_tray" (default) → hide window; "quit" → exit app.
            let main_window = app
                .get_webview_window("main")
                .unwrap_or_else(|| {
                    tracing::error!(
                        target: crate::logging::app_target(),
                        event = "app.window.missing",
                        layer = "backend",
                        area = "window",
                        outcome = "failed",
                        "main window not found during setup"
                    );
                    panic!("main window not found");
                });
            let app_handle = app.handle().clone();
            let db_for_close = state::AppState::default_db_ref(&app_handle);

            // Log initial close behavior setting
            if let Some(db) = db_for_close.as_ref() {
                let repo = crate::repositories::SettingsRepository::new(db);
                let initial_behavior = repo.get("close_behavior").ok().flatten()
                    .unwrap_or_else(|| "minimize_to_tray".to_string());
                tracing::info!(
                    target: crate::logging::app_target(),
                    event = "settings.close_behavior.loaded",
                    layer = "backend",
                    area = "settings",
                    outcome = "success",
                    close_behavior = %initial_behavior,
                    "close behavior loaded"
                );
            }

            main_window.clone().on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        let behavior = db_for_close.as_ref()
                            .and_then(|db| {
                                let repo = crate::repositories::SettingsRepository::new(db);
                                repo.get("close_behavior").ok().flatten()
                            })
                            .unwrap_or_else(|| "minimize_to_tray".to_string());

                        tracing::info!(
                            target: crate::logging::app_target(),
                            event = "window.close.requested",
                            layer = "backend",
                            area = "window",
                            outcome = "started",
                            close_behavior = %behavior,
                            "window close requested"
                        );

                        match behavior.as_str() {
                            "quit" => {
                                tracing::info!(
                                    target: crate::logging::app_target(),
                                    event = "app.quit.requested",
                                    layer = "backend",
                                    area = "app",
                                    outcome = "started",
                                    "app exit requested"
                                );
                                app_handle.exit(0);
                            }
                            "minimize_to_taskbar" => {
                                tracing::info!(
                                    target: crate::logging::app_target(),
                                    event = "window.minimize.started",
                                    layer = "backend",
                                    area = "window",
                                    outcome = "started",
                                    destination = "taskbar",
                                    "minimize window to taskbar"
                                );
                                api.prevent_close();
                                if let Err(e) = main_window.minimize() {
                                    tracing::error!(
                                        target: crate::logging::app_target(),
                                        event = "window.minimize.failed",
                                        layer = "backend",
                                        area = "window",
                                        outcome = "failed",
                                        destination = "taskbar",
                                        error = %e,
                                        "failed to minimize window to taskbar"
                                    );
                                } else {
                                    tracing::debug!(
                                        target: crate::logging::app_target(),
                                        event = "window.minimize.completed",
                                        layer = "backend",
                                        area = "window",
                                        outcome = "success",
                                        destination = "taskbar",
                                        "window minimized to taskbar"
                                    );
                                }
                            }
                            _ => {
                                // Default: minimize to tray
                                tracing::info!(
                                    target: crate::logging::app_target(),
                                    event = "window.hide.started",
                                    layer = "backend",
                                    area = "window",
                                    outcome = "started",
                                    destination = "tray",
                                    "hide window to tray"
                                );
                                api.prevent_close();
                                if let Err(e) = main_window.hide() {
                                    tracing::error!(
                                        target: crate::logging::app_target(),
                                        event = "window.hide.failed",
                                        layer = "backend",
                                        area = "window",
                                        outcome = "failed",
                                        destination = "tray",
                                        error = %e,
                                        "failed to hide window to tray"
                                    );
                                } else {
                                    tracing::debug!(
                                        target: crate::logging::app_target(),
                                        event = "window.hide.completed",
                                        layer = "backend",
                                        area = "window",
                                        outcome = "success",
                                        destination = "tray",
                                        "window hidden to tray"
                                    );
                                }
                            }
                        }
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        tracing::trace!(
                            target: crate::logging::app_target(),
                            event = "window.focus.changed",
                            layer = "backend",
                            area = "window",
                            focused = *focused,
                            "window focus changed"
                        );
                    }
                    tauri::WindowEvent::Resized(size) => {
                        tracing::trace!(
                            target: crate::logging::app_target(),
                            event = "window.resize.changed",
                            layer = "backend",
                            area = "window",
                            width = size.width,
                            height = size.height,
                            "window size changed"
                        );
                    }
                    tauri::WindowEvent::Moved(position) => {
                        tracing::trace!(
                            target: crate::logging::app_target(),
                            event = "window.move.changed",
                            layer = "backend",
                            area = "window",
                            x = position.x,
                            y = position.y,
                            "window moved"
                        );
                    }
                    _ => {}
                }
            });

            #[cfg(desktop)]
            {
                // Non-fatal: if the hotkey is already taken by another app,
                // log a warning instead of crashing the entire setup.
                if let Err(e) = register_global_shortcut(app.handle()) {
                    tracing::warn!(
                        target: crate::logging::app_target(),
                        event = "shortcut.register.failed",
                        layer = "backend",
                        area = "shortcut",
                        outcome = "failed",
                        error = %e,
                        "failed to register global shortcut"
                    );
                }
            }

            // Register the skillshub:// scheme at runtime on Windows/Linux.
            // macOS uses the Info.plist entry generated from the config.
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }

            // Auto-refresh repo registries on startup if enabled.
            {
                let state = app.state::<state::AppState>();
                let repo = crate::repositories::SettingsRepository::new(&state.db);
                let auto_refresh = repo.get("auto_refresh_on_startup")
                    .ok()
                    .flatten()
                    .map(|v| v == "true")
                    .unwrap_or(false);
                tracing::info!(
                    target: crate::logging::app_target(),
                    event = "repo.auto_refresh.configured",
                    layer = "backend",
                    area = "repo",
                    enabled = auto_refresh,
                    "startup auto refresh setting loaded"
                );
                if auto_refresh {
                    let db = state.db.clone();
                    std::thread::spawn(move || {
                        tracing::debug!(
                            target: crate::logging::app_target(),
                            event = "repo.auto_refresh.started",
                            layer = "backend",
                            area = "repo",
                            outcome = "started",
                            "startup repository refresh started"
                        );
                        match crate::repo::scanner::sync_all_repo_registries(&db) {
                            Ok(result) => tracing::info!(
                                target: crate::logging::app_target(),
                                event = "repo.auto_refresh.completed",
                                layer = "backend",
                                area = "repo",
                                outcome = "success",
                                registered = result.registered,
                                removed = result.removed,
                                "startup repository refresh completed"
                            ),
                            Err(e) => tracing::warn!(
                                target: crate::logging::app_target(),
                                event = "repo.auto_refresh.failed",
                                layer = "backend",
                                area = "repo",
                                outcome = "failed",
                                error = %e,
                                "startup repository refresh failed"
                            ),
                        }
                    });
                }
            }

            tracing::info!(
                target: crate::logging::app_target(),
                event = "app.ready",
                layer = "backend",
                area = "app",
                outcome = "success",
                "Skills Hub started"
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // health
            crate::commands::health::health_check,
            // logging
            crate::commands::logging::write_frontend_log,
            // skills
            crate::commands::skills::get_managed_skills,
            crate::commands::skills::delete_managed_skill,
            crate::commands::skills::delete_managed_skills,
            crate::commands::skills::set_skill_enabled,
            crate::commands::skills::update_skill_source_url,
            crate::commands::skills::import_existing_skill,
            crate::commands::skills::list_local_skills_cmd,
            crate::commands::skills::install_local_selection,
            // tags
            crate::commands::tags::get_tags,
            crate::commands::tags::create_tag,
            crate::commands::tags::rename_tag,
            crate::commands::tags::delete_tag,
            crate::commands::tags::get_skill_tags,
            crate::commands::tags::set_skill_tags,
            crate::commands::tags::bulk_set_skill_tags,
            // sync
            crate::commands::sync::sync_skill_to_tool,
            crate::commands::sync::unsync_skill_from_tool,
            crate::commands::sync::sync_suite_to_tool,
            crate::commands::sync::unsync_suite_from_tool,
            crate::commands::sync::get_scope_preferences,
            crate::commands::sync::set_scope_preference,
            crate::commands::sync::get_recent_projects,
            crate::commands::sync::save_recent_project,
            crate::commands::sync::list_suite_sub_skills,
            crate::commands::sync::bulk_sync_skills,
            // files
            crate::commands::files::list_skill_files,
            crate::commands::files::read_skill_file,
            crate::commands::files::write_skill_file,
            // tools
            crate::commands::tools::get_tool_status,
            crate::commands::tools::get_tool_skills,
            crate::commands::tools::get_tool_adapter_configs,
            crate::commands::tools::save_tool_adapter_config,
            crate::commands::tools::reset_tool_adapter_config,
            crate::commands::tools::delete_tool_skill,
            crate::commands::tools::open_tool_skills_dir,
            crate::commands::tools::skill_to_community_repo,
            crate::commands::tools::clear_tool_skills,
            // settings
            crate::commands::settings::get_default_sync_tools,
            crate::commands::settings::save_default_sync_tools,
            crate::commands::settings::get_auto_check_update,
            crate::commands::settings::set_auto_check_update,
            crate::commands::settings::get_community_repo_path,
            crate::commands::settings::set_community_repo_path,
            crate::commands::settings::get_custom_repo_path,
            crate::commands::settings::set_custom_repo_path,
            crate::commands::settings::open_settings_folder,
            crate::commands::settings::get_close_behavior,
            crate::commands::settings::set_close_behavior,
            crate::commands::settings::get_show_tray_icon,
            crate::commands::settings::set_show_tray_icon,
            crate::commands::settings::get_log_level,
            crate::commands::settings::set_log_level,
            crate::commands::settings::get_auto_refresh_on_startup,
            crate::commands::settings::set_auto_refresh_on_startup,
            crate::commands::settings::reset_general_settings,
            // database
            crate::commands::database::db_overview,
            crate::commands::database::db_table_data,
            crate::commands::database::db_maintenance,
            crate::commands::database::db_reset,
            crate::commands::database::db_export,
            crate::commands::database::db_open_folder,
            crate::commands::database::db_import,
            // onboarding
            crate::commands::onboarding::get_onboarding_plan,
            // tasks
            crate::commands::tasks::get_task_list,
            crate::commands::tasks::get_task,
            crate::commands::tasks::cancel_task,
            // update
            crate::commands::update::check_update,
            crate::commands::update::do_update,
            // prompts
            crate::commands::prompts::scan_prompt_files,
            crate::commands::prompts::scan_project_prompt_files,
            crate::commands::prompts::get_prompt_files,
            crate::commands::prompts::read_prompt_file,
            crate::commands::prompts::write_prompt_file,
            crate::commands::prompts::delete_prompt_file,
            // misc
            crate::commands::misc::pick_folder,
            crate::commands::misc::cancel_current_operation,
            crate::commands::misc::reorder,
            crate::commands::misc::open_new_window,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tracing::error!(
                target: crate::logging::app_target(),
                event = "app.run.failed",
                layer = "backend",
                area = "app",
                outcome = "failed",
                error = %e,
                "failed to run Skills Hub"
            );
            panic!("failed to run Skills Hub: {}", e);
        });
}

/// Build the system tray icon with a context menu.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    tracing::debug!(
        target: crate::logging::app_target(),
        event = "tray.build.started",
        layer = "backend",
        area = "tray",
        outcome = "started",
        "building system tray"
    );
    let version = app.package_info().version.to_string();
    let version_label = format!("Skills Hub v{}", version);

    let menu = Menu::new(app)?;

    // --- Version info (disabled item) ---
    menu.append(&MenuItem::with_id(
        app,
        "version",
        &version_label,
        false,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- Window controls ---
    menu.append(&MenuItem::with_id(
        app,
        "show",
        "显示窗口",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "new_window",
        "新建窗口",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- Update & web ---
    menu.append(&MenuItem::with_id(
        app,
        "check_update",
        "检查更新",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "open_website",
        "打开官方网站",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- Open directory submenu ---
    let dir_submenu = Submenu::with_items(
        app,
        "打开目录",
        true,
        &[
            &MenuItem::with_id(app, "open_app_dir", "应用目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_data_dir", "工作目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_resource_dir", "内核目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_log_dir", "日志目录", true, None::<&str>)?,
        ],
    )?;
    menu.append(&dir_submenu)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- App lifecycle ---
    menu.append(&MenuItem::with_id(
        app,
        "restart",
        "重启应用",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| {
            tracing::error!(
                target: crate::logging::app_target(),
                event = "tray.icon.missing",
                layer = "backend",
                area = "tray",
                outcome = "failed",
                "window icon is not configured"
            );
            panic!("window icon must be configured");
        });

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Skills Hub")
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            tracing::debug!(
                target: crate::logging::app_target(),
                event = "tray.menu.clicked",
                layer = "backend",
                area = "tray",
                outcome = "started",
                menu_id = id,
                "tray menu clicked"
            );
            match id {
                "show" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.window.show.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray show window clicked"
                    );
                    show_main_window(app);
                }
                "new_window" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.window.new.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray new window clicked"
                    );
                    if let Err(e) = crate::commands::misc::create_new_window(app) {
                        tracing::error!(
                            target: crate::logging::app_target(),
                            event = "tray.window.new.failed",
                            layer = "backend",
                            area = "tray",
                            outcome = "failed",
                            error = %e,
                            "failed to create new window from tray"
                        );
                    }
                }
                "check_update" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.update_check.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray check update clicked"
                    );
                    tray_check_update(app);
                }
                "open_website" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.website.open.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray open website clicked"
                    );
                    open_url("https://github.com/lucan6290/skills-hub");
                }
                "open_app_dir" => open_app_directory(app, AppDir::App),
                "open_data_dir" => open_app_directory(app, AppDir::Data),
                "open_resource_dir" => open_app_directory(app, AppDir::Resource),
                "open_log_dir" => open_app_directory(app, AppDir::Log),
                "restart" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.app.restart.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray restart app clicked"
                    );
                    app.restart();
                }
                "quit" => {
                    tracing::info!(
                        target: crate::logging::app_target(),
                        event = "tray.app.quit.clicked",
                        layer = "backend",
                        area = "tray",
                        outcome = "started",
                        "tray quit app clicked"
                    );
                    app.exit(0);
                }
                _ => tracing::warn!(
                    target: crate::logging::app_target(),
                    event = "tray.menu.unknown",
                    layer = "backend",
                    area = "tray",
                    outcome = "failed",
                    menu_id = id,
                    "unknown tray menu id"
                ),
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                tracing::debug!(
                    target: crate::logging::app_target(),
                    event = "tray.icon.left_click",
                    layer = "backend",
                    area = "tray",
                    outcome = "started",
                    "tray icon left clicked"
                );
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    tracing::info!(
        target: crate::logging::app_target(),
        event = "tray.build.completed",
        layer = "backend",
        area = "tray",
        outcome = "success",
        "system tray built"
    );
    Ok(())
}

/// Directory types for the tray "打开目录" submenu.
enum AppDir {
    App,
    Data,
    Resource,
    Log,
}

/// Resolve and open a specific app directory in the system file manager.
fn open_app_directory(app: &tauri::AppHandle, dir: AppDir) {
    let dir_name = match dir {
        AppDir::App => "应用目录",
        AppDir::Data => "工作目录",
        AppDir::Resource => "内核目录",
        AppDir::Log => "日志目录",
    };
    tracing::info!(
        target: crate::logging::app_target(),
        event = "directory.open.started",
        layer = "backend",
        area = "filesystem",
        outcome = "started",
        directory_kind = %dir_name,
        "opening app directory"
    );

    let path = match dir {
        AppDir::App => std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        AppDir::Data => Some(crate::config::resolve_data_dir()),
        AppDir::Resource => app.path().resource_dir().ok(),
        AppDir::Log => Some(crate::config::resolve_data_dir().join("logs")),
    };

    let Some(path) = path else {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "directory.resolve.failed",
            layer = "backend",
            area = "filesystem",
            outcome = "failed",
            directory_kind = %dir_name,
            "failed to resolve app directory"
        );
        return;
    };
    tracing::debug!(
        target: crate::logging::app_target(),
        event = "directory.resolved",
        layer = "backend",
        area = "filesystem",
        outcome = "success",
        directory_kind = %dir_name,
        path = %path.display(),
        "app directory resolved"
    );

    if !path.exists() {
        tracing::debug!(
            target: crate::logging::app_target(),
            event = "directory.create.started",
            layer = "backend",
            area = "filesystem",
            outcome = "started",
            directory_kind = %dir_name,
            path = %path.display(),
            "creating missing app directory"
        );
        if let Err(e) = std::fs::create_dir_all(&path) {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "directory.create.failed",
                layer = "backend",
                area = "filesystem",
                outcome = "failed",
                directory_kind = %dir_name,
                path = %path.display(),
                error = %e,
                "failed to create app directory"
            );
            return;
        }
    }

    if let Err(e) = crate::filesystem::open_folder(&path) {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "directory.open.failed",
            layer = "backend",
            area = "filesystem",
            outcome = "failed",
            directory_kind = %dir_name,
            path = %path.display(),
            error = %e,
            "failed to open app directory"
        );
    } else {
        tracing::debug!(
            target: crate::logging::app_target(),
            event = "directory.open.completed",
            layer = "backend",
            area = "filesystem",
            outcome = "success",
            directory_kind = %dir_name,
            path = %path.display(),
            "app directory opened"
        );
    }
}

/// Open a URL in the system default browser.
fn open_url(url: &str) {
    tracing::info!(
        target: crate::logging::app_target(),
        event = "browser.open.started",
        layer = "backend",
        area = "shell",
        outcome = "started",
        url = %url,
        "opening url in default browser"
    );
    #[cfg(windows)]
    {
        match std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
        {
            Ok(_) => tracing::debug!(
                target: crate::logging::app_target(),
                event = "browser.open.spawned",
                layer = "backend",
                area = "shell",
                outcome = "success",
                platform = "windows",
                url = %url,
                "url open command spawned"
            ),
            Err(e) => tracing::error!(
                target: crate::logging::app_target(),
                event = "browser.open.failed",
                layer = "backend",
                area = "shell",
                outcome = "failed",
                platform = "windows",
                url = %url,
                error = %e,
                "failed to open url"
            ),
        }
    }
    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("open").arg(url).spawn() {
            Ok(_) => tracing::debug!(
                target: crate::logging::app_target(),
                event = "browser.open.spawned",
                layer = "backend",
                area = "shell",
                outcome = "success",
                platform = "macos",
                url = %url,
                "url open command spawned"
            ),
            Err(e) => tracing::error!(
                target: crate::logging::app_target(),
                event = "browser.open.failed",
                layer = "backend",
                area = "shell",
                outcome = "failed",
                platform = "macos",
                url = %url,
                error = %e,
                "failed to open url"
            ),
        }
    }
    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("xdg-open").arg(url).spawn() {
            Ok(_) => tracing::debug!(
                target: crate::logging::app_target(),
                event = "browser.open.spawned",
                layer = "backend",
                area = "shell",
                outcome = "success",
                platform = "linux",
                url = %url,
                "url open command spawned"
            ),
            Err(e) => tracing::error!(
                target: crate::logging::app_target(),
                event = "browser.open.failed",
                layer = "backend",
                area = "shell",
                outcome = "failed",
                platform = "linux",
                url = %url,
                error = %e,
                "failed to open url"
            ),
        }
    }
}

/// Check for updates from the tray menu, showing a system notification with the result.
fn tray_check_update(app: &tauri::AppHandle) {
    use tauri_plugin_notification::NotificationExt;

    tracing::debug!(
        target: crate::logging::app_target(),
        event = "tray.update_check.started",
        layer = "backend",
        area = "update",
        outcome = "started",
        "tray update check started"
    );
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let version = app_handle.package_info().version.to_string();
        let result = crate::update::check_for_update(&version, "tray");

        if let Some(err) = &result.error {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "tray.update_check.failed",
                layer = "backend",
                area = "update",
                outcome = "failed",
                current_version = %result.current_version,
                error = %err,
                "tray update check failed"
            );
        } else if result.update_available {
            tracing::info!(
                target: crate::logging::app_target(),
                event = "tray.update_check.update_available",
                layer = "backend",
                area = "update",
                outcome = "success",
                current_version = %result.current_version,
                latest_version = %result.latest_version,
                "tray update check found new version"
            );
        } else {
            tracing::debug!(
                target: crate::logging::app_target(),
                event = "tray.update_check.up_to_date",
                layer = "backend",
                area = "update",
                outcome = "success",
                current_version = %result.current_version,
                "tray update check is up to date"
            );
        }

        let (title, body) = if let Some(err) = &result.error {
            ("检查更新失败".to_string(), err.clone())
        } else if result.update_available {
            (
                "发现新版本".to_string(),
                format!("v{} → v{}\n点击应用内更新按钮进行安装", result.current_version, result.latest_version),
            )
        } else {
            ("已是最新版本".to_string(), format!("v{}", result.current_version))
        };

        if let Err(e) = app_handle
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show()
        {
            tracing::error!(
                target: crate::logging::app_target(),
                event = "tray.update_notification.failed",
                layer = "backend",
                area = "notification",
                outcome = "failed",
                error = %e,
                "failed to send tray update notification"
            );
        }
    });
}

/// Register the global hotkey Ctrl+Shift+Space to show/focus the main window.
#[cfg(desktop)]
fn register_global_shortcut(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let hotkey = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    app.global_shortcut().register(hotkey).map_err(|e| {
        tracing::warn!(
            target: crate::logging::app_target(),
            event = "shortcut.register.failed",
            layer = "backend",
            area = "shortcut",
            outcome = "failed",
            shortcut = "Ctrl+Shift+Space",
            error = %e,
            "global shortcut registration failed"
        );
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    tracing::info!(
        target: crate::logging::app_target(),
        event = "shortcut.register.completed",
        layer = "backend",
        area = "shortcut",
        outcome = "success",
        shortcut = "Ctrl+Shift+Space",
        "global shortcut registered"
    );
    Ok(())
}

/// Show, unminimize and focus the main window. No-op if the window is gone.
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        tracing::debug!(
            target: crate::logging::app_target(),
            event = "window.show.started",
            layer = "backend",
            area = "window",
            outcome = "started",
            "showing main window"
        );
        if let Err(e) = window.unminimize() {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "window.unminimize.failed",
                layer = "backend",
                area = "window",
                outcome = "failed",
                error = %e,
                "failed to unminimize main window"
            );
        }
        if let Err(e) = window.show() {
            tracing::error!(
                target: crate::logging::app_target(),
                event = "window.show.failed",
                layer = "backend",
                area = "window",
                outcome = "failed",
                error = %e,
                "failed to show main window"
            );
        } else {
            tracing::debug!(
                target: crate::logging::app_target(),
                event = "window.show.completed",
                layer = "backend",
                area = "window",
                outcome = "success",
                "main window shown"
            );
        }
        if let Err(e) = window.set_focus() {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "window.focus.failed",
                layer = "backend",
                area = "window",
                outcome = "failed",
                error = %e,
                "failed to focus main window"
            );
        } else {
            tracing::debug!(
                target: crate::logging::app_target(),
                event = "window.focus.completed",
                layer = "backend",
                area = "window",
                outcome = "success",
                "main window focused"
            );
        }
    } else {
        tracing::error!(
            target: crate::logging::app_target(),
            event = "window.main.missing",
            layer = "backend",
            area = "window",
            outcome = "failed",
            "main window missing"
        );
    }
}
