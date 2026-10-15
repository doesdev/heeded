//! Path arithmetic the way Node's `path` module did it, so that what is
//! printed and stored matches: lexical, no filesystem access.

use std::path::{Component, Path, PathBuf};

/// Drops `.` and folds `..` without touching the disk.
pub fn clean(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();

    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }

    out
}

/// A Git Bash path such as `/c/Users` as Windows knows it.
pub fn native(dir: &str) -> String {
    if !cfg!(windows) {
        return dir.to_string();
    }

    let bytes = dir.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic();
    let ends = bytes.len() == 2 || (bytes.len() > 2 && bytes[2] == b'/');

    if drive && ends {
        format!("{}:/{}", &dir[1..2], dir.get(3..).unwrap_or(""))
    } else {
        dir.to_string()
    }
}

/// `base` joined with `path`, unless `path` is already absolute.
pub fn resolve(base: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        clean(path)
    } else {
        clean(&base.join(path))
    }
}

fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return clean(path);
    }

    let here = std::env::current_dir().unwrap_or_default();

    clean(&here.join(path))
}

fn same(a: &Component, b: &Component) -> bool {
    let (a, b) = (
        a.as_os_str().to_string_lossy(),
        b.as_os_str().to_string_lossy(),
    );

    if cfg!(windows) {
        a.eq_ignore_ascii_case(&b)
    } else {
        a == b
    }
}

/// Forward slashes, whatever the platform.
pub fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// `file` relative to `root`, with forward slashes. A file on another
/// drive comes back whole.
pub fn within(root: &Path, file: &Path) -> String {
    let (root, file) = (absolute(root), absolute(file));
    let from: Vec<Component> = root.components().collect();
    let to: Vec<Component> = file.components().collect();

    if from
        .first()
        .zip(to.first())
        .is_none_or(|(a, b)| !same(a, b))
    {
        return slashed(&file);
    }

    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(a, b)| same(a, b))
        .count();
    let mut parts: Vec<String> = vec!["..".to_string(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|part| part.as_os_str().to_string_lossy().into_owned()),
    );

    parts.join("/")
}

/// The folder heeded keeps its own config and state in.
pub fn user_home() -> PathBuf {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };

    std::env::var_os(var).map(PathBuf::from).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_matches_node() {
        let root = if cfg!(windows) { "C:\\a\\b" } else { "/a/b" };
        let cases = [
            ("c/d.txt", "c/d.txt"),
            ("..", ".."),
            ("../x", "../x"),
            (".", ""),
        ];

        for (rel, expected) in cases {
            let file = resolve(Path::new(root), rel);
            assert_eq!(within(Path::new(root), &file), expected, "{rel}");
        }
    }

    #[test]
    fn native_turns_git_bash_paths_into_windows_ones() {
        if cfg!(windows) {
            assert_eq!(native("/c/Users/x"), "c:/Users/x");
            assert_eq!(native("/c"), "c:/");
        }
        assert_eq!(native("sub/dir"), "sub/dir");
        assert_eq!(native("/cd/x"), "/cd/x");
    }

    #[test]
    fn clean_folds_dots() {
        let path = clean(Path::new("a/./b/../c"));
        assert_eq!(slashed(&path), "a/c");
    }
}
