use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub const DB_FILE_NAME: &str = "agentkit.db";

pub const LEGACY_APP_IDENTIFIERS: &[&str] = &["com.tauri.dev", "com.tauri.dev.skillshub", "com.lucan.skillshub"];

/// Specification for a prompt file that an AI tool uses.
#[derive(Debug, Clone)]
pub struct PromptFileSpec {
    /// File name, e.g. "CLAUDE.md"
    pub file_name: &'static str,
    /// Scope: "global", "project", or "both"
    pub scope: &'static str,
    /// Global path template relative to home dir (e.g. ".claude/CLAUDE.md")
    pub global_rel: Option<&'static str>,
    /// Project path template relative to project root (e.g. "CLAUDE.md")
    pub project_rel: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ToolAdapterDefaults {
    pub display_name: String,
    pub skills_dir: String,
    pub detect_dir: String,
    pub project_skills_dir: Option<String>,
    pub supports_symlink: bool,
    pub supports_junction: bool,
    pub force_copy: bool,
    pub supports_project_scope: Option<bool>,
    pub prompt_files: Vec<PromptFileSpec>,
}

pub fn default_tool_adapters() -> HashMap<String, ToolAdapterDefaults> {
    let mut map = HashMap::new();

    macro_rules! add_tool {
        ($key:expr, $name:expr, $skills:expr, $detect:expr, $proj:expr, $symlink:expr, $junction:expr, $copy:expr, $scope:expr, $prompts:expr) => {
            map.insert(
                $key.to_string(),
                ToolAdapterDefaults {
                    display_name: $name.to_string(),
                    skills_dir: $skills.to_string(),
                    detect_dir: $detect.to_string(),
                    project_skills_dir: $proj.map(|s: &str| s.to_string()),
                    supports_symlink: $symlink,
                    supports_junction: $junction,
                    force_copy: $copy,
                    supports_project_scope: $scope,
                    prompt_files: $prompts,
                },
            );
        };
    }

    add_tool!(
        "cursor",
        "Cursor",
        ".cursor/skills",
        ".cursor",
        Some(".agents/skills"),
        false,
        false,
        true,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".cursorrules",
            scope: "project",
            global_rel: None,
            project_rel: Some(".cursorrules")
        }]
    );
    add_tool!(
        "claude_code",
        "Claude Code",
        ".claude/skills",
        ".claude",
        Some(".claude/skills"),
        true,
        true,
        false,
        Some(true),
        vec![
            PromptFileSpec {
                file_name: "CLAUDE.md",
                scope: "both",
                global_rel: Some(".claude/CLAUDE.md"),
                project_rel: Some("CLAUDE.md")
            },
            PromptFileSpec {
                file_name: "AGENTS.md",
                scope: "both",
                global_rel: Some(".claude/AGENTS.md"),
                project_rel: Some("AGENTS.md")
            },
            PromptFileSpec {
                file_name: "README.md",
                scope: "project",
                global_rel: None,
                project_rel: Some("README.md")
            }
        ]
    );
    add_tool!(
        "codex",
        "Codex",
        ".codex/skills",
        ".codex",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![
            PromptFileSpec {
                file_name: "CLAUDE.md",
                scope: "both",
                global_rel: Some(".codex/CLAUDE.md"),
                project_rel: Some("CLAUDE.md")
            },
            PromptFileSpec {
                file_name: "AGENTS.md",
                scope: "both",
                global_rel: Some(".codex/AGENTS.md"),
                project_rel: Some("AGENTS.md")
            },
            PromptFileSpec {
                file_name: "README.md",
                scope: "project",
                global_rel: None,
                project_rel: Some("README.md")
            }
        ]
    );
    add_tool!(
        "opencode",
        "OpenCode",
        ".config/opencode/skills",
        ".config/opencode",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "antigravity",
        "Antigravity",
        ".gemini/antigravity/skills",
        ".gemini/antigravity",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "amp",
        "Amp",
        ".config/agents/skills",
        ".config/agents",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "kimi_cli",
        "Kimi Code CLI",
        ".config/agents/skills",
        ".config/agents",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "augment",
        "Augment",
        ".augment/skills",
        ".augment",
        Some(".augment/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: "AGENTS.md",
            scope: "project",
            global_rel: None,
            project_rel: Some("AGENTS.md")
        }]
    );
    add_tool!(
        "openclaw",
        "OpenClaw",
        ".openclaw/skills",
        ".openclaw",
        Some("skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "copaw",
        "Copaw",
        ".copaw/skill_pool",
        ".copaw",
        Some(".copaw/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "cline",
        "Cline",
        ".agents/skills",
        ".agents",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".clinerules",
            scope: "project",
            global_rel: None,
            project_rel: Some(".clinerules")
        }]
    );
    add_tool!(
        "codebuddy",
        "CodeBuddy",
        ".codebuddy/skills",
        ".codebuddy",
        Some(".codebuddy/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "command_code",
        "Command Code",
        ".commandcode/skills",
        ".commandcode",
        Some(".commandcode/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "continue",
        "Continue",
        ".continue/skills",
        ".continue",
        Some(".continue/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "crush",
        "Crush",
        ".config/crush/skills",
        ".config/crush",
        Some(".crush/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "junie",
        "Junie",
        ".junie/skills",
        ".junie",
        Some(".junie/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "iflow_cli",
        "iFlow CLI",
        ".iflow/skills",
        ".iflow",
        Some(".iflow/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "kiro_cli",
        "Kiro CLI",
        ".kiro/skills",
        ".kiro",
        Some(".kiro/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "kode",
        "Kode",
        ".kode/skills",
        ".kode",
        Some(".kode/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "mcpjam",
        "MCPJam",
        ".mcpjam/skills",
        ".mcpjam",
        Some(".mcpjam/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "mistral_vibe",
        "Mistral Vibe",
        ".vibe/skills",
        ".vibe",
        Some(".vibe/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "mux",
        "Mux",
        ".mux/skills",
        ".mux",
        Some(".mux/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "openclaude",
        "OpenClaude IDE",
        ".openclaude/skills",
        ".openclaude",
        Some(".openclaude/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "openhands",
        "OpenHands",
        ".openhands/skills",
        ".openhands",
        Some(".openhands/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "pi",
        "Pi",
        ".pi/agent/skills",
        ".pi",
        Some(".pi/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "qoder",
        "Qoder",
        ".qoder/skills",
        ".qoder",
        Some(".qoder/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "qoderwork",
        "QoderWork",
        ".qoderwork/skills",
        ".qoderwork",
        Some(".qoder/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "qwen_code",
        "Qwen Code",
        ".qwen/skills",
        ".qwen",
        Some(".qwen/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "trae",
        "Trae",
        ".trae/skills",
        ".trae",
        Some(".trae/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".traerules",
            scope: "project",
            global_rel: None,
            project_rel: Some(".traerules")
        }]
    );
    add_tool!(
        "trae_cn",
        "Trae CN",
        ".trae-cn/skills",
        ".trae-cn",
        Some(".trae/skills"),
        true,
        true,
        false,
        Some(true),
        vec![
            PromptFileSpec {
                file_name: ".traerules",
                scope: "project",
                global_rel: None,
                project_rel: Some(".traerules")
            },
            PromptFileSpec {
                file_name: "CLAUDE.md",
                scope: "both",
                global_rel: Some(".trae-cn/CLAUDE.md"),
                project_rel: Some("CLAUDE.md")
            },
            PromptFileSpec {
                file_name: "AGENTS.md",
                scope: "both",
                global_rel: Some(".trae-cn/AGENTS.md"),
                project_rel: Some("AGENTS.md")
            },
            PromptFileSpec {
                file_name: "README.md",
                scope: "project",
                global_rel: None,
                project_rel: Some("README.md")
            }
        ]
    );
    add_tool!(
        "zencoder",
        "Zencoder",
        ".zencoder/skills",
        ".zencoder",
        Some(".zencoder/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "neovate",
        "Neovate",
        ".neovate/skills",
        ".neovate",
        Some(".neovate/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "pochi",
        "Pochi",
        ".pochi/skills",
        ".pochi",
        Some(".pochi/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "adal",
        "AdaL",
        ".adal/skills",
        ".adal",
        Some(".adal/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "kilo_code",
        "Kilo Code",
        ".kilocode/skills",
        ".kilocode",
        Some(".kilocode/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".kilocoderc",
            scope: "project",
            global_rel: None,
            project_rel: Some(".kilocoderc")
        }]
    );
    add_tool!(
        "roo_code",
        "Roo Code",
        ".roo/skills",
        ".roo",
        Some(".roo/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".roorules",
            scope: "project",
            global_rel: None,
            project_rel: Some(".roorules")
        }]
    );
    add_tool!(
        "goose",
        "Goose",
        ".config/goose/skills",
        ".config/goose",
        Some(".goose/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".gooserc",
            scope: "project",
            global_rel: None,
            project_rel: Some(".gooserc")
        }]
    );
    add_tool!(
        "gemini_cli",
        "Gemini CLI",
        ".gemini/skills",
        ".gemini",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: "GEMINI.md",
            scope: "both",
            global_rel: Some(".gemini/GEMINI.md"),
            project_rel: Some("GEMINI.md")
        }]
    );
    add_tool!(
        "github_copilot",
        "GitHub Copilot",
        ".copilot/skills",
        ".copilot",
        Some(".agents/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: "copilot-instructions.md",
            scope: "project",
            global_rel: None,
            project_rel: Some(".github/copilot-instructions.md")
        }]
    );
    add_tool!(
        "clawdbot",
        "Clawdbot",
        ".clawdbot/skills",
        ".clawdbot",
        Some(".clawdbot/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "droid",
        "Droid",
        ".factory/skills",
        ".factory",
        Some(".factory/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "windsurf",
        "Windsurf",
        ".codeium/windsurf/skills",
        ".codeium/windsurf",
        Some(".windsurf/skills"),
        true,
        true,
        false,
        Some(true),
        vec![PromptFileSpec {
            file_name: ".windsurfrules",
            scope: "project",
            global_rel: None,
            project_rel: Some(".windsurfrules")
        }]
    );
    add_tool!(
        "moltbot",
        "MoltBot",
        ".moltbot/skills",
        ".moltbot",
        Some(".moltbot/skills"),
        true,
        true,
        false,
        Some(true),
        vec![]
    );
    add_tool!(
        "hermes_agent",
        "Hermes Agent",
        ".hermes/skills",
        ".hermes",
        Some(".hermes/skills"),
        true,
        true,
        false,
        Some(false),
        vec![]
    );

    map
}

/// Root directory: ~/.agentkit
pub fn resolve_root_dir() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agentkit")
}

/// Data directory: ~/.agentkit/data
pub fn resolve_data_dir() -> PathBuf {
    resolve_root_dir().join("data")
}

/// Migrate legacy data directory from ~/.skills-hub to ~/.agentkit.
/// Also renames the inner `skillshub` or `agentkit` subdirectory to `skills`.
/// Only runs if the old directory exists and the new one does not.
pub fn migrate_legacy_data_dir() {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let home = PathBuf::from(home);
    let old_root = home.join(".skills-hub");
    let new_root = resolve_root_dir();

    // Step 1: Rename ~/.skills-hub → ~/.agentkit (if old exists, new doesn't)
    if old_root.exists() && !new_root.exists() {
        if let Err(e) = std::fs::rename(&old_root, &new_root) {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "migrate.legacy_dir.failed",
                layer = "backend",
                area = "config",
                error = %e,
                "failed to rename ~/.skills-hub to ~/.agentkit"
            );
            return;
        }
        tracing::info!(
            target: crate::logging::app_target(),
            event = "migrate.legacy_dir.success",
            layer = "backend",
            area = "config",
            "migrated data directory from ~/.skills-hub to ~/.agentkit"
        );
    }

    // Step 2: Rename ~/.agentkit/skillshub → ~/.agentkit/skills (if old exists, new doesn't)
    let skills_dir = new_root.join("skills");
    let old_skillshub = new_root.join("skillshub");
    if old_skillshub.exists() && !skills_dir.exists() {
        if let Err(e) = std::fs::rename(&old_skillshub, &skills_dir) {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "migrate.inner_dir.failed",
                layer = "backend",
                area = "config",
                error = %e,
                "failed to rename ~/.agentkit/skillshub to ~/.agentkit/skills"
            );
        } else {
            tracing::info!(
                target: crate::logging::app_target(),
                event = "migrate.inner_dir.success",
                layer = "backend",
                area = "config",
                "migrated inner directory from skillshub to skills"
            );
        }
    }

    // Step 3: Rename ~/.agentkit/agentkit → ~/.agentkit/skills (if old exists, new doesn't)
    let old_agentkit = new_root.join("agentkit");
    if old_agentkit.exists() && !skills_dir.exists() {
        if let Err(e) = std::fs::rename(&old_agentkit, &skills_dir) {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "migrate.inner_dir.failed",
                layer = "backend",
                area = "config",
                error = %e,
                "failed to rename ~/.agentkit/agentkit to ~/.agentkit/skills"
            );
        } else {
            tracing::info!(
                target: crate::logging::app_target(),
                event = "migrate.inner_dir.success",
                layer = "backend",
                area = "config",
                "migrated inner directory from agentkit to skills"
            );
        }
    }
}

pub fn default_db_path() -> PathBuf {
    let data_dir = resolve_data_dir();
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join(DB_FILE_NAME)
}

/// Migrate DB paths from old directory names to `skills`.
/// Updates `community_path` and `source_ref` in the `skills` table:
/// - `~/.agentkit/agentkit/` → `~/.agentkit/skills/`
/// - `~/.agentkit/skillshub/` → `~/.agentkit/skills/`
/// Handles both `\` (Windows) and `/` (Unix) path separators.
/// If a new-path entry already exists (from a prior scan), the old entry is deleted instead.
pub fn migrate_db_skill_paths() {
    let db_path = default_db_path();
    if !db_path.exists() {
        return;
    }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                target: crate::logging::app_target(),
                event = "migrate.db_paths.open_failed",
                layer = "backend",
                area = "config",
                error = %e,
                "failed to open DB for path migration"
            );
            return;
        }
    };

    // Replace patterns for both Windows (\) and Unix (/) separators.
    // Each pair: (old_pattern, new_pattern)
    let replacements: &[(&str, &str)] = &[
        // Windows: agentkit → skills
        (r"\.agentkit\agentkit\", r"\.agentkit\skills\"),
        // Unix: agentkit → skills
        ("/.agentkit/agentkit/", "/.agentkit/skills/"),
        // Windows: skillshub → skills
        (r"\.agentkit\skillshub\", r"\.agentkit\skills\"),
        // Unix: skillshub → skills
        ("/.agentkit/skillshub/", "/.agentkit/skills/"),
    ];

    let mut total_deleted = 0;
    let mut total_updated = 0;

    for (old_pat, new_pat) in replacements {
        let like_pattern = format!("%{}%", old_pat);

        // Step 1: Delete old-path entries whose new-path already exists (duplicates).
        // This prevents UNIQUE constraint violations during the UPDATE.
        let delete_sql = format!(
            "DELETE FROM skills WHERE community_path LIKE ? AND REPLACE(community_path, ?, ?) IN (SELECT community_path FROM skills)"
        );
        match conn.execute(&delete_sql, rusqlite::params![&like_pattern, old_pat, new_pat]) {
            Ok(count) => total_deleted += count,
            Err(e) => {
                tracing::warn!(
                    target: crate::logging::app_target(),
                    event = "migrate.db_paths.delete_failed",
                    layer = "backend",
                    area = "config",
                    error = %e,
                    pattern = old_pat,
                    "failed to delete duplicate old-path entries"
                );
            }
        }

        // Step 2: Update remaining old-path entries to new path.
        let update_sql = format!(
            "UPDATE skills SET community_path = REPLACE(community_path, ?, ?), source_ref = REPLACE(source_ref, ?, ?) WHERE community_path LIKE ? OR source_ref LIKE ?"
        );
        match conn.execute(
            &update_sql,
            rusqlite::params![
                old_pat, new_pat,
                old_pat, new_pat,
                &like_pattern, &like_pattern,
            ],
        ) {
            Ok(count) => total_updated += count,
            Err(e) => {
                tracing::warn!(
                    target: crate::logging::app_target(),
                    event = "migrate.db_paths.update_failed",
                    layer = "backend",
                    area = "config",
                    error = %e,
                    pattern = old_pat,
                    "failed to update DB paths"
                );
            }
        }
    }

    if total_deleted > 0 || total_updated > 0 {
        tracing::info!(
            target: crate::logging::app_target(),
            event = "migrate.db_paths.success",
            layer = "backend",
            area = "config",
            deleted_duplicates = total_deleted,
            updated_rows = total_updated,
            "migrated DB skill paths from old directory names to 'skills'"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tool_adapters_count() {
        let adapters = default_tool_adapters();
        assert!(adapters.len() >= 40);
    }

    #[test]
    fn test_cursor_config() {
        let adapters = default_tool_adapters();
        let cursor = adapters.get("cursor").unwrap();
        assert_eq!(cursor.display_name, "Cursor");
        assert_eq!(cursor.skills_dir, ".cursor/skills");
        assert!(cursor.force_copy);
        assert!(!cursor.supports_symlink);
    }

    #[test]
    fn test_resolve_data_dir_returns_path() {
        let path = resolve_data_dir();
        assert!(!path.as_os_str().is_empty());
    }
}
