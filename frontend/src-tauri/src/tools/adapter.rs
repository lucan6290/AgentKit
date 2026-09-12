//! Tool adapter module.
//!
//! Provides tool configuration, path resolution, installation detection,
//! and skill directory scanning for all 44 built-in AI tools.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config;
use crate::platform;

/// A detected skill in a tool's skills directory.
#[derive(Debug, Clone, Serialize)]
pub struct DetectedSkill {
    pub tool: String,
    pub name: String,
    pub path: String,
    pub is_link: bool,
    pub link_target: Option<String>,
}

/// Runtime tool adapter built from database or config defaults.
#[derive(Debug, Clone)]
pub struct ToolAdapter {
    pub tool_key: String,
    pub display_name: String,
    pub relative_skills_dir: String,
    pub relative_detect_dir: String,
    pub supports_symlink: bool,
    pub supports_junction: bool,
    pub force_copy: bool,
    pub supports_project_scope_override: Option<bool>,
    pub project_relative_skills_dir: Option<String>,
    pub is_custom: bool,
}

// ── Default adapters from config ──────────────────────────────────────

/// Build default tool adapters from `config::default_tool_adapters()`.
pub fn default_adapters() -> Vec<ToolAdapter> {
    let defaults = config::default_tool_adapters();
    let mut result: Vec<ToolAdapter> = defaults
        .into_iter()
        .map(|(key, cfg)| ToolAdapter {
            tool_key: key,
            display_name: cfg.display_name,
            relative_skills_dir: cfg.skills_dir,
            relative_detect_dir: cfg.detect_dir,
            supports_symlink: cfg.supports_symlink,
            supports_junction: cfg.supports_junction,
            force_copy: cfg.force_copy,
            supports_project_scope_override: cfg.supports_project_scope,
            project_relative_skills_dir: cfg.project_skills_dir,
            is_custom: false,
        })
        .collect();
    result.sort_by(|a, b| a.tool_key.cmp(&b.tool_key));
    result
}

// ── DB-first effective adapters ───────────────────────────────────────

/// Database-first: read enabled tool configs from the database.
/// Falls back to config defaults if the database is unavailable.
pub fn effective_tool_adapters(db: &crate::db::Database) -> Vec<ToolAdapter> {
    use crate::repositories::ToolAdapterConfigsRepository;
    let repo = ToolAdapterConfigsRepository::new(db);
    match repo.list_enabled() {
        Ok(configs) if !configs.is_empty() => configs
            .into_iter()
            .map(|c| ToolAdapter {
                tool_key: c.tool_key,
                display_name: c.display_name,
                relative_skills_dir: c.skills_dir,
                relative_detect_dir: c.detect_dir,
                supports_symlink: c.supports_symlink,
                supports_junction: c.supports_junction,
                force_copy: c.force_copy,
                supports_project_scope_override: c.supports_project_scope,
                project_relative_skills_dir: c.project_skills_dir,
                is_custom: c.is_custom,
            })
            .collect(),
        _ => default_adapters(),
    }
}

/// Find an adapter by tool key (normalized).
pub fn adapter_by_key<'a>(adapters: &'a [ToolAdapter], key: &str) -> Option<&'a ToolAdapter> {
    let normalized = normalize_tool_key(key);
    adapters.iter().find(|a| a.tool_key == normalized)
}

// ── Path Resolution ───────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| "C:\\Users\\Default".to_string()),
        )
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
    }
}

/// Normalize path separators to the platform's native separator.
/// Config paths use `/` internally; on Windows they must be converted to `\`.
fn normalize_separators(path: String) -> String {
    #[cfg(windows)]
    {
        path.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path
    }
}

/// Resolve the global skills directory path for a tool adapter.
pub fn resolve_default_path(adapter: &ToolAdapter) -> String {
    let configured = expand_tilde(&adapter.relative_skills_dir);
    let p = PathBuf::from(&configured);
    if p.is_absolute() {
        return normalize_separators(configured);
    }
    let joined = home_dir().join(p).to_string_lossy().to_string();
    normalize_separators(joined)
}

/// Resolve the detect directory path for a tool adapter.
fn resolve_detect_path(adapter: &ToolAdapter) -> String {
    let configured = expand_tilde(&adapter.relative_detect_dir);
    let p = PathBuf::from(&configured);
    if p.is_absolute() {
        return normalize_separators(configured);
    }
    let joined = home_dir().join(p).to_string_lossy().to_string();
    normalize_separators(joined)
}

/// Return the project-relative skills directory.
fn project_relative_skills_dir(adapter: &ToolAdapter) -> &str {
    adapter
        .project_relative_skills_dir
        .as_deref()
        .unwrap_or(&adapter.relative_skills_dir)
}

/// Resolve the project-level skills directory path.
pub fn resolve_project_path(adapter: &ToolAdapter, project_root: &str) -> String {
    let rel = project_relative_skills_dir(adapter);
    let joined = PathBuf::from(project_root)
        .join(rel)
        .to_string_lossy()
        .to_string();
    normalize_separators(joined)
}

/// Whether the tool supports project-scope skills.
pub fn supports_project_scope(adapter: &ToolAdapter) -> bool {
    if let Some(override_val) = adapter.supports_project_scope_override {
        return override_val;
    }
    adapter.tool_key != "hermes_agent"
}

/// Return sync capabilities for a tool adapter.
pub fn tool_sync_capabilities(adapter: &ToolAdapter) -> SyncCapabilities {
    SyncCapabilities {
        supports_symlink: adapter.supports_symlink,
        supports_junction: adapter.supports_junction,
        force_copy: adapter.force_copy,
        supports_project_scope: supports_project_scope(adapter),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncCapabilities {
    pub supports_symlink: bool,
    pub supports_junction: bool,
    pub force_copy: bool,
    pub supports_project_scope: bool,
}

// ── Installation Detection ────────────────────────────────────────────

/// Extra installation indicators beyond the home config directory.
/// For tools that commonly leave residual config directories after uninstall,
/// we require at least one of these paths to also exist.
/// Paths support environment variables via %VAR% syntax (Windows) or $VAR (Unix).
/// Relative paths are resolved against the home directory (same as detect_dir).
struct ExtraIndicator {
    /// Tool key to apply these indicators to.
    tool_key: &'static str,
    /// Additional paths that must exist (at least one) to confirm installation.
    /// Supports:
    ///   - Absolute paths
    ///   - %APPDATA%/..., %LOCALAPPDATA%/... (Windows env vars)
    ///   - Paths starting with ~/ or relative to home
    paths: &'static [&'static str],
    /// If true, require the detect dir to contain files/subdirs beyond
    /// just a "skills" folder to avoid empty-residual false positives.
    require_content: bool,
}

/// Tools that need extra verification beyond home config directory presence.
/// This prevents false positives from leftover config directories after uninstall.
static EXTRA_INDICATORS: &[ExtraIndicator] = &[
    // Electron-based IDEs — leave %APPDATA% data dir when installed
    ExtraIndicator { tool_key: "trae", paths: &["%APPDATA%/Trae"], require_content: false },
    ExtraIndicator { tool_key: "trae_cn", paths: &["%APPDATA%/Trae CN", "%APPDATA%/TRAE SOLO CN"], require_content: false },
    ExtraIndicator { tool_key: "cursor", paths: &["%APPDATA%/Cursor", "%LOCALAPPDATA%/Programs/Cursor"], require_content: false },
    ExtraIndicator { tool_key: "windsurf", paths: &["%APPDATA%/Windsurf"], require_content: false },
    // CLI tools — installed via npm/cargo/brew, leave cmd in PATH or global npm
    ExtraIndicator { tool_key: "claude_code", paths: &["%APPDATA%/npm/claude.cmd", "claude"], require_content: false },
    ExtraIndicator { tool_key: "codex", paths: &["%APPDATA%/Codex", "%APPDATA%/npm/codex.cmd", "%USERPROFILE%/.codex/auth.json"], require_content: false },
    ExtraIndicator { tool_key: "github_copilot", paths: &["%APPDATA%/npm/copilot.cmd", "%LOCALAPPDATA%/Programs/GitHub Copilot"], require_content: true },
    // VS Code / IDE extensions — need actual extension installed or full CLI
    ExtraIndicator { tool_key: "cline", paths: &["%USERPROFILE%/.vscode/extensions/*cline*", "%APPDATA%/Code/User/globalStorage/*cline*"], require_content: true },
    ExtraIndicator { tool_key: "continue", paths: &["%APPDATA%/Continue", "%USERPROFILE%/.continue/config.yaml"], require_content: false },
    ExtraIndicator { tool_key: "codebuddy", paths: &["%APPDATA%/CodeBuddy", "%USERPROFILE%/.vscode/extensions/*codebuddy*"], require_content: true },
    ExtraIndicator { tool_key: "qoder", paths: &["%APPDATA%/Qoder", "%LOCALAPPDATA%/Programs/Qoder"], require_content: false },
    ExtraIndicator { tool_key: "roo_code", paths: &["%APPDATA%/Roo Code", "%USERPROFILE%/.vscode/extensions/*roo*"], require_content: true },
    ExtraIndicator { tool_key: "kilo_code", paths: &["%APPDATA%/Kilo Code", "%USERPROFILE%/.vscode/extensions/*kilo*"], require_content: true },
    ExtraIndicator { tool_key: "goose", paths: &["%APPDATA%/npm/goose.cmd", "goose"], require_content: false },
    ExtraIndicator { tool_key: "gemini_cli", paths: &["%APPDATA%/npm/gemini.cmd", "gemini"], require_content: false },
];

/// Expand environment variables in a path (%VAR% on Windows, $VAR on Unix).
fn expand_env_vars(path_str: &str) -> String {
    let mut result = path_str.to_string();
    // Expand %VAR% patterns (Windows style)
    let mut i = 0;
    while let Some(start) = result[i..].find('%') {
        let start = i + start;
        if let Some(end) = result[start + 1..].find('%') {
            let end = start + 1 + end;
            let var_name = &result[start + 1..end];
            if let Ok(val) = std::env::var(var_name) {
                result = format!("{}{}{}", &result[..start], val, &result[end + 1..]);
                i = start + val.len();
            } else {
                i = end + 1;
            }
        } else {
            break;
        }
    }
    result
}

/// Resolve an indicator path (supports env vars, ~/ prefix, absolute paths).
/// The returned path uses platform-native separators.
fn resolve_indicator_path(template: &str) -> PathBuf {
    let expanded = expand_env_vars(template);
    let p = PathBuf::from(&expanded);
    if p.is_absolute() {
        return PathBuf::from(normalize_separators(expanded));
    }
    // Handle ~/ prefix
    if expanded.starts_with("~/") || expanded.starts_with("~\\") {
        let after = &expanded[2..];
        let joined = home_dir().join(after).to_string_lossy().to_string();
        return PathBuf::from(normalize_separators(joined));
    }
    let joined = home_dir().join(p).to_string_lossy().to_string();
    PathBuf::from(normalize_separators(joined))
}

/// Check if a path exists, supporting simple glob suffix (*/suffix or prefix*/)
/// for VS Code extension directories.
fn path_with_glob_exists(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_string();
    if !path_str.contains('*') {
        return path.exists();
    }
    // Simple glob: handle * as a single wildcard in a path component
    // Find the component containing *
    let components: Vec<_> = path_str.split(|c| c == '\\' || c == '/').collect();
    for (i, comp) in components.iter().enumerate() {
        if comp.contains('*') {
            // Build the prefix path up to (but not including) this component
            let prefix: String = if i == 0 {
                String::new()
            } else {
                components[..i].join(std::path::MAIN_SEPARATOR_STR)
            };
            let prefix_path = if prefix.is_empty() {
                PathBuf::from(std::path::MAIN_SEPARATOR_STR)
            } else {
                PathBuf::from(&prefix)
            };
            if !prefix_path.exists() {
                return false;
            }
            let pattern = comp.replace('*', "");
            if let Ok(entries) = std::fs::read_dir(&prefix_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Match: *suffix or prefix*
                    if (comp.starts_with('*') && name.ends_with(&pattern))
                        || (comp.ends_with('*') && name.starts_with(&pattern))
                        || (comp.starts_with('*') && comp.ends_with('*') && name.contains(&pattern[1..pattern.len()-1]))
                    {
                        // If there are more components after the wildcard, check they exist
                        if i + 1 < components.len() {
                            let rest: String = components[i+1..].join(std::path::MAIN_SEPARATOR_STR);
                            if entry.path().join(&rest).exists() {
                                return true;
                            }
                        } else {
                            return true;
                        }
                    }
                }
            }
            return false;
        }
    }
    false
}

/// Check if the detect directory has meaningful content beyond just a skills folder.
/// Returns true if the directory exists and contains files/subdirs other than "skills".
fn detect_dir_has_content(detect_path: &Path) -> bool {
    let dir = match std::fs::read_dir(detect_path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name != "skills" && !name.starts_with('.') {
            return true;
        }
    }
    // Also check if there's a .lock or .config file
    for entry in std::fs::read_dir(detect_path).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && name != "." && name != ".." && name != ".gitkeep" {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                // Has a dotfile like .skill-lock.json — that's real content
                return true;
            }
        }
    }
    false
}

/// Check if a tool is installed.
///
/// First verifies the detect (config) directory exists in the home folder.
/// For tools with known uninstall-residue issues, additionally requires at least one
/// extra indicator (AppData program dir, npm global cmd, etc.) to confirm real installation.
pub fn is_tool_installed(adapter: &ToolAdapter) -> bool {
    if adapter.relative_detect_dir.is_empty() {
        return false;
    }
    let detect_path = resolve_detect_path(adapter);
    let detect_dir = Path::new(&detect_path);
    if !detect_dir.exists() {
        return false;
    }

    // Look up extra indicators for this tool
    if let Some(indicator) = EXTRA_INDICATORS.iter().find(|e| e.tool_key == adapter.tool_key) {
        // Check require_content: if set, detect_dir must have real content beyond just "skills"
        if indicator.require_content && !detect_dir_has_content(detect_dir) {
            return false;
        }
        // Check extra paths: at least one must exist
        for tmpl in indicator.paths {
            let resolved = resolve_indicator_path(tmpl);
            // For simple command names (no separators), check PATH
            if !tmpl.contains('/') && !tmpl.contains('\\') && !tmpl.contains('%') {
                if which::exists(tmpl) {
                    return true;
                }
                continue;
            }
            if path_with_glob_exists(&resolved) {
                return true;
            }
        }
        // None of the extra indicators matched
        return false;
    }

    // No extra indicators defined — directory existence is sufficient
    true
}

/// Minimal which/where helper: check if an executable is found in PATH.
mod which {
    pub fn exists(name: &str) -> bool {
        std::env::var_os("PATH").is_some_and(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let mut path = dir.join(name);
                if path.exists() {
                    return true;
                }
                path.set_extension("exe");
                if path.exists() {
                    return true;
                }
                path.set_extension("cmd");
                if path.exists() {
                    return true;
                }
                path.set_extension("ps1");
                path.exists()
            })
        })
    }
}

// ── Skills Dir Sharing ────────────────────────────────────────────────

/// Find all adapters sharing the same global skills directory.
pub fn adapters_sharing_skills_dir<'a>(
    adapters: &'a [ToolAdapter],
    adapter: &ToolAdapter,
) -> Vec<&'a ToolAdapter> {
    adapters
        .iter()
        .filter(|a| a.relative_skills_dir == adapter.relative_skills_dir)
        .collect()
}

/// Find all adapters sharing the same project skills directory.
pub fn adapters_sharing_project_skills_dir<'a>(
    adapters: &'a [ToolAdapter],
    adapter: &ToolAdapter,
) -> Vec<&'a ToolAdapter> {
    let rel = project_relative_skills_dir(adapter);
    adapters
        .iter()
        .filter(|a| project_relative_skills_dir(a) == rel)
        .collect()
}

// ── Skill Scanning ────────────────────────────────────────────────────

/// Detect if a path is a symlink or Windows junction.
fn detect_link(path: &Path) -> (bool, Option<String>) {
    // Try symlink first
    if path.is_symlink() {
        if let Ok(target) = std::fs::read_link(path) {
            return (true, Some(clean_windows_prefix(&target.to_string_lossy())));
        }
    }

    // Also try readlink in case is_symlink returns false but it's still a link
    if let Ok(target) = std::fs::read_link(path) {
        return (true, Some(clean_windows_prefix(&target.to_string_lossy())));
    }

    // Windows junction
    if platform::is_junction(path) {
        if let Ok(target) = std::fs::read_link(path) {
            return (true, Some(clean_windows_prefix(&target.to_string_lossy())));
        }
        return (true, None);
    }

    (false, None)
}

/// Strip Windows extended-length path prefix.
fn clean_windows_prefix(s: &str) -> String {
    let mut result = s.to_string();
    for prefix in &["\\\\?\\", "\\??\\"] {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].to_string();
        }
    }
    result
}

/// Scan a tool's skills directory for detected skills.
pub fn scan_tool_dir(adapter: &ToolAdapter, dir_path: &str) -> Vec<DetectedSkill> {
    let mut results = Vec::new();
    let dir = Path::new(dir_path);

    if !dir.exists() {
        return results;
    }

    let ignore_hint = "Application Support/com.tauri.dev/skills";

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => {
            let mut v: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            v.sort_by_key(|e| e.file_name());
            v
        }
        Err(_) => return results,
    };

    for entry in entries {
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            || (path.is_symlink() && path.read_link().map(|t| t.is_dir()).unwrap_or(false));

        if !is_dir {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        // Codex: skip .system directory
        if adapter.tool_key == "codex" && name == ".system" {
            continue;
        }

        let (is_link, link_target) = detect_link(&path);

        // Skip paths containing ignore hint
        let normalized_path = path_str.replace('\\', "/");
        if normalized_path.contains(ignore_hint) {
            continue;
        }
        if let Some(ref lt) = link_target {
            if lt.replace('\\', "/").contains(ignore_hint) {
                continue;
            }
        }

        // Must contain SKILL.md
        if !path.join("SKILL.md").is_file() {
            continue;
        }

        results.push(DetectedSkill {
            tool: adapter.tool_key.clone(),
            name,
            path: path_str,
            is_link,
            link_target,
        });
    }

    results
}

// ── Helpers ───────────────────────────────────────────────────────────

fn normalize_tool_key(key: &str) -> String {
    let cleaned = key
        .trim()
        .to_lowercase()
        .replace(' ', "_")
        .replace('-', "_");
    cleaned
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn expand_tilde(input: &str) -> String {
    crate::utils::path_safety::expand_home(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_adapters_count() {
        let adapters = default_adapters();
        assert_eq!(adapters.len(), 44, "must have exactly 44 built-in tools");
    }

    #[test]
    fn test_cursor_config_matches_python() {
        let adapters = default_adapters();
        let cursor = adapters.iter().find(|a| a.tool_key == "cursor").unwrap();
        assert_eq!(cursor.display_name, "Cursor");
        assert_eq!(cursor.relative_skills_dir, ".cursor/skills");
        assert_eq!(cursor.relative_detect_dir, ".cursor");
        assert_eq!(
            cursor.project_relative_skills_dir.as_deref(),
            Some(".agents/skills")
        );
        assert!(!cursor.supports_symlink);
        assert!(!cursor.supports_junction);
        assert!(cursor.force_copy);
        assert_eq!(cursor.supports_project_scope_override, Some(true));
    }

    #[test]
    fn test_claude_code_config() {
        let adapters = default_adapters();
        let cc = adapters
            .iter()
            .find(|a| a.tool_key == "claude_code")
            .unwrap();
        assert_eq!(cc.display_name, "Claude Code");
        assert_eq!(cc.relative_skills_dir, ".claude/skills");
        assert_eq!(cc.relative_detect_dir, ".claude");
        assert!(cc.supports_symlink);
        assert!(cc.supports_junction);
        assert!(!cc.force_copy);
    }

    #[test]
    fn test_hermes_agent_no_project_scope() {
        let adapters = default_adapters();
        let ha = adapters
            .iter()
            .find(|a| a.tool_key == "hermes_agent")
            .unwrap();
        assert_eq!(ha.supports_project_scope_override, Some(false));
        assert!(!supports_project_scope(ha));
    }

    #[test]
    fn test_normalize_tool_key() {
        assert_eq!(normalize_tool_key("Cursor"), "cursor");
        assert_eq!(normalize_tool_key("Claude Code"), "claude_code");
        assert_eq!(normalize_tool_key("trae-cn"), "trae_cn");
        assert_eq!(normalize_tool_key("  spaces  "), "spaces");
    }

    #[test]
    fn test_adapter_by_key() {
        let adapters = default_adapters();
        assert!(adapter_by_key(&adapters, "cursor").is_some());
        assert!(adapter_by_key(&adapters, "Cursor").is_some());
        assert!(adapter_by_key(&adapters, "nonexistent").is_none());
    }

    #[test]
    fn test_resolve_default_path_relative() {
        let adapter = ToolAdapter {
            tool_key: "test".to_string(),
            display_name: "Test".to_string(),
            relative_skills_dir: ".test/skills".to_string(),
            relative_detect_dir: ".test".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };
        let path = resolve_default_path(&adapter);
        assert!(path.contains(".test"));
        assert!(path.contains("skills"));
    }

    #[test]
    fn test_resolve_project_path() {
        let adapter = ToolAdapter {
            tool_key: "test".to_string(),
            display_name: "Test".to_string(),
            relative_skills_dir: ".test/skills".to_string(),
            relative_detect_dir: ".test".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: Some(true),
            project_relative_skills_dir: Some(".agents/skills".to_string()),
            is_custom: false,
        };
        let path = resolve_project_path(&adapter, "/my/project");
        assert!(path.contains("my"));
        assert!(path.contains("project"));
        assert!(path.contains(".agents"));
    }

    #[test]
    fn test_scan_tool_dir_empty() {
        let adapter = default_adapters().into_iter().next().unwrap();
        let result = scan_tool_dir(&adapter, "/nonexistent/path/12345");
        assert!(result.is_empty());
    }

    #[test]
    fn test_scan_tool_dir_finds_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        std::fs::create_dir_all(&skill_dir).unwrap();
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: My Skill\n---\n# My Skill\n",
        )
        .unwrap();

        let adapter = ToolAdapter {
            tool_key: "test".to_string(),
            display_name: "Test".to_string(),
            relative_skills_dir: ".test/skills".to_string(),
            relative_detect_dir: ".test".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };

        let result = scan_tool_dir(&adapter, &dir.path().to_string_lossy());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-skill");
        assert!(!result[0].is_link);
    }

    #[test]
    fn test_scan_tool_dir_skips_non_skill_dirs() {
        let dir = tempfile::tempdir().unwrap();
        // Directory without SKILL.md should be skipped
        let not_skill = dir.path().join("not-a-skill");
        std::fs::create_dir_all(&not_skill).unwrap();
        std::fs::write(not_skill.join("README.md"), "hello").unwrap();

        let adapter = ToolAdapter {
            tool_key: "test".to_string(),
            display_name: "Test".to_string(),
            relative_skills_dir: ".test/skills".to_string(),
            relative_detect_dir: ".test".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };

        let result = scan_tool_dir(&adapter, &dir.path().to_string_lossy());
        assert!(result.is_empty());
    }

    #[test]
    fn test_is_tool_installed_false_for_missing() {
        let adapter = ToolAdapter {
            tool_key: "nonexistent_tool_xyz".to_string(),
            display_name: "NonExistent".to_string(),
            relative_skills_dir: ".nonexistent_xyz/skills".to_string(),
            relative_detect_dir: ".nonexistent_xyz".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };
        assert!(!is_tool_installed(&adapter));
    }

    #[test]
    fn test_clean_windows_prefix() {
        assert_eq!(clean_windows_prefix("\\\\?\\C:\\Users"), "C:\\Users");
        assert_eq!(clean_windows_prefix("\\??\\C:\\Users"), "C:\\Users");
        assert_eq!(clean_windows_prefix("C:\\Users"), "C:\\Users");
    }

    #[test]
    fn test_all_44_tool_keys_present() {
        let adapters = default_adapters();
        let expected_keys = vec![
            "adal",
            "amp",
            "antigravity",
            "augment",
            "claude_code",
            "clawdbot",
            "cline",
            "codebuddy",
            "codex",
            "command_code",
            "continue",
            "copaw",
            "crush",
            "droid",
            "gemini_cli",
            "github_copilot",
            "goose",
            "hermes_agent",
            "iflow_cli",
            "junie",
            "kilo_code",
            "kiro_cli",
            "kimi_cli",
            "kode",
            "mcpjam",
            "mistral_vibe",
            "moltbot",
            "mux",
            "neovate",
            "openclaw",
            "openclaude",
            "opencode",
            "openhands",
            "pi",
            "pochi",
            "qoder",
            "qoderwork",
            "qwen_code",
            "roo_code",
            "trae",
            "trae_cn",
            "windsurf",
            "zencoder",
            "cursor",
        ];
        for key in &expected_keys {
            assert!(
                adapters.iter().any(|a| a.tool_key == *key),
                "missing tool: {}",
                key
            );
        }
    }
}
