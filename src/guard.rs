//! Before a tool runs: refuses a commit that breaks a rule, and asks the
//! owner before a commit, a push or a settings change the settings do not
//! allow. Only in a stamped repo.

use std::path::Path;

use serde_json::{Value, json};

use crate::commit::{
    CHANGELOG, Identity, attribution_problem, changelog_owed, classify, identity_problem,
    scan_diff, secret_file, secret_in, stages, target_of,
};
use crate::consts::MARKER;
use crate::git::{git, root_and_common};
use crate::harness::{Event, Outcome};
use crate::notes::notes_of;
use crate::paths::{clean, resolve, within};
use crate::settings::{get_str, settings_files, settings_for};
use crate::snapshot::fingerprint;
use crate::store::Store;

const SMALL: u64 = 256 * 1024;

fn lines(text: Option<String>) -> Vec<String> {
    text.unwrap_or_default()
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn config(where_: &Path, key: &str) -> String {
    git(where_, &["config", key])
        .unwrap_or_default()
        .trim()
        .to_string()
}

struct Incoming {
    files: Vec<String>,
    diffs: Vec<String>,
    fresh: Vec<String>,
}

// what the commit will hold: the index, plus the working tree when the
// command stages for itself
fn incoming(where_: &Path, wide: bool) -> Incoming {
    let staged = lines(git(where_, &["diff", "--cached", "--name-only"]));
    let cached = git(where_, &["diff", "--cached", "-U0"]).unwrap_or_default();

    if !wide {
        return Incoming {
            files: staged,
            diffs: vec![cached],
            fresh: vec![],
        };
    }

    let changed = lines(git(where_, &["diff", "--name-only"]));
    let fresh = lines(git(where_, &["ls-files", "--others", "--exclude-standard"]));
    let mut files: Vec<String> = Vec::new();
    for file in staged.into_iter().chain(changed).chain(fresh.clone()) {
        if !files.contains(&file) {
            files.push(file);
        }
    }
    let diffs = vec![cached, git(where_, &["diff", "-U0"]).unwrap_or_default()];

    Incoming {
        files,
        diffs,
        fresh,
    }
}

fn scan_file(where_: &Path, file: &str) -> Vec<(String, &'static str)> {
    let full = where_.join(file);
    let small = std::fs::metadata(&full).is_ok_and(|meta| meta.len() <= SMALL);
    let text = if small {
        std::fs::read_to_string(&full).unwrap_or_default()
    } else {
        String::new()
    };

    text.split('\n')
        .filter_map(secret_in)
        .map(|kind| (file.to_string(), kind))
        .collect()
}

fn secrets_in(where_: &Path, coming: &Incoming) -> Vec<String> {
    let named = coming
        .files
        .iter()
        .filter(|file| secret_file(file))
        .map(|file| (file.clone(), "a credentials file"));
    let in_diffs = coming.diffs.iter().flat_map(|diff| scan_diff(diff));
    let in_fresh = coming.fresh.iter().flat_map(|file| scan_file(where_, file));
    let mut all: Vec<String> = Vec::new();

    for (file, kind) in named.chain(in_diffs).chain(in_fresh) {
        let line = format!("{file}: {kind}");
        if !all.contains(&line) {
            all.push(line);
        }
    }

    all
}

pub struct Guard<'a> {
    home: &'a Path,
    store: &'a Store,
}

impl<'a> Guard<'a> {
    pub fn new(home: &'a Path, store: &'a Store) -> Self {
        Guard { home, store }
    }

    fn nagged(&self, id: &str, files: &[String]) -> bool {
        let key = fingerprint(files.join("\n").as_bytes());
        let name = format!("{id}-guard");
        let mut state = self
            .store
            .load(&name)
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));

        if state
            .get(&key)
            .is_some_and(|seen| seen.as_bool() == Some(true))
        {
            return true;
        }

        state[&key] = json!(true);
        self.store.save(&name, &state);

        false
    }

    fn commit(
        &self,
        id: &str,
        root: &Path,
        where_: &Path,
        command: &str,
        settings: &Value,
    ) -> Option<String> {
        let coming = incoming(where_, stages(command));
        let (name, email) = (config(where_, "user.name"), config(where_, "user.email"));
        let actual = Identity {
            name: &name,
            email: &email,
        };
        let wanted = Identity {
            name: get_str(settings, "identity", "name"),
            email: get_str(settings, "identity", "email"),
        };
        let secrets = secrets_in(where_, &coming);
        let problems: Vec<String> = [
            identity_problem(command, &actual, &wanted),
            attribution_problem(command, get_str(settings, "identity", "attribution")),
            (!secrets.is_empty())
                .then(|| format!("what looks like a secret:\n{}", secrets.join("\n"))),
        ]
        .into_iter()
        .flatten()
        .collect();

        if !problems.is_empty() {
            return Some(format!(
                "heeded: this commit has {}",
                problems.join("; and ")
            ));
        }
        if settings["checks"]["changelog"].as_bool() != Some(true) {
            return None;
        }

        let inside = notes_of(root)
            .ok()
            .flatten()
            .map(|notes| within(root, &notes))
            .unwrap_or_default();
        let notes_at = if inside.starts_with("..") {
            ""
        } else {
            inside.as_str()
        };
        let quiet: Vec<String> = settings["checks"]["quiet"]
            .as_array()
            .map(|all| {
                all.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        if !changelog_owed(&coming.files, notes_at, &quiet) || self.nagged(id, &coming.files) {
            return None;
        }

        Some(format!(
            "heeded: this commit does not touch {CHANGELOG}. If it changes anything a user \
can see, do or be affected by, add a line under \"## [Unreleased]\" first. If it does not, \
run the same command again."
        ))
    }

    pub fn check(&self, event: &Event) -> Outcome {
        // the nearest marker, found without git: most tool calls end here
        let dir = clean(Path::new(&event.dir));
        let Some(root) = dir.ancestors().find(|at| at.join(MARKER).exists()) else {
            return Outcome::default();
        };
        let root = root.to_path_buf();

        let tool = event.tool.as_deref().unwrap_or("");
        if tool == "Edit" || tool == "Write" {
            let here = std::env::current_dir().unwrap_or_default();
            let file = resolve(
                &here,
                event
                    .input
                    .get("file_path")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            );
            let ours = settings_files(&root, self.home);

            return if ours
                .iter()
                .any(|own| resolve(&here, &own.display().to_string()) == file)
            {
                Outcome::ask(format!(
                    "heeded: {} sets what needs the owner's OK. Changing it needs it too.",
                    file.display()
                ))
            } else {
                Outcome::default()
            };
        }

        let command = event
            .input
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("");
        let acts = classify(command);
        if acts.is_empty() {
            return Outcome::default();
        }

        // a git -C or cd into another repo is that repo's business, if it is stamped
        let cwd = event
            .cwd
            .as_deref()
            .filter(|cwd| !cwd.is_empty())
            .map_or(root.clone(), Into::into);
        let where_ = target_of(command, &cwd);
        // a linked worktree of a stamped repo is that repo: a hidden marker
        // never reaches the worktree, so it cannot carry one of its own
        let (repo, common) = root_and_common(&where_);
        let stamped = if repo == root || repo.join(MARKER).exists() {
            Some(repo.clone())
        } else {
            common
                .as_deref()
                .and_then(Path::parent)
                .filter(|main| main.join(MARKER).exists())
                .map(Path::to_path_buf)
        };
        let own: Vec<&str> = acts
            .into_iter()
            .filter(|act| stamped.is_some() || *act == "settings")
            .collect();
        if own.is_empty() {
            return Outcome::default();
        }

        let repo = stamped.unwrap_or(repo);
        let settings = settings_for(&repo, self.home);
        let id = event.id.as_deref().unwrap_or("undefined");
        if own.contains(&"commit")
            && let Some(refusal) = self.commit(id, &repo, &where_, command, &settings)
        {
            return Outcome::deny(refusal);
        }

        let what = |act: &str| match act {
            "commit" => "a commit",
            "push" => "something that leaves the machine",
            _ => "changing heeded settings",
        };
        let asks: Vec<&str> = own
            .into_iter()
            .filter(|act| {
                *act == "settings" || settings["permission"][*act].as_str() != Some("allow")
            })
            .map(what)
            .collect();

        if asks.is_empty() {
            Outcome::default()
        } else {
            Outcome::ask(format!(
                "heeded: {} needs the owner's OK here.",
                asks.join(" and ")
            ))
        }
    }
}
