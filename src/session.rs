//! What a session owes: the pure part of the gate, with no files and no
//! git, so that every rule here can be tested from a table.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::consts::MARKER;
use crate::paths::{resolve, within};
use crate::snapshot::Snapshot;

pub const OUTSIDE: &str = "outside";
pub const UNTRACKED: &str = "untracked";

const ASK: &str = "Rewrite it before stopping, using the heeded:handoff skill: \
what this session did, what it left open, and what comes next.";
const LISTED: usize = 20;
const AGENT_TTL: f64 = 6.0 * 60.0 * 60.0 * 1000.0;

/// One move: a tool call that changed the repo.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Entry {
    pub tool: String,
    pub what: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub handoff: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct State {
    pub stamped: bool,
    pub last: Snapshot,
    pub pending: Vec<Entry>,
    pub blocks: u64,
    pub agents: BTreeMap<String, f64>,
    pub sides: BTreeMap<String, Option<String>>,
}

pub fn open(now: &Snapshot) -> State {
    State {
        stamped: now.handoff.is_some(),
        last: now.clone(),
        ..State::default()
    }
}

fn text(input: &Value, key: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn relative(root: &str, file: &str) -> String {
    if file.is_empty() {
        return String::new();
    }

    let here = std::env::current_dir().unwrap_or_default();

    within(Path::new(root), &resolve(&here, file))
}

// commands can carry secrets, so only the description is kept
pub fn to_move(tool: &str, input: &Value, root: &str) -> Entry {
    let what = match tool {
        "Edit" | "Write" => relative(root, &text(input, "file_path")),
        "NotebookEdit" => relative(root, &text(input, "notebook_path")),
        "Bash" | "PowerShell" => text(input, "description"),
        _ => String::new(),
    };

    Entry {
        tool: tool.to_string(),
        what,
        ..Entry::default()
    }
}

/// Compares a snapshot with the last one; a change is one journal entry.
/// `now.side` is a worktree a subagent is in: work there is this
/// session's too.
pub fn track(state: &State, now: &Snapshot, mv: Entry) -> (State, Option<Entry>) {
    let last = &state.last;
    let wrote = now.handoff.is_some() && now.handoff != last.handoff;
    let lost = now.handoff.is_none() && last.handoff.is_some();
    let aside = now
        .side
        .as_ref()
        .is_some_and(|side| state.sides.get(&side.root) != Some(&side.tree));
    let worked = lost || now.tree != last.tree || aside;
    let stamped = state.stamped || wrote;
    let mut next = State {
        stamped,
        last: now.clone(),
        ..state.clone()
    };

    if let Some(side) = &now.side {
        next.sides.insert(side.root.clone(), side.tree.clone());
    }
    if !stamped || (!wrote && !worked) {
        return (next, None);
    }

    let entry = Entry {
        handoff: wrote,
        ..mv
    };
    if wrote {
        next.pending.clear();
    } else if entry.tool != OUTSIDE {
        next.pending.push(entry.clone());
    }

    (next, Some(entry))
}

/// Subagents still out, ignoring any whose stop was never seen.
pub fn live(agents: &BTreeMap<String, f64>, at: f64) -> usize {
    agents.values().filter(|ts| at - **ts < AGENT_TTL).count()
}

pub fn decide(state: &State, now: &Snapshot, out: usize) -> (bool, &'static str) {
    if !state.stamped {
        return (false, "unstamped");
    }
    if now.notes.is_none() {
        return (true, "marker-missing");
    }
    if now.handoff.is_none() {
        return (true, "handoff-missing");
    }
    if !state.pending.is_empty() && out > 0 {
        return (false, "agents-out");
    }
    if !state.pending.is_empty() {
        return (true, "handoff-stale");
    }

    (false, "current")
}

/// The moves as a list, newest twenty, with a count of the rest.
pub fn list<'a>(moves: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let moves: Vec<_> = moves.into_iter().collect();
    let start = moves.len().saturating_sub(LISTED);
    let mut lines: Vec<String> = Vec::new();

    if start > 0 {
        lines.push(format!("- ({start} earlier)"));
    }
    for (tool, what) in &moves[start..] {
        let sep = if what.is_empty() { "" } else { " " };
        lines.push(format!("- {tool}{sep}{what}"));
    }

    lines.join("\n")
}

pub fn listed(entries: &[Entry]) -> String {
    list(
        entries
            .iter()
            .map(|entry| (entry.tool.as_str(), entry.what.as_str())),
    )
}

pub fn listed_values(entries: &[Value]) -> String {
    let field = |entry: &'_ Value, key: &str| {
        entry
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let pairs: Vec<(String, String)> = entries
        .iter()
        .map(|entry| (field(entry, "tool"), field(entry, "what")))
        .collect();

    list(
        pairs
            .iter()
            .map(|(tool, what)| (tool.as_str(), what.as_str())),
    )
}

pub fn reason_for(why: &str, file: &str, pending: &[Entry]) -> String {
    match why {
        "marker-missing" => format!(
            "{MARKER} is gone, and it is what opts this repo in to heeded. Put it back \
before stopping, unless the owner asked for this repo to opt out."
        ),
        "handoff-missing" => format!("{file} is gone. {ASK}"),
        _ => format!(
            "This session changed the repo after the last write to {file}. {ASK}\nMoves since:\n{}",
            listed(pending)
        ),
    }
}

pub fn notes_note(notes: &str) -> String {
    format!(
        "heeded: this repo's working notes are in {notes}. Before anything else read \
handoff.md there (what the last session did and left open) and plan.md (current \
intent). decisions/ holds one record per decision. `npm run check` is the oracle."
    )
}

/// The owner's own additions, from config; one line stays inline.
pub fn directives_note(directives: &[(String, String)]) -> String {
    if directives.is_empty() {
        return String::new();
    }

    let items: Vec<String> = directives
        .iter()
        .map(|(name, said)| {
            let sep = if said.contains('\n') { "\n" } else { " " };
            format!("- **{name}**:{sep}{said}")
        })
        .collect();

    format!(
        "heeded: the owner's standing directives, from their config. A repo's own \
instructions can add to these, not loosen them.\n{}",
        items.join("\n")
    )
}

pub fn leftover_note(file: &str, moves: &[Value]) -> String {
    format!(
        "heeded: earlier sessions changed this repo after {file} was last written, so \
it may not describe the current state. Moves since:\n{}",
        listed_values(moves)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Side;

    fn at(handoff: Option<&str>, tree: &str) -> Snapshot {
        Snapshot {
            handoff: handoff.map(str::to_string),
            tree: Some(tree.to_string()),
            notes: Some("n".to_string()),
            ..Snapshot::default()
        }
    }

    fn edit() -> Entry {
        Entry {
            tool: "Edit".into(),
            what: "a.js".into(),
            ..Entry::default()
        }
    }

    fn human() -> Entry {
        Entry {
            tool: OUTSIDE.into(),
            ..Entry::default()
        }
    }

    #[test]
    fn track_turns_snapshot_changes_into_journal_entries() {
        let cases = [
            (
                "nothing changed",
                at(Some("a"), "x"),
                at(Some("a"), "x"),
                edit(),
                None,
                0,
            ),
            (
                "work is owed a handoff",
                at(Some("a"), "x"),
                at(Some("a"), "y"),
                edit(),
                Some(false),
                1,
            ),
            (
                "a handoff write settles up",
                at(Some("a"), "x"),
                at(Some("b"), "x"),
                edit(),
                Some(true),
                0,
            ),
            (
                "both at once settles up",
                at(Some("a"), "x"),
                at(Some("b"), "y"),
                edit(),
                Some(true),
                0,
            ),
            (
                "losing the handoff is work",
                at(Some("a"), "x"),
                at(None, "x"),
                edit(),
                Some(false),
                1,
            ),
            (
                "human work is not owed",
                at(Some("a"), "x"),
                at(Some("a"), "y"),
                human(),
                Some(false),
                0,
            ),
            (
                "unstamped repos stay quiet",
                at(None, "x"),
                at(None, "y"),
                edit(),
                None,
                0,
            ),
            (
                "a new handoff stamps the repo",
                at(None, "x"),
                at(Some("a"), "x"),
                edit(),
                Some(true),
                0,
            ),
        ];

        for (name, last, now, mv, handoff, owed) in cases {
            let (state, entry) = track(&open(&last), &now, mv);

            assert_eq!(entry.map(|entry| entry.handoff), handoff, "{name}");
            assert_eq!(state.pending.len(), owed, "{name}");
            assert_eq!(state.last, now, "{name}");
        }
    }

    #[test]
    fn track_clears_what_was_owed_once_the_handoff_is_written() {
        let steps = [
            (at(Some("a"), "y"), 1),
            (at(Some("a"), "z"), 2),
            (at(Some("b"), "z"), 0),
            (at(Some("b"), "w"), 1),
        ];
        let mut state = open(&at(Some("a"), "x"));

        for (now, owed) in steps {
            state = track(&state, &now, edit()).0;
            assert_eq!(state.pending.len(), owed);
        }
    }

    #[test]
    fn track_sees_work_in_a_side_worktree() {
        let side = |tree: &str| {
            Some(Side {
                root: "w".into(),
                tree: Some(tree.into()),
            })
        };
        let mut now = at(Some("a"), "x");
        let state = open(&now);

        now.side = side("1");
        let (state, entry) = track(&state, &now, edit());
        assert!(entry.is_some());

        let (state, entry) = track(&state, &now, edit());
        assert!(entry.is_none());

        now.side = side("2");
        assert_eq!(track(&state, &now, edit()).0.pending.len(), 2);
    }

    #[test]
    fn decide_covers_every_state() {
        let owed = vec![Entry {
            handoff: false,
            ..edit()
        }];
        let state = |stamped: bool, pending: &Vec<Entry>| State {
            stamped,
            pending: pending.clone(),
            ..State::default()
        };
        let none = vec![];
        let missing = Snapshot {
            notes: None,
            ..at(Some("a"), "x")
        };
        let cases = [
            (state(false, &none), at(None, "x"), 0, false, "unstamped"),
            (
                state(true, &none),
                at(None, "x"),
                0,
                true,
                "handoff-missing",
            ),
            (
                state(true, &owed),
                at(Some("a"), "x"),
                0,
                true,
                "handoff-stale",
            ),
            (state(true, &none), at(Some("a"), "x"), 0, false, "current"),
            (
                state(true, &none),
                missing.clone(),
                0,
                true,
                "marker-missing",
            ),
            (state(false, &none), missing, 0, false, "unstamped"),
            (
                state(true, &owed),
                at(Some("a"), "x"),
                1,
                false,
                "agents-out",
            ),
            (state(true, &none), at(Some("a"), "x"), 2, false, "current"),
            (
                state(true, &owed),
                at(None, "x"),
                2,
                true,
                "handoff-missing",
            ),
        ];

        for (state, now, out, block, why) in cases {
            assert_eq!(decide(&state, &now, out), (block, why), "{why}");
        }
    }

    #[test]
    fn live_drops_agents_never_seen_to_stop() {
        let hour = 3_600_000.0;
        let at = 100.0 * hour;
        let agents = |ages: &[f64]| {
            ages.iter()
                .enumerate()
                .map(|(i, age)| (i.to_string(), at - age * hour))
                .collect()
        };

        assert_eq!(live(&BTreeMap::new(), at), 0);
        assert_eq!(live(&agents(&[1.0, 5.0]), at), 2);
        assert_eq!(live(&agents(&[1.0, 7.0]), at), 1);
    }

    #[test]
    fn to_move_names_files_and_never_keeps_a_command() {
        let root = if cfg!(windows) { "C:\\root" } else { "/root" };
        let file = format!("{root}/lib/a.js");
        let cases = [
            ("Edit", serde_json::json!({ "file_path": file }), "lib/a.js"),
            (
                "Write",
                serde_json::json!({ "file_path": file }),
                "lib/a.js",
            ),
            (
                "NotebookEdit",
                serde_json::json!({ "notebook_path": file }),
                "lib/a.js",
            ),
            (
                "Bash",
                serde_json::json!({ "command": "TOKEN=abc curl", "description": "Fetch it" }),
                "Fetch it",
            ),
            (
                "Bash",
                serde_json::json!({ "command": "TOKEN=abc curl" }),
                "",
            ),
            (
                "PowerShell",
                serde_json::json!({ "command": "x", "description": "Run it" }),
                "Run it",
            ),
            ("SomethingNew", serde_json::json!({ "anything": 1 }), ""),
            ("Edit", Value::Null, ""),
        ];

        for (tool, input, what) in cases {
            assert_eq!(to_move(tool, &input, root).what, what, "{tool}");
        }
    }

    #[test]
    fn list_shows_the_newest_twenty() {
        let moves: Vec<(String, String)> = (0..22)
            .map(|i| ("Edit".to_string(), format!("{i}.txt")))
            .collect();
        let shown = list(moves.iter().map(|(a, b)| (a.as_str(), b.as_str())));

        assert!(shown.starts_with("- (2 earlier)\n- Edit 2.txt"));
        assert!(shown.ends_with("- Edit 21.txt"));
        assert_eq!(list([("Bash", "")]), "- Bash");
    }
}
