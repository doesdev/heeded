use std::path::PathBuf;

use crate::paths::user_home;

pub const MARKER: &str = ".heeded.json";
pub const NOTES: &str = ".heeded";
pub const HANDOFF: &str = "handoff.md";
pub const JOURNAL: &str = "journal.jsonl";
pub const REWORK: &str = "rework.jsonl";
pub const SETTINGS: &str = "config.json";
pub const JOURNAL_MAX: usize = 500;
pub const MAX_BLOCKS: u64 = 3;
pub const DIRECTIVE_MAX: usize = 4000;

fn var(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Where the user's config lives: `HEEDED_HOME`, else `~/.heeded`.
pub fn home_dir() -> PathBuf {
    var("HEEDED_HOME").unwrap_or_else(|| user_home().join(".heeded"))
}

/// Where session state lives: the harness's plugin data folder, else
/// a folder under the user's heeded home.
pub fn state_dir() -> PathBuf {
    var("CLAUDE_PLUGIN_DATA").unwrap_or_else(|| home_dir().join("state"))
}
