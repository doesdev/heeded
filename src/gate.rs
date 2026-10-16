//! The gate, with no idea which harness is asking: an event comes in, and
//! what to say comes out.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::clock::{iso, now_ms};
use crate::consts::{JOURNAL_MAX, MAX_BLOCKS};
use crate::harness::{Event, Outcome};
use crate::journal::{Journal, unhanded};
use crate::session::{
    Entry, OUTSIDE, State, UNTRACKED, decide, directives_note, leftover_note, live, notes_note,
    open, reason_for, to_move, track,
};
use crate::settings::{directives_of, settings_for};
use crate::snapshot::{Snapshot, checkout_of, moved, ours_of, project_side, snapshot};
use crate::store::Store;

fn plain(tool: &str) -> Entry {
    Entry {
        tool: tool.to_string(),
        ..Entry::default()
    }
}

/// One event's view of one session: the repo as it is now, and where the
/// session's state and journal live.
struct Turn<'a> {
    store: &'a Store,
    home: &'a Path,
    id: String,
    session: Option<String>,
    checkout: Option<String>,
    project: &'a Path,
    journal: Journal,
    now: Snapshot,
}

impl Turn<'_> {
    fn load(&self) -> State {
        self.store
            .load(&self.id)
            .and_then(|saved| serde_json::from_value(saved).ok())
            .unwrap_or_else(|| self.first())
    }

    // a session kept in a worktree starts knowing how the project's own
    // checkout looks, so only a change there is its work
    fn first(&self) -> State {
        let mut state = State {
            checkout: self.checkout.clone(),
            ..open(&self.now)
        };
        if self.checkout.is_some() {
            let side = project_side(self.project);
            state.sides.insert(side.root, side.tree);
        }

        state
    }

    fn save(&self, state: &State) {
        self.store.save(
            &self.id,
            &serde_json::to_value(state).unwrap_or(Value::Null),
        );
    }

    fn log(&self, entry: &Entry) {
        let mut line = Map::new();
        line.insert("ts".into(), json!(iso(now_ms())));
        if let Some(session) = &self.session {
            line.insert("session".into(), json!(session));
        }
        if let Ok(Value::Object(fields)) = serde_json::to_value(entry) {
            line.extend(fields);
        }

        self.journal.append(&Value::Object(line));
    }

    fn sync(&self, mv: Entry) -> State {
        let state = self.load();
        let (last, now) = (&state.last, &self.now);
        let moved = moved(
            Path::new(&now.root),
            last.head.as_deref(),
            now.head.as_deref(),
            &ours_of(now),
        );
        let (state, entry) = track(&state, now, mv, moved);

        if let Some(entry) = entry.filter(|_| self.now.journal.is_some()) {
            self.log(&entry);
        }
        self.save(&state);

        state
    }

    fn start(&self) -> Outcome {
        let state = self.sync(plain(OUTSIDE));
        let Some(notes) = self.now.notes.as_deref().filter(|_| state.stamped) else {
            return Outcome::default();
        };

        self.journal.trim(JOURNAL_MAX);
        let left = unhanded(&self.journal.read(), self.session.as_deref());
        let file = self.now.file.as_deref().unwrap_or("");
        let settings = settings_for(Path::new(&self.now.root), self.home);
        let private = Path::new(file)
            .parent()
            .map(|dir| dir.display().to_string())
            .filter(|dir| dir != notes);
        let parts = [
            notes_note(notes, private.as_deref()),
            directives_note(&directives_of(&settings)),
            if left.is_empty() {
                String::new()
            } else {
                leftover_note(file, &left)
            },
        ];
        let context: Vec<String> = parts.into_iter().filter(|part| !part.is_empty()).collect();

        Outcome {
            context: Some(context.join("\n")),
            ..Outcome::default()
        }
    }

    fn stop(&self) -> Outcome {
        let state = self.sync(plain(UNTRACKED));
        let (block, why) = decide(&state, &self.now, live(&state.agents, now_ms() as f64));

        if !block {
            if state.blocks > 0 {
                self.save(&State { blocks: 0, ..state });
            }
            return Outcome::default();
        }

        let file = self.now.file.as_deref();
        if state.blocks >= MAX_BLOCKS {
            let notice = format!(
                "heeded: stopped without a handoff after {MAX_BLOCKS} refusals ({why}). {} is out of date.",
                file.unwrap_or("It")
            );
            return Outcome {
                notice: Some(notice),
                ..Outcome::default()
            };
        }

        let refusal = reason_for(why, file.unwrap_or("undefined"), &state.pending);
        self.save(&State {
            blocks: state.blocks + 1,
            ..state
        });

        Outcome {
            refusal: Some(refusal),
            ..Outcome::default()
        }
    }

    // a subagent's moves arrive under this session's id, so all that is
    // kept is whether it is still out, and where its worktree started
    fn agent(&self, agent: &str, out: bool) -> Outcome {
        let mut state = self.load();

        if let Some(side) = &self.now.side {
            state.sides.insert(side.root.clone(), side.tree.clone());
        }
        if out {
            state.agents.insert(agent.to_string(), now_ms() as f64);
        } else {
            state.agents.remove(agent);
        }
        self.save(&state);

        Outcome::default()
    }
}

// the checkout a session keeps its notes in, fixed when it is first seen:
// a session started in a stamped worktree of the project (the project is
// the event's folder, the worktree its cwd) writes that worktree's handoff,
// which its branch carries, and leaves the project's alone
fn checkout_for(store: &Store, id: &str, event: &Event) -> Option<String> {
    if let Some(saved) = store.load(id) {
        return saved
            .get("checkout")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    if event.agent.is_some() {
        return None;
    }

    checkout_of(Path::new(&event.dir), event.cwd.as_deref()).map(|side| side.display().to_string())
}

pub fn handle(store: &Store, home: &Path, event: &Event) -> Result<Outcome, String> {
    let id = event.id.clone().unwrap_or_else(|| "undefined".to_string());
    let checkout = checkout_for(store, &id, event);
    let dir = checkout.as_deref().unwrap_or(&event.dir);
    let now = snapshot(Path::new(dir), event.cwd.as_deref())?;
    let turn = Turn {
        store,
        home,
        id,
        checkout,
        project: Path::new(&event.dir),
        session: event.id.clone(),
        journal: Journal::new(now.journal.as_ref().map(PathBuf::from)),
        now,
    };
    let agent = event.agent.as_deref().unwrap_or("undefined");

    Ok(match event.name.as_deref() {
        Some("start") => turn.start(),
        Some("prompt") => {
            turn.sync(plain(OUTSIDE));
            Outcome::default()
        }
        Some("tool") => {
            let tool = event.tool.as_deref().unwrap_or("");
            let mv = to_move(tool, &event.input, &turn.now.root);
            turn.sync(Entry {
                agent: event.agent.clone(),
                ..mv
            });
            Outcome::default()
        }
        Some("agent-start") => turn.agent(agent, true),
        Some("agent-stop") => turn.agent(agent, false),
        Some("stop") => turn.stop(),
        _ => Outcome::default(),
    })
}
