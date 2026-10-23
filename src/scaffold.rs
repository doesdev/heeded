//! `heeded init`: stamps a repo without overwriting anything.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::consts::{HANDOFF, MARKER, NOTES};
use crate::git::git;
use crate::notes::{notes_of, private_of};
use crate::paths::{resolve, within};

const MANIFEST: &str = "package.json";
const EXCLUDE: &str = "info/exclude";
const IGNORE_ALL: &str = "*\n";

// the templates folder, built in; sorted the way a directory walk sorts them
const NOTES_TEMPLATES: [(&str, &str); 5] = [
    (".gitignore", include_str!("../templates/notes/.gitignore")),
    (
        "decisions/README.md",
        include_str!("../templates/notes/decisions/README.md"),
    ),
    ("handoff.md", include_str!("../templates/notes/handoff.md")),
    ("plan.md", include_str!("../templates/notes/plan.md")),
    (
        "rework.jsonl",
        include_str!("../templates/notes/rework.jsonl"),
    ),
];
const ROOT_TEMPLATES: [(&str, &str); 1] =
    [(MANIFEST, include_str!("../templates/root/package.json"))];

pub struct Stamped {
    pub notes: PathBuf,
    pub private: PathBuf,
    pub files: Vec<(String, &'static str)>,
    pub check: bool,
}

// never overwrites: a second run, or a repo with its own files, loses nothing
fn create(dest: &Path, text: &str) -> Result<&'static str, String> {
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).map_err(|ex| format!("{}: {ex}", dir.display()))?;
    }

    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
    {
        Ok(mut out) => {
            out.write_all(text.as_bytes())
                .map_err(|ex| format!("{}: {ex}", dest.display()))?;
            Ok("created")
        }
        Err(ex) if ex.kind() == std::io::ErrorKind::AlreadyExists => Ok("kept"),
        Err(ex) => Err(format!("{}: {ex}", dest.display())),
    }
}

fn stamp(templates: &[(&str, &str)], dir: &Path) -> Result<Vec<(PathBuf, &'static str)>, String> {
    templates
        .iter()
        .map(|(file, text)| {
            let dest = dir.join(file);
            create(&dest, text).map(|did| (dest, did))
        })
        .collect()
}

fn has_check(root: &Path) -> bool {
    std::fs::read_to_string(root.join(MANIFEST))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|manifest| manifest.get("scripts")?.get("check").cloned())
        .is_some_and(|check| check.as_str().is_some_and(|check| !check.is_empty()))
}

// private by default: this clone's git stops seeing them, .gitignore is not touched
fn hide(root: &Path, patterns: &[String]) -> Result<Vec<(PathBuf, &'static str)>, String> {
    let Some(found) = git(root, &["rev-parse", "--git-path", EXCLUDE]) else {
        return Ok(vec![]);
    };
    let drive = |pattern: &str| pattern.as_bytes().get(2) == Some(&b':');
    let wanted: Vec<&String> = patterns
        .iter()
        .filter(|pattern| !pattern.starts_with("/..") && !drive(pattern))
        .filter(|pattern| git(root, &["check-ignore", "-q", &pattern[1..]]).is_none())
        .collect();

    if wanted.is_empty() {
        return Ok(vec![]);
    }

    let file = resolve(root, found.trim());
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let lines: Vec<&str> = wanted.iter().map(|pattern| pattern.as_str()).collect();
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .map_err(|ex| format!("{}: {ex}", file.display()))?;
    writeln!(out, "{}", lines.join("\n")).map_err(|ex| format!("{}: {ex}", file.display()))?;

    Ok(vec![(file, "updated")])
}

/// An existing marker wins over the notes folder asked for, and only what
/// this run created is hidden: what was already there is the repo's choice.
/// A `package.json` is only stamped while the oracle is `npm run check`.
pub fn scaffold(
    root: &Path,
    asked: Option<&str>,
    private: Option<&str>,
    tracked: bool,
    npm: bool,
) -> Result<Stamped, String> {
    let marker = root.join(MARKER);
    let mut wanted = serde_json::json!({ "notes": asked.unwrap_or(NOTES) });
    if let Some(private) = private {
        wanted["privateNotes"] = private.into();
    }
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&wanted).unwrap_or_default()
    );
    let marked = create(&marker, &text)?;
    let notes = notes_of(root)?.unwrap_or_else(|| root.join(NOTES));
    let private = private_of(root)?.unwrap_or_else(|| notes.clone());
    let apart = private != notes;
    // a new private folder ignores itself, in every clone and without a
    // line in the repo's .gitignore; one already there is left as it is
    let sealed = apart && !private.exists() && !within(root, &private).starts_with("..");
    let fresh: Vec<String> = [
        (marked == "created").then(|| format!("/{MARKER}")),
        (!notes.exists()).then(|| format!("/{}/", within(root, &notes))),
        (npm && !root.join(MANIFEST).exists()).then(|| format!("/{MANIFEST}")),
    ]
    .into_iter()
    .flatten()
    .collect();

    let mut stamped = vec![(marker, marked)];
    let (handoff, rest): (Vec<_>, Vec<_>) = NOTES_TEMPLATES
        .into_iter()
        .partition(|(file, _)| apart && *file == HANDOFF);
    stamped.extend(stamp(&rest, &notes)?);
    if sealed {
        stamped.extend(stamp(&[(".gitignore", IGNORE_ALL)], &private)?);
    }
    stamped.extend(stamp(&handoff, &private)?);
    if npm {
        stamped.extend(stamp(&ROOT_TEMPLATES, root)?);
    }
    if !tracked {
        stamped.extend(hide(root, &fresh)?);
    }

    Ok(Stamped {
        files: stamped
            .into_iter()
            .map(|(file, did)| (within(root, &file), did))
            .collect(),
        check: !npm || has_check(root),
        notes,
        private,
    })
}
