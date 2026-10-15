//! What the repo looks like right now: where its notes are, a fingerprint
//! of the handoff, and one of everything else that has changed.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::consts::{HANDOFF, JOURNAL};
use crate::git::{common_of, git, root_of};
use crate::notes::notes_of;
use crate::paths::{clean, within};

pub const SECTIONS: [&str; 4] = ["Did", "Did not", "Open", "Next"];

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Side {
    pub root: String,
    pub tree: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Snapshot {
    pub root: String,
    pub notes: Option<String>,
    pub file: Option<String>,
    pub journal: Option<String>,
    pub handoff: Option<String>,
    pub tree: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
}

/// FNV-1a, 64 bits, as hex: stable across builds, which a stored
/// fingerprint has to be.
pub fn fingerprint(data: &[u8]) -> String {
    let hash = data.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    });

    format!("{hash:016x}")
}

pub fn hash_file(file: &Path) -> Option<String> {
    if file.is_dir() {
        return None;
    }

    std::fs::read(file).ok().map(|data| fingerprint(&data))
}

// a porcelain v2 record: its status letters, and the path after `skip`
// space-separated fields
fn record(rest: &str, skip: usize) -> (String, String) {
    let parts: Vec<&str> = rest.splitn(skip + 1, ' ').collect();
    let xy = parts.first().copied().unwrap_or("");

    (
        xy.to_string(),
        parts.last().copied().unwrap_or("").to_string(),
    )
}

/// HEAD and every changed path from `git status --porcelain=v2 --branch
/// -z`. A rename or copy carries its original path in the next field.
pub fn read_status(status: &str) -> (String, Vec<(String, String)>) {
    let mut fields = status.split('\0').filter(|field| !field.is_empty());
    let mut head = String::new();
    let mut paths = Vec::new();

    while let Some(field) = fields.next() {
        let (kind, rest) = field.split_once(' ').unwrap_or((field, ""));

        match kind {
            "#" => {
                if let Some(oid) = rest.strip_prefix("branch.oid ") {
                    head = oid.to_string();
                }
            }
            "1" => paths.push(record(rest, 7)),
            "2" => {
                paths.push(record(rest, 8));
                fields.next();
            }
            "u" => paths.push(record(rest, 9)),
            "?" => paths.push(("??".to_string(), rest.to_string())),
            _ => {}
        }
    }

    (head, paths)
}

/// One fingerprint of HEAD and every changed file, leaving out `ours` and
/// any nested repo that shows up as a whole folder. One git call.
pub fn hash_tree(root: &Path, ours: &[String]) -> Option<String> {
    let status = git(
        root,
        &["status", "--porcelain=v2", "--branch", "-z", "-uall"],
    )?;
    let (head, paths) = read_status(&status);
    let entries: Vec<_> = paths
        .into_iter()
        .filter(|(_, file)| !ours.contains(file) && !file.ends_with('/'))
        .map(|(xy, file)| json!([xy, file, hash_file(&root.join(&file))]))
        .collect();

    Some(fingerprint(
        json!([head.trim(), entries]).to_string().as_bytes(),
    ))
}

// another worktree of the same repo, when that is where the event happened
fn side_of(dir: &Path, root: &Path, cwd: Option<&str>) -> Option<Side> {
    let cwd = clean(Path::new(cwd?));
    if cwd == clean(dir) || cwd == root {
        return None;
    }

    let side = root_of(&cwd);
    if side == root {
        return None;
    }

    let common = common_of(root)?;
    if common_of(&side)? != common {
        return None;
    }

    Some(Side {
        root: side.display().to_string(),
        tree: hash_tree(&side, &[]),
    })
}

pub fn snapshot(dir: &Path, cwd: Option<&str>) -> Result<Snapshot, String> {
    let root: PathBuf = root_of(dir);
    let notes = notes_of(&root)?;
    let file = notes.as_ref().map(|notes| notes.join(HANDOFF));
    let journal = notes.as_ref().map(|notes| notes.join(JOURNAL));
    let ours: Vec<String> = [&file, &journal]
        .into_iter()
        .flatten()
        .map(|own| within(&root, own))
        .collect();
    let show = |path: &Option<PathBuf>| path.as_ref().map(|path| path.display().to_string());

    Ok(Snapshot {
        handoff: file.as_ref().and_then(|file| hash_file(file)),
        tree: hash_tree(&root, &ours),
        side: side_of(dir, &root, cwd),
        notes: show(&notes),
        file: show(&file),
        journal: show(&journal),
        root: root.display().to_string(),
    })
}

/// What is wrong with a handoff's shape, by name.
pub fn check_handoff(text: &str) -> Vec<&'static str> {
    let heads: Vec<&str> = text
        .lines()
        .filter_map(|line| line.trim_end_matches('\r').strip_prefix("## "))
        .collect();
    let first = text.split('\n').next().unwrap_or("").trim_end_matches('\r');
    let titled = first == "# Handoff" && text.len() > first.len();
    let dated = text.lines().any(|line| {
        line.strip_prefix("Last session: ")
            .is_some_and(|rest| rest.starts_with(|c: char| !c.is_whitespace()))
    });
    let checks = [
        ("title", titled),
        ("last-session", dated),
        ("sections", heads == SECTIONS),
    ];

    checks
        .into_iter()
        .filter(|(_, ok)| !ok)
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_handoff_holds_the_fixed_shape() {
        let body: Vec<String> = SECTIONS
            .iter()
            .map(|name| format!("## {name}\n\n- x\n"))
            .collect();
        let head = "# Handoff\n\nLast session: 2026-10-09T06:04Z\n";
        let doc = |head: &str, body: &[String]| format!("{head}\n{}", body.join("\n"));
        let mut reversed = body.clone();
        reversed.reverse();
        let mut extra = body.clone();
        extra.push("## Misc\n".to_string());
        let cases: Vec<(String, Vec<&str>)> = vec![
            (doc(head, &body), vec![]),
            (doc(head, &body).replace('\n', "\r\n"), vec![]),
            (doc("# Notes\n\nLast session: x\n", &body), vec!["title"]),
            (doc(&format!("intro\n{head}"), &body), vec!["title"]),
            (doc("# Handoff\n", &body), vec!["last-session"]),
            (doc(head, &body[1..]), vec!["sections"]),
            (doc(head, &reversed), vec!["sections"]),
            (doc(head, &extra), vec!["sections"]),
            (String::new(), vec!["title", "last-session", "sections"]),
        ];

        for (text, problems) in cases {
            assert_eq!(check_handoff(&text), problems, "{text:?}");
        }
    }

    #[test]
    fn read_status_takes_head_and_every_kind_of_change() {
        let status = [
            "# branch.oid abc123",
            "# branch.head main",
            "1 .M N... 100644 100644 100644 h1 h2 b/c d.txt",
            "2 R. N... 100644 100644 100644 h1 h2 R100 new.txt",
            "old.txt",
            "u UU N... 100644 100644 100644 100644 h1 h2 h3 both.txt",
            "? a.txt",
            "? nested/",
        ]
        .join("\0");
        let pair = |xy: &str, path: &str| (xy.to_string(), path.to_string());

        assert_eq!(
            read_status(&status),
            (
                "abc123".to_string(),
                vec![
                    pair(".M", "b/c d.txt"),
                    pair("R.", "new.txt"),
                    pair("UU", "both.txt"),
                    pair("??", "a.txt"),
                    pair("??", "nested/"),
                ]
            )
        );
        assert_eq!(
            read_status("# branch.oid (initial)\0"),
            ("(initial)".to_string(), vec![])
        );
    }

    #[test]
    fn fingerprints_are_fnv1a() {
        assert_eq!(fingerprint(b""), "cbf29ce484222325");
        assert_eq!(fingerprint(b"a"), "af63dc4c8601ec8c");
    }
}
