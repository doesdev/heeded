use std::path::{Path, PathBuf};

use crate::consts::{MARKER, NOTES};
use crate::paths::resolve;

// the marker as JSON; `Ok(None)` when there is none
fn marker_of(root: &Path) -> Result<Option<serde_json::Value>, String> {
    let file = root.join(MARKER);
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(ex) if ex.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(ex) => return Err(format!("{}: {ex}", file.display())),
    };

    Ok(Some(
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null),
    ))
}

fn folder(marker: &serde_json::Value, key: &str) -> Option<String> {
    marker
        .get(key)?
        .as_str()
        .filter(|dir| !dir.is_empty())
        .map(str::to_string)
}

/// The notes folder a repo's marker names. `Ok(None)` when there is no
/// marker; an error when there is one and it cannot be read.
pub fn notes_of(root: &Path) -> Result<Option<PathBuf>, String> {
    let Some(marker) = marker_of(root)? else {
        return Ok(None);
    };

    match folder(&marker, "notes") {
        Some(notes) => Ok(Some(resolve(root, &notes))),
        None => Err(format!(
            "{} must be JSON like {{ \"notes\": \"{NOTES}\" }}",
            root.join(MARKER).display()
        )),
    }
}

/// Where the handoff and journal live: the marker's `privateNotes`, else
/// the notes folder.
pub fn private_of(root: &Path) -> Result<Option<PathBuf>, String> {
    let Some(notes) = notes_of(root)? else {
        return Ok(None);
    };
    let marker = marker_of(root)?.unwrap_or_default();

    match marker.get("privateNotes") {
        None => Ok(Some(notes)),
        Some(_) => match folder(&marker, "privateNotes") {
            Some(private) => Ok(Some(resolve(root, &private))),
            None => Err(format!(
                "{}: privateNotes must name a folder",
                root.join(MARKER).display()
            )),
        },
    }
}

/// The private folder, when the marker sets one apart from the notes.
pub fn apart_of(root: &Path) -> Option<PathBuf> {
    let notes = notes_of(root).ok().flatten()?;

    private_of(root)
        .ok()
        .flatten()
        .filter(|private| *private != notes)
}

/// Is there a marker in this folder or any folder above it.
pub fn marked(dir: &Path) -> bool {
    dir.ancestors().any(|at| at.join(MARKER).exists())
}
