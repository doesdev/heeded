//! heeded: good habits, installed rather than remembered.
//!
//! One binary is both the CLI and the hook. The hook reads one event from
//! a harness (Claude Code or Cursor) on stdin and answers it.

pub mod cli;
pub mod clock;
pub mod commit;
pub mod consts;
pub mod gate;
pub mod git;
pub mod guard;
pub mod harness;
pub mod journal;
pub mod notes;
pub mod paths;
pub mod rework;
pub mod scaffold;
pub mod session;
pub mod settings;
pub mod snapshot;
pub mod store;
pub mod wizard;

use std::io::Read;
use std::path::Path;

use crate::consts::{home_dir, state_dir};
use crate::harness::{Emitted, Outcome, harness_for};
use crate::notes::marked;
use crate::store::Store;

/// Answers one hook event read from stdin.
pub fn hook() -> Result<Emitted, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|ex| ex.to_string())?;
    let input: serde_json::Value =
        serde_json::from_str(&text).map_err(|ex| format!("hook input: {ex}"))?;
    let harness = harness_for(&input);
    let env = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
    let event = harness.parse(&input, &env);
    let store = Store::new(state_dir());
    let id = event.id.as_deref().unwrap_or("undefined");

    // the way out for a repo that never opted in, before git is touched;
    // a session already under way still goes through, so deleting the
    // marker does not get one past the gate
    let outcome = match event.name.as_deref() {
        None => Outcome::default(),
        Some(_) if !marked(Path::new(&event.dir)) && !store.has(id) => Outcome::default(),
        Some("pre") => guard::Guard::new(&home_dir(), &store).check(&event),
        Some(_) => gate::handle(&store, &home_dir(), &event)?,
    };

    Ok(harness.emit(&event, &outcome))
}
