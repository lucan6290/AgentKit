use std::path::Path;
use std::sync::OnceLock;

use serde_json::{Map, Value};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{filter_fn, LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload;

use crate::contracts::FrontendLogPayload;

const APP_TARGET: &str = "skills_hub";
const FRONTEND_TARGET: &str = "frontend";
const MAX_STRING_LEN: usize = 4_096;
const MAX_REDACTION_DEPTH: usize = 8;

static LOG_GUARDS: OnceLock<Vec<WorkerGuard>> = OnceLock::new();
static LEVEL_RELOAD_HANDLE: OnceLock<reload::Handle<LevelFilter, tracing_subscriber::Registry>> =
    OnceLock::new();

pub fn init(log_dir: &Path, configured_level: &str) {
    if let Err(err) = std::fs::create_dir_all(log_dir) {
        eprintln!(
            "failed to create log directory {}: {}",
            log_dir.display(),
            err
        );
        return;
    }

    let (level_filter, reload_handle) = reload::Layer::new(parse_level_filter(configured_level));
    let file_appender = tracing_appender::rolling::daily(log_dir, "skills-hub.jsonl");
    let error_appender = tracing_appender::rolling::daily(log_dir, "skills-hub-error.jsonl");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);
    let (error_writer, error_guard) = tracing_appender::non_blocking(error_appender);

    let file_layer = fmt::layer()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(file_writer);

    let stdout_layer = fmt::layer()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(std::io::stdout);

    let error_layer = fmt::layer()
        .json()
        .flatten_event(true)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(error_writer)
        .with_filter(filter_fn(|metadata| *metadata.level() <= Level::ERROR));

    match tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(level_filter)
            .with(file_layer)
            .with(stdout_layer)
            .with(error_layer),
    ) {
        Ok(()) => {
            if LOG_GUARDS.set(vec![file_guard, error_guard]).is_err() {
                eprintln!("logging guards were already initialized");
            }
            if LEVEL_RELOAD_HANDLE.set(reload_handle).is_err() {
                eprintln!("log level reload handle was already initialized");
            }
        }
        Err(err) => eprintln!("failed to initialize tracing subscriber: {}", err),
    }
}

pub fn set_level(level: &str) -> Result<(), String> {
    let level_filter = match level {
        "debug" | "info" | "warn" | "error" => parse_level_filter(level),
        _ => return Err(format!("invalid log level: {}", level)),
    };

    let handle = LEVEL_RELOAD_HANDLE
        .get()
        .ok_or_else(|| "logging has not been initialized".to_string())?;
    handle
        .reload(level_filter)
        .map_err(|err| format!("failed to reload log level: {}", err))
}

pub fn emit_frontend_log(payload: FrontendLogPayload) {
    let event_name = normalize_required(payload.event, "frontend.event");
    let message = truncate_string(&payload.message);
    let area = payload.area.unwrap_or_else(|| "app".to_string());
    let outcome = payload.outcome.unwrap_or_else(|| "unknown".to_string());
    let command = payload.command.unwrap_or_default();
    let task_id = payload.task_id.unwrap_or_default();
    let error_code = payload.error_code.unwrap_or_default();
    let duration_ms = payload.duration_ms;
    let app_version = payload.app_version.unwrap_or_default();
    let meta = payload.meta.map(redact_json).unwrap_or(Value::Null);
    let meta_json = meta.to_string();

    match payload.level.as_str() {
        "debug" => tracing::debug!(
            target: FRONTEND_TARGET,
            event = %event_name,
            layer = "frontend",
            area = %area,
            outcome = %outcome,
            command = %command,
            task_id = %task_id,
            error_code = %error_code,
            duration_ms = duration_ms,
            app_version = %app_version,
            meta = %meta_json,
            "{}",
            message
        ),
        "warn" => tracing::warn!(
            target: FRONTEND_TARGET,
            event = %event_name,
            layer = "frontend",
            area = %area,
            outcome = %outcome,
            command = %command,
            task_id = %task_id,
            error_code = %error_code,
            duration_ms = duration_ms,
            app_version = %app_version,
            meta = %meta_json,
            "{}",
            message
        ),
        "error" => tracing::error!(
            target: FRONTEND_TARGET,
            event = %event_name,
            layer = "frontend",
            area = %area,
            outcome = %outcome,
            command = %command,
            task_id = %task_id,
            error_code = %error_code,
            duration_ms = duration_ms,
            app_version = %app_version,
            meta = %meta_json,
            "{}",
            message
        ),
        _ => tracing::info!(
            target: FRONTEND_TARGET,
            event = %event_name,
            layer = "frontend",
            area = %area,
            outcome = %outcome,
            command = %command,
            task_id = %task_id,
            error_code = %error_code,
            duration_ms = duration_ms,
            app_version = %app_version,
            meta = %meta_json,
            "{}",
            message
        ),
    }
}

pub fn parse_level_filter(level: &str) -> LevelFilter {
    match level {
        "debug" => LevelFilter::DEBUG,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    }
}

pub const fn app_target() -> &'static str {
    APP_TARGET
}

fn normalize_required(value: String, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        truncate_string(trimmed)
    }
}

fn redact_json(value: Value) -> Value {
    redact_json_at_depth(value, 0)
}

fn redact_json_at_depth(value: Value, depth: usize) -> Value {
    if depth >= MAX_REDACTION_DEPTH {
        return Value::String("[TRUNCATED]".to_string());
    }

    match value {
        Value::Object(map) => {
            let mut redacted = Map::with_capacity(map.len());
            for (key, value) in map {
                if is_sensitive_key(&key) {
                    redacted.insert(key, Value::String("[REDACTED]".to_string()));
                } else {
                    redacted.insert(key, redact_json_at_depth(value, depth + 1));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|item| redact_json_at_depth(item, depth + 1))
                .collect(),
        ),
        Value::String(value) => Value::String(truncate_string(&value)),
        other => other,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "token"
            | "secret"
            | "password"
            | "authorization"
            | "cookie"
            | "api_key"
            | "access_key"
            | "refresh_token"
            | "private_key"
    )
}

fn truncate_string(value: &str) -> String {
    if value.len() <= MAX_STRING_LEN {
        return value.to_string();
    }

    let mut end = MAX_STRING_LEN;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...[TRUNCATED]", &value[..end])
}
