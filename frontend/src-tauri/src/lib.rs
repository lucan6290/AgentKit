pub mod commands;
pub mod config;
pub mod contracts;
pub mod db;
pub mod error;
pub mod filesystem;
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
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

pub fn run() {
    let log_dir = crate::config::resolve_data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    // Install a panic hook so that crashes are captured in the dedicated error
    // log file. Without this, panics only print to stderr and are lost in a
    // GUI application.
    let error_log_path = log_dir.join("skills-hub-error.log");
    std::panic::set_hook(Box::new(move |panic_info| {
        use std::io::Write;
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
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
        log::error!("PANIC: {}", panic_info);
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
    let level_filter = match log_level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    log::info!("Skills Hub 启动中, 日志级别: {}", log_level);

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Folder {
                    path: log_dir.clone(),
                    file_name: Some("skills-hub".to_string()),
                }),
                // Dedicated error log: only captures Error-level messages
                // → skills-hub-error.log
                Target::new(TargetKind::Folder {
                    path: log_dir,
                    file_name: Some("skills-hub-error".to_string()),
                })
                .filter(|metadata| metadata.level() == log::Level::Error),
                Target::new(TargetKind::Webview),
            ])
            .level(level_filter)
            .max_file_size(10_000_000) // 10 MB per file
            .rotation_strategy(RotationStrategy::KeepSome(7)) // keep 7 rotated files
            .timezone_strategy(TimezoneStrategy::UseLocal)
            .build())
        // single-instance must be the first plugin; the deep-link feature forwards
        // scheme URLs from a second process to the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            log::info!("检测到第二个实例启动，已转发到现有窗口, args={:?}", args);
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
                        log::debug!("全局快捷键触发: {:?}", shortcut);
                        show_main_window(app);
                    }
                })
                .build(),
        )
        .manage(state::AppState::default())
        .setup(|app| {
            log::info!("Tauri setup 阶段开始");
            build_tray(app.handle())?;

            // Intercept the close button based on user setting:
            // "minimize_to_tray" (default) → hide window; "quit" → exit app.
            let main_window = app
                .get_webview_window("main")
                .unwrap_or_else(|| {
                    log::error!("setup 阶段未找到主窗口");
                    panic!("main window not found");
                });
            let app_handle = app.handle().clone();
            let db_for_close = state::AppState::default_db_ref(&app_handle);

            // Log initial close behavior setting
            if let Some(db) = db_for_close.as_ref() {
                let repo = crate::repositories::SettingsRepository::new(db);
                let initial_behavior = repo.get("close_behavior").ok().flatten()
                    .unwrap_or_else(|| "minimize_to_tray".to_string());
                log::info!("关闭行为设置: {}", initial_behavior);
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

                        log::info!("用户触发窗口关闭，当前关闭行为: {}", behavior);

                        match behavior.as_str() {
                            "quit" => {
                                log::info!("执行退出应用");
                                app_handle.exit(0);
                            }
                            "minimize_to_taskbar" => {
                                log::info!("执行最小化到任务栏");
                                api.prevent_close();
                                if let Err(e) = main_window.minimize() {
                                    log::error!("最小化到任务栏失败: {}", e);
                                } else {
                                    log::debug!("窗口已最小化到任务栏");
                                }
                            }
                            _ => {
                                // Default: minimize to tray
                                log::info!("执行最小化到托盘 (默认)");
                                api.prevent_close();
                                if let Err(e) = main_window.hide() {
                                    log::error!("隐藏窗口到托盘失败: {}", e);
                                } else {
                                    log::debug!("窗口已隐藏到托盘");
                                }
                            }
                        }
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        log::trace!("窗口焦点变化: focused={}", focused);
                    }
                    tauri::WindowEvent::Resized(size) => {
                        log::trace!("窗口大小变化: {}x{}", size.width, size.height);
                    }
                    tauri::WindowEvent::Moved(position) => {
                        log::trace!("窗口移动: ({}, {})", position.x, position.y);
                    }
                    _ => {}
                }
            });

            #[cfg(desktop)]
            {
                // Non-fatal: if the hotkey is already taken by another app,
                // log a warning instead of crashing the entire setup.
                if let Err(e) = register_global_shortcut(app.handle()) {
                    log::warn!("全局快捷键注册失败: {e}");
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
                log::info!("启动时自动刷新: {}", if auto_refresh { "已启用" } else { "已禁用" });
                if auto_refresh {
                    let db = state.db.clone();
                    std::thread::spawn(move || {
                        log::debug!("开始启动时自动刷新仓库...");
                        match crate::repo::scanner::sync_all_repo_registries(&db) {
                            Ok(result) => log::info!(
                                "启动时自动刷新完成: registered={}, removed={}",
                                result.registered, result.removed
                            ),
                            Err(e) => log::warn!("启动时自动刷新失败: {}", e),
                        }
                    });
                }
            }

            log::info!("Skills Hub 启动完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // health
            crate::commands::health::health_check,
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
            log::error!("应用启动失败: {}", e);
            panic!("failed to run Skills Hub: {}", e);
        });
}

/// Build the system tray icon with a context menu.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    log::debug!("构建系统托盘...");
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
            log::error!("窗口图标未配置");
            panic!("window icon must be configured");
        });

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Skills Hub")
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            log::debug!("托盘菜单点击: {}", id);
            match id {
                "show" => {
                    log::info!("托盘菜单: 显示窗口");
                    show_main_window(app);
                }
                "new_window" => {
                    log::info!("托盘菜单: 新建窗口");
                    if let Err(e) = crate::commands::misc::create_new_window(app) {
                        log::error!("新建窗口失败: {}", e);
                    }
                }
                "check_update" => {
                    log::info!("托盘菜单: 检查更新");
                    tray_check_update(app);
                }
                "open_website" => {
                    log::info!("托盘菜单: 打开官方网站");
                    open_url("https://github.com/lucan6290/skills-hub");
                }
                "open_app_dir" => open_app_directory(app, AppDir::App),
                "open_data_dir" => open_app_directory(app, AppDir::Data),
                "open_resource_dir" => open_app_directory(app, AppDir::Resource),
                "open_log_dir" => open_app_directory(app, AppDir::Log),
                "restart" => {
                    log::info!("托盘菜单: 重启应用");
                    app.restart();
                }
                "quit" => {
                    log::info!("托盘菜单: 退出应用");
                    app.exit(0);
                }
                _ => log::warn!("未知托盘菜单 ID: {}", id),
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                log::debug!("托盘左键点击: 显示/聚焦窗口");
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    log::info!("系统托盘构建完成");
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
    log::info!("打开目录: {}", dir_name);

    let path = match dir {
        AppDir::App => std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        AppDir::Data => Some(crate::config::resolve_data_dir()),
        AppDir::Resource => app.path().resource_dir().ok(),
        AppDir::Log => Some(crate::config::resolve_data_dir().join("logs")),
    };

    let Some(path) = path else {
        log::warn!("无法解析目录路径: {}", dir_name);
        return;
    };
    log::debug!("目录路径: {}", path.display());

    if !path.exists() {
        log::debug!("目录不存在，正在创建: {}", path.display());
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::warn!("无法创建目录 {}: {}", path.display(), e);
            return;
        }
    }

    if let Err(e) = crate::filesystem::open_folder(&path) {
        log::warn!("无法打开目录 {}: {}", path.display(), e);
    } else {
        log::debug!("目录已在文件管理器中打开: {}", path.display());
    }
}

/// Open a URL in the system default browser.
fn open_url(url: &str) {
    log::info!("在默认浏览器中打开 URL: {}", url);
    #[cfg(windows)]
    {
        match std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
        {
            Ok(_) => log::debug!("URL 打开命令已执行 (Windows)"),
            Err(e) => log::error!("打开 URL 失败: {}", e),
        }
    }
    #[cfg(target_os = "macos")]
    {
        match std::process::Command::new("open").arg(url).spawn() {
            Ok(_) => log::debug!("URL 打开命令已执行 (macOS)"),
            Err(e) => log::error!("打开 URL 失败: {}", e),
        }
    }
    #[cfg(target_os = "linux")]
    {
        match std::process::Command::new("xdg-open").arg(url).spawn() {
            Ok(_) => log::debug!("URL 打开命令已执行 (Linux)"),
            Err(e) => log::error!("打开 URL 失败: {}", e),
        }
    }
}

/// Check for updates from the tray menu, showing a system notification with the result.
fn tray_check_update(app: &tauri::AppHandle) {
    use tauri_plugin_notification::NotificationExt;

    log::debug!("托盘触发检查更新");
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let version = app_handle.package_info().version.to_string();
        let result = crate::update::check_for_update(&version, "tray");

        if let Some(err) = &result.error {
            log::warn!("检查更新失败: {}", err);
        } else if result.update_available {
            log::info!("发现新版本: v{} → v{}", result.current_version, result.latest_version);
        } else {
            log::debug!("已是最新版本: v{}", result.current_version);
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
            log::error!("发送更新通知失败: {}", e);
        }
    });
}

/// Register the global hotkey Ctrl+Shift+Space to show/focus the main window.
#[cfg(desktop)]
fn register_global_shortcut(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let hotkey = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    app.global_shortcut().register(hotkey).map_err(|e| {
        log::warn!("全局快捷键注册失败: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    log::info!("全局快捷键 Ctrl+Shift+Space 注册成功");
    Ok(())
}

/// Show, unminimize and focus the main window. No-op if the window is gone.
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        log::debug!("显示主窗口");
        if let Err(e) = window.unminimize() {
            log::warn!("取消最小化失败: {}", e);
        }
        if let Err(e) = window.show() {
            log::error!("显示窗口失败: {}", e);
        } else {
            log::debug!("窗口已显示");
        }
        if let Err(e) = window.set_focus() {
            log::warn!("窗口聚焦失败: {}", e);
        } else {
            log::debug!("窗口已聚焦");
        }
    } else {
        log::error!("未找到主窗口，无法显示");
    }
}
