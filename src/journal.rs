use std::io::Write;
use std::path::PathBuf;

use serde_json::Value;

use crate::session::OUTSIDE;

/// An append-only JSON-lines file; unreadable lines are skipped.
pub struct Journal {
    file: Option<PathBuf>,
}

impl Journal {
    pub fn new(file: Option<PathBuf>) -> Self {
        Journal { file }
    }

    fn lines(&self) -> Vec<String> {
        let Some(file) = &self.file else {
            return vec![];
        };
        let text = std::fs::read_to_string(file).unwrap_or_default();

        text.split('\n')
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    }

    pub fn read(&self) -> Vec<Value> {
        self.lines()
            .iter()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    pub fn append(&self, entry: &Value) {
        let Some(file) = &self.file else { return };
        let opened = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file);

        if let Ok(mut out) = opened {
            let _ = writeln!(out, "{entry}");
        }
    }

    pub fn trim(&self, max: usize) {
        let (Some(file), all) = (&self.file, self.lines()) else {
            return;
        };
        if all.len() <= max {
            return;
        }

        let _ = std::fs::write(file, format!("{}\n", all[all.len() - max..].join("\n")));
    }
}

fn handed(entry: &Value) -> bool {
    entry
        .get("handoff")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// Everything after the last handoff write.
pub fn since_handoff(entries: &[Value]) -> &[Value] {
    let start = entries.iter().rposition(handed).map_or(0, |last| last + 1);

    &entries[start..]
}

/// Moves other sessions made after the handoff was last written.
pub fn unhanded(entries: &[Value], session: Option<&str>) -> Vec<Value> {
    since_handoff(entries)
        .iter()
        .filter(|entry| entry.get("tool").and_then(Value::as_str) != Some(OUTSIDE))
        .filter(|entry| entry.get("session").and_then(Value::as_str) != session)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn unhanded_returns_other_sessions_moves_since_the_last_handoff() {
        let mv = |session: &str, tool: &str, handoff: bool| json!({ "session": session, "tool": tool, "handoff": handoff });
        let entries = vec![
            mv("a", "Edit", false),
            mv("a", "Edit", true),
            mv("a", "Bash", false),
            mv("a", OUTSIDE, false),
            mv("b", "Edit", false),
        ];

        assert_eq!(unhanded(&entries, Some("b")), vec![entries[2].clone()]);
        assert_eq!(
            unhanded(&entries, Some("c")),
            vec![entries[2].clone(), entries[4].clone()]
        );
        assert!(unhanded(&entries[..2], Some("c")).is_empty());
        assert!(unhanded(&[], Some("c")).is_empty());
        assert_eq!(since_handoff(&entries), &entries[2..]);
        assert_eq!(since_handoff(&entries[..1]), &entries[..1]);
    }

    #[test]
    fn appends_skips_junk_and_trims_to_the_newest() {
        let file =
            std::env::temp_dir().join(format!("heeded-journal-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&file);
        let journal = Journal::new(Some(file.clone()));

        assert!(journal.read().is_empty());
        journal.trim(2);
        assert!(!file.exists());

        std::fs::write(&file, "{\"n\":1}\nnot json\n").unwrap();
        journal.append(&json!({ "n": 2 }));
        journal.append(&json!({ "n": 3 }));
        assert_eq!(
            journal.read(),
            vec![json!({ "n": 1 }), json!({ "n": 2 }), json!({ "n": 3 })]
        );

        journal.trim(2);
        assert_eq!(journal.read(), vec![json!({ "n": 2 }), json!({ "n": 3 })]);
        let _ = std::fs::remove_file(file);
    }
}
