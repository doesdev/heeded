use std::path::{Path, PathBuf};

use crate::consts::{MARKER, NOTES};
use crate::paths::resolve;

/// The notes folder a repo's marker names. `Ok(None)` when there is no
/// marker; an error when there is one and it cannot be read.
pub fn notes_of(root: &Path) -> Result<Option<PathBuf>, String> {
    let file = root.join(MARKER);
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(ex) if ex.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(ex) => return Err(format!("{}: {ex}", file.display())),
    };

    let notes = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|marker| marker.get("notes")?.as_str().map(str::to_string))
        .filter(|notes| !notes.is_empty());

    match notes {
        Some(notes) => Ok(Some(resolve(root, &notes))),
        None => Err(format!(
            "{} must be JSON like {{ \"notes\": \"{NOTES}\" }}",
            file.display()
        )),
    }
}

/// Is there a marker in this folder or any folder above it.
pub fn marked(dir: &Path) -> bool {
    dir.ancestors().any(|at| at.join(MARKER).exists())
}
