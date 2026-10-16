//! The library's parts that touch files and git, against real ones.

mod common;

use std::path::Path;

use common::*;
use heeded::harness::{Claude, Cursor, Event, Harness, Outcome};
use heeded::notes::{notes_of, private_of};
use heeded::settings::{defaults, directives_of, settings_for};
use heeded::snapshot::snapshot;
use serde_json::{Value, json};

#[test]
fn snapshot_sees_every_kind_of_work_and_ignores_our_own_files() {
    let lab = Lab::new("snapshot");
    let repo = lab.repo(true);
    let tree = || snapshot(&repo, None).unwrap().tree;
    let mut trees = vec![tree()];
    let steps: Vec<Box<dyn Fn()>> = vec![
        Box::new(|| write(&repo, "new.txt", "one\n")),
        Box::new(|| write(&repo, "new.txt", "two\n")),
        Box::new(|| write(&repo, "README.md", "changed\n")),
        Box::new(|| git(&repo, &["add", "-A"])),
        Box::new(|| git(&repo, &["commit", "-q", "-m", "Work"])),
        Box::new(|| git(&repo, &["mv", "new.txt", "renamed.txt"])),
        Box::new(|| std::fs::remove_file(repo.join("README.md")).unwrap()),
    ];

    for step in steps {
        step();
        trees.push(tree());
    }
    let mut unique = trees.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), trees.len());

    let before = snapshot(&repo, None).unwrap();
    write(&repo, HANDOFF, "# Handoff\n\nDid things.\n");
    write(&repo, JOURNAL, "{}\n");
    let after = snapshot(&repo, None).unwrap();
    assert_eq!(after.tree, before.tree);
    assert_ne!(after.handoff, before.handoff);
    assert_eq!(after.root, repo.display().to_string());
}

#[test]
fn snapshot_outside_a_git_repo_has_no_tree() {
    let lab = Lab::new("nogit");
    let dir = lab.dir("nogit");
    write(&dir, MARKER, MARKED);
    write(&dir, HANDOFF, "# Handoff\n");
    let snap = snapshot(&dir, None).unwrap();

    assert_eq!(snap.tree, None);
    assert!(snap.handoff.is_some());
}

#[test]
fn notes_of_reads_the_marker_and_refuses_a_broken_one() {
    let lab = Lab::new("marker");
    let dir = lab.dir("marker");
    let elsewhere = lab.tmp.join("elsewhere");
    assert_eq!(notes_of(&dir).unwrap(), None);

    let good = [
        (MARKED.to_string(), dir.join(NOTES)),
        (
            "{ \"notes\": \"private/checkout/notes\" }".to_string(),
            dir.join("private").join("checkout").join("notes"),
        ),
        ("{ \"notes\": \".\" }".to_string(), dir.clone()),
        (
            json!({ "notes": elsewhere.display().to_string() }).to_string(),
            elsewhere.clone(),
        ),
    ];
    for (text, notes) in good {
        write(&dir, MARKER, &text);
        assert_eq!(notes_of(&dir).unwrap(), Some(notes), "{text}");
    }
    for text in [
        "",
        "not json",
        "null",
        "{}",
        "{ \"notes\": \"\" }",
        "{ \"notes\": 3 }",
    ] {
        write(&dir, MARKER, text);
        let ex = notes_of(&dir).unwrap_err();
        assert!(
            ex.ends_with("must be JSON like { \"notes\": \".heeded\" }"),
            "{text}: {ex}"
        );
    }
}

#[test]
fn private_of_falls_back_to_the_notes_and_refuses_a_broken_one() {
    let lab = Lab::new("private-of");
    let dir = lab.dir("marker");
    assert_eq!(private_of(&dir).unwrap(), None);

    let good = [
        (MARKED, dir.join(NOTES)),
        (
            "{ \"notes\": \"docs\", \"privateNotes\": \".private\" }",
            dir.join(".private"),
        ),
    ];
    for (text, private) in good {
        write(&dir, MARKER, text);
        assert_eq!(private_of(&dir).unwrap(), Some(private), "{text}");
    }
    for text in [
        "{ \"notes\": \"docs\", \"privateNotes\": \"\" }",
        "{ \"notes\": \"docs\", \"privateNotes\": null }",
        "{ \"notes\": \"docs\", \"privateNotes\": 3 }",
    ] {
        write(&dir, MARKER, text);
        let ex = private_of(&dir).unwrap_err();
        assert!(
            ex.ends_with("privateNotes must name a folder"),
            "{text}: {ex}"
        );
    }
}

#[test]
fn settings_layer_the_repo_over_the_user_over_the_defaults() {
    let lab = Lab::new("settings");
    let home = lab.tmp.join("home-1");
    let repo = lab.dir("settings");
    assert_eq!(settings_for(&repo, &home), defaults());

    write(&home, "config.json", &json!({ "permission": { "commit": "allow" }, "identity": { "email": "a@c.dev" }, "junk": 1 }).to_string());
    write(
        &repo,
        MARKER,
        &json!({ "notes": "n", "permission": { "push": "allow", "commit": "ask" } }).to_string(),
    );
    assert_eq!(
        settings_for(&repo, &home),
        json!({
            "permission": { "commit": "ask", "push": "allow" },
            "identity": { "name": "", "email": "a@c.dev", "attribution": "any" },
            "checks": { "changelog": false, "quiet": [] },
            "directives": {}
        })
    );

    write(&home, "config.json", "not json");
    assert_eq!(
        settings_for(&repo, &home)["identity"],
        defaults()["identity"]
    );
}

#[test]
fn directives_layer_by_name_come_from_text_or_a_file_and_can_be_switched_off() {
    let lab = Lab::new("directives-layer");
    let home = lab.tmp.join("home-1");
    let repo = lab.dir("directed");
    let long = "x".repeat(4050);
    write(&home, "voice.md", "Short sentences.\nNo filler.\n");
    write(&home, "long.md", &long);
    write(&repo, "notes/style.md", "Tabs here.\n");
    write(&lab.tmp, "outside.md", "not for you\n");
    let user = json!({ "directives": {
        "voice": { "file": "voice.md" }, "style": "Two spaces.", "tone": "Dry.",
        "long": { "file": "long.md" }, "blank": "  ", "junk": 7
    }});
    let marker = json!({ "notes": "n", "directives": {
        "style": { "file": "notes/style.md" }, "tone": false, "missing": { "file": "nope.md" },
        "escape": { "file": "../outside.md" }, "absolute": { "file": home.join("voice.md").display().to_string() }
    }});
    write(&home, "config.json", &user.to_string());
    write(&repo, MARKER, &marker.to_string());

    let said = directives_of(&settings_for(&repo, &home));
    let names: Vec<&str> = said.iter().map(|(name, _)| name.as_str()).collect();
    let text = |name: &str| said.iter().find(|(own, _)| own == name).unwrap().1.clone();

    assert_eq!(names, ["voice", "style", "long", "missing"]);
    assert_eq!(text("voice"), "Short sentences.\nNo filler.");
    assert_eq!(text("style"), "Tabs here.");
    assert!(text("long").ends_with("(cut at 4000 characters)"));
    assert!(text("missing").starts_with("(heeded could not read "));
    let alone: Vec<String> = directives_of(&settings_for(&repo, &lab.tmp.join("no-such-home")))
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(alone, ["style", "missing"]);
    assert!(directives_of(&defaults()).is_empty());
}

type Vars = &'static [(&'static str, &'static str)];

fn env(pairs: Vars) -> impl Fn(&str) -> Option<String> {
    move |name: &str| {
        pairs
            .iter()
            .find(|(own, _)| *own == name)
            .map(|(_, value)| value.to_string())
    }
}

#[test]
fn the_claude_adapter_turns_its_hook_input_into_a_common_event() {
    let input =
        json!({ "session_id": "s", "cwd": "/in", "tool_name": "Edit", "tool_input": { "a": 1 } });
    let cases: [(&str, Vars, Option<&str>, &str); 7] = [
        ("SessionStart", &[], Some("start"), "/in"),
        ("UserPromptSubmit", &[], Some("prompt"), "/in"),
        (
            "PostToolUse",
            &[("CLAUDE_PROJECT_DIR", "/env")],
            Some("tool"),
            "/env",
        ),
        ("Stop", &[], Some("stop"), "/in"),
        ("SubagentStart", &[], Some("agent-start"), "/in"),
        ("SubagentStop", &[], Some("agent-stop"), "/in"),
        ("PreCompact", &[], None, "/in"),
    ];

    for (name, vars, event, dir) in cases {
        let mut given = input.clone();
        given["hook_event_name"] = json!(name);
        assert_eq!(
            Claude.parse(&given, &env(vars)),
            Event {
                name: event.map(str::to_string),
                id: Some("s".into()),
                dir: dir.into(),
                cwd: Some("/in".into()),
                agent: None,
                tool: Some("Edit".into()),
                input: json!({ "a": 1 }),
            },
            "{name}"
        );
    }

    let event = Event::default();
    let emit = |outcome: Outcome| Claude.emit(&event, &outcome);
    let refused = emit(Outcome {
        refusal: Some("no".into()),
        ..Outcome::default()
    });
    assert_eq!((refused.stderr.as_deref(), refused.code), (Some("no"), 2));
    assert_eq!(
        emit(Outcome {
            context: Some("hi".into()),
            ..Outcome::default()
        })
        .stdout
        .as_deref(),
        Some("hi")
    );
    assert_eq!(emit(Outcome::default()).stdout, None);
    let notice = emit(Outcome {
        notice: Some("gave up".into()),
        ..Outcome::default()
    })
    .stdout
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&notice).unwrap(),
        json!({ "systemMessage": "gave up" })
    );
}

#[test]
fn the_cursor_adapter_does_the_same_from_cursors_shapes() {
    let base = json!({ "cursor_version": "3", "conversation_id": "c", "workspace_roots": ["/ws"] });
    let edit = json!({ "file_path": "/ws/a.js" });
    let cases = [
        (
            json!({ "hook_event_name": "sessionStart", "session_id": "s" }),
            Some("start"),
            None,
            Value::Null,
        ),
        (
            json!({ "hook_event_name": "beforeSubmitPrompt" }),
            Some("prompt"),
            None,
            Value::Null,
        ),
        (
            json!({ "hook_event_name": "UserPromptSubmit" }),
            Some("prompt"),
            None,
            Value::Null,
        ),
        (
            json!({ "hook_event_name": "postToolUse", "tool_name": "Shell", "tool_input": { "command": "x" } }),
            Some("tool"),
            Some("Bash"),
            json!({ "command": "x" }),
        ),
        (
            json!({ "hook_event_name": "postToolUse", "tool_name": "Write", "tool_input": edit }),
            Some("tool"),
            Some("Write"),
            edit.clone(),
        ),
        (
            json!({ "hook_event_name": "afterFileEdit", "file_path": "/ws/a.js" }),
            Some("tool"),
            Some("Write"),
            edit.clone(),
        ),
        (
            json!({ "hook_event_name": "afterShellExecution", "command": "SECRET=1 x" }),
            Some("tool"),
            Some("Bash"),
            json!({}),
        ),
        (
            json!({ "hook_event_name": "stop", "status": "completed" }),
            Some("stop"),
            None,
            Value::Null,
        ),
        (
            json!({ "hook_event_name": "preCompact" }),
            None,
            None,
            Value::Null,
        ),
    ];

    for (extra, name, tool, input) in cases {
        let mut given = base.clone();
        given
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let event = Cursor.parse(&given, &env(&[]));

        assert_eq!(event.name.as_deref(), name, "{extra}");
        assert_eq!(event.id.as_deref(), Some("c"));
        assert_eq!(event.dir, "/ws");
        assert_eq!(event.tool.as_deref(), tool, "{extra}");
        assert_eq!(event.input, input, "{extra}");
    }
    assert_eq!(
        Cursor
            .parse(&base, &env(&[("CURSOR_PROJECT_DIR", "/env")]))
            .dir,
        "/env"
    );
    assert_eq!(
        Cursor
            .parse(&json!({ "session_id": "s" }), &env(&[]))
            .id
            .as_deref(),
        Some("s")
    );

    let out = |name: &str, outcome: Outcome| {
        let event = Event {
            name: Some(name.into()),
            ..Event::default()
        };
        let emitted = Cursor.emit(&event, &outcome);
        assert_eq!(emitted.code, 0);
        serde_json::from_str::<Value>(&emitted.stdout.unwrap()).unwrap()
    };
    assert_eq!(
        out(
            "start",
            Outcome {
                context: Some("hi".into()),
                ..Outcome::default()
            }
        ),
        json!({ "additional_context": "hi" })
    );
    assert_eq!(out("start", Outcome::default()), json!({}));
    assert_eq!(
        out("prompt", Outcome::default()),
        json!({ "continue": true })
    );
    assert_eq!(out("tool", Outcome::default()), json!({}));
    assert_eq!(
        out(
            "stop",
            Outcome {
                refusal: Some("no".into()),
                ..Outcome::default()
            }
        ),
        json!({ "followup_message": "no" })
    );
    assert_eq!(
        out(
            "stop",
            Outcome {
                notice: Some("gave up".into()),
                ..Outcome::default()
            }
        ),
        json!({ "user_message": "gave up" })
    );
    assert_eq!(
        out("pre", Outcome::ask("why".into())),
        json!({ "permission": "ask", "user_message": "why", "agent_message": "why" })
    );
    assert_eq!(
        out("pre", Outcome::default()),
        json!({ "permission": "allow" })
    );
    let _ = Path::new(".");
}
