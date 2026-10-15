use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::paths::{clean, resolve};

/// Runs git in `cwd`. Its output when it succeeds, `None` when it fails
/// or cannot start. Never takes an optional lock: a status refresh that
/// wrote the index could collide with the agent's own git command.
pub fn git(cwd: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The top of the repo `dir` is in, or `dir` itself outside one.
pub fn root_of(dir: &Path) -> PathBuf {
    match git(dir, &["rev-parse", "--show-toplevel"]) {
        Some(top) => clean(Path::new(top.trim())),
        None => dir.to_path_buf(),
    }
}

/// The git folder that every worktree of a repo shares.
pub fn common_of(dir: &Path) -> Option<PathBuf> {
    let found = git(dir, &["rev-parse", "--git-common-dir"])?;

    Some(resolve(dir, found.trim()))
}
