//! The hook as Claude Code and Cursor run it, against real repos.

mod common;

use common::*;
use serde_json::json;

#[test]
fn stop_passes_when_the_session_did_no_work() {
    let lab = Lab::new("idle");
    let session = lab.started(true);

    assert_eq!(session.stop().status, 0);
    assert!(session.journal().is_empty());
}

#[test]
fn an_unstamped_repo_is_left_alone_journal_and_all() {
    let lab = Lab::new("unstamped");
    let session = lab.started(false);
    session.edit("work.txt", "work\n");

    assert_eq!(session.stop().status, 0);
    assert!(!exists(&session.repo.join(JOURNAL)));
}

#[test]
fn stop_refuses_after_work_until_the_handoff_is_written() {
    let lab = Lab::new("owed");
    let session = lab.started(true);
    session.edit("lib/work.js", "work\n");

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains(&native(&session.repo, HANDOFF)));
    assert!(refused.stderr.contains("- Edit lib/work.js"));
    assert!(refused.stderr.contains("heeded:handoff skill"));

    session.edit(HANDOFF, "# Handoff\n\nAdded work.\n");
    assert_eq!(session.stop().status, 0);
}

#[test]
fn work_after_the_handoff_makes_it_stale_again() {
    let lab = Lab::new("stale");
    let session = lab.started(true);
    session.edit("one.txt", "one\n");
    session.edit(HANDOFF, "# Handoff\n\nAdded one.\n");
    session.edit("two.txt", "two\n");

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains("- Edit two.txt"));
    assert!(!refused.stderr.contains("one.txt"));

    session.edit(HANDOFF, "# Handoff\n\nAdded one and two.\n");
    assert_eq!(session.stop().status, 0);

    session.prompt();
    session.edit("three.txt", "three\n");
    assert_eq!(session.stop().status, 2);
}

#[test]
fn the_journal_records_each_move_by_description_for_commands() {
    let lab = Lab::new("journal");
    let session = lab.started(true);
    session.edit("one.txt", "one\n");
    session.bash("Look around", || {});
    session.bash("Commit it", || {
        git(&session.repo, &["add", "-A"]);
        git(&session.repo, &["commit", "-q", "-m", "Work"]);
    });
    session.edit(HANDOFF, "# Handoff\n\nCommitted one.\n");

    let entries = session.journal();
    let raw = read(&session.repo.join(JOURNAL));
    let seen: Vec<_> = entries
        .iter()
        .map(|e| (e["tool"].clone(), e["what"].clone(), e["handoff"].clone()))
        .collect();

    assert_eq!(
        seen,
        vec![
            (json!("Edit"), json!("one.txt"), json!(false)),
            (json!("Bash"), json!("Commit it"), json!(false)),
            (json!("Edit"), json!(HANDOFF), json!(true)),
        ]
    );
    assert!(
        entries
            .iter()
            .all(|entry| entry["session"] == json!(session.id))
    );
    assert!(entries.iter().all(|entry| {
        entry["ts"]
            .as_str()
            .is_some_and(|ts| ts.len() == 24 && ts.ends_with('Z'))
    }));
    assert!(!raw.contains("hunter2"));
    assert_eq!(session.stop().status, 0);
}

#[test]
fn changes_nobody_journaled_are_caught_at_stop() {
    let lab = Lab::new("sneaky");
    let session = lab.started(true);
    write(&session.repo, "sneaky.txt", "made by a tool with no hook\n");

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains("- untracked"));
}

#[test]
fn the_humans_edits_between_turns_are_logged_but_not_owed() {
    let lab = Lab::new("human");
    let session = lab.started(true);
    assert_eq!(session.stop().status, 0);

    write(&session.repo, "human.txt", "typed by hand\n");
    session.prompt();
    session.bash("Run the tests", || {});

    assert_eq!(session.stop().status, 0);
    assert_eq!(moves(&session.journal()), vec![pair("outside", "")]);
}

#[test]
fn stop_refuses_when_the_handoff_was_deleted() {
    let lab = Lab::new("deleted");
    let session = lab.started(true);
    session.bash("Tidy up", || {
        std::fs::remove_file(session.repo.join(HANDOFF)).unwrap()
    });

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains("is gone"));
}

#[test]
fn stop_refuses_when_the_marker_was_deleted() {
    let lab = Lab::new("unmarked");
    let session = lab.started(true);
    session.bash("Opt out quietly", || {
        std::fs::remove_file(session.repo.join(MARKER)).unwrap()
    });

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.starts_with(&format!("{MARKER} is gone")));

    for _ in 1..MAX_BLOCKS {
        assert_eq!(session.stop().status, 2);
    }
    assert_eq!(session.stop().status, 0);
}

#[test]
fn stop_gives_up_loudly_after_max_blocks_refusals() {
    let lab = Lab::new("gives-up");
    let session = lab.started(true);
    session.edit("work.txt", "work\n");

    for _ in 0..MAX_BLOCKS {
        assert_eq!(session.stop().status, 2);
    }

    let gave_up = session.stop();
    assert_eq!(gave_up.status, 0);
    let message = gave_up.json()["systemMessage"]
        .as_str()
        .unwrap_or("")
        .to_string();
    assert!(message.contains(&native(&session.repo, HANDOFF)));
}

#[test]
fn the_next_session_is_told_what_the_last_one_left_unhanded() {
    let lab = Lab::new("leftover");
    let first = lab.started(true);
    first.edit("work.txt", "work\n");
    for _ in 0..=MAX_BLOCKS {
        first.stop();
    }

    let second = lab.session(&first.repo, "second");
    let started = second.start();
    assert_eq!(started.status, 0);
    assert!(started.stdout.contains("- Edit work.txt"));
    assert!(started.stdout.contains(&native(&first.repo, HANDOFF)));

    second.edit(HANDOFF, "# Handoff\n\nCaught up.\n");
    assert_eq!(second.stop().status, 0);
    assert!(
        !lab.session(&first.repo, "third")
            .start()
            .stdout
            .contains("earlier")
    );
}

#[test]
fn a_hook_that_arrives_mid_session_starts_from_now() {
    let lab = Lab::new("late");
    let repo = lab.repo(true);
    let session = lab.session(&repo, "late");
    write(&repo, "before.txt", "before\n");

    assert_eq!(session.stop().status, 0);
    session.edit("after.txt", "after\n");
    assert_eq!(session.stop().status, 2);
}

#[test]
fn resuming_a_session_keeps_what_it_owes() {
    let lab = Lab::new("resume");
    let session = lab.started(true);
    session.edit("work.txt", "work\n");

    assert_eq!(session.start().status, 0);
    assert_eq!(session.stop().status, 2);
}

#[test]
fn a_session_is_told_where_the_notes_are_when_it_starts() {
    let lab = Lab::new("told");
    let repo = lab.repo(true);
    let started = lab.session(&repo, "told").start();

    assert_eq!(started.status, 0);
    assert!(
        started
            .stdout
            .contains(&repo.join(NOTES).display().to_string())
    );
    assert!(started.stdout.contains("handoff.md"));
    assert!(!started.stdout.contains("earlier sessions"));
    assert_eq!(lab.session(&lab.repo(false), "quiet").start().stdout, "");
}

#[test]
fn heeded_ships_no_rules_of_its_own() {
    let lab = Lab::new("no-rules");
    let told = lab.session(&lab.repo(true), "rules").start().stdout;

    assert!(!told.contains("standing directives"));
    assert_eq!(told.trim().lines().count(), 1);
}

#[test]
fn a_docs_folder_with_a_handoff_in_it_opts_nothing_in() {
    let lab = Lab::new("docs");
    let session = lab.started(false);
    session.edit("docs/handoff.md", "# Handoff\n");
    session.edit("docs/manual.md", "a product\n");

    assert_eq!(session.stop().status, 0);
    assert!(!exists(&session.repo.join("docs").join("journal.jsonl")));
}

#[test]
fn a_marker_with_no_handoff_behind_it_leaves_the_repo_alone() {
    let lab = Lab::new("absent");
    let repo = lab.repo(false);
    write(&repo, MARKER, "{ \"notes\": \"private\" }\n");
    let session = lab.session(&repo, "absent");

    assert_eq!(session.start().stdout, "");
    session.edit("work.txt", "work\n");
    assert_eq!(session.stop().status, 0);
    assert!(!exists(&repo.join("private")));
}

#[test]
fn notes_in_a_private_checkout_the_repo_ignores_still_gate_it() {
    let lab = Lab::new("private");
    let repo = lab.repo(false);
    let inner = repo.join("private");
    let handoff = "private/notes/handoff.md";
    write(&repo, ".gitignore", "private\n");
    write(&repo, MARKER, "{ \"notes\": \"private/notes\" }\n");
    std::fs::create_dir_all(&inner).unwrap();
    git(&inner, &["init", "-q"]);
    write(&repo, handoff, "# Handoff\n");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "Opt in"]);

    let session = lab.session(&repo, "private");
    session.start();
    session.edit("private/notes/plan.md", "# Plan\n");
    assert_eq!(session.stop().status, 0);

    session.edit("work.txt", "work\n");
    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains(&native(&inner, "notes/handoff.md")));

    session.edit(handoff, "# Handoff\n\nAdded work.\n");
    assert_eq!(session.stop().status, 0);
    let entries = journal(&inner.join("notes").join("journal.jsonl"));
    let seen: Vec<_> = entries
        .iter()
        .map(|e| (e["what"].clone(), e["handoff"].clone()))
        .collect();
    assert_eq!(
        seen,
        vec![
            (json!("work.txt"), json!(false)),
            (json!(handoff), json!(true))
        ]
    );
}

#[test]
fn a_repo_stamped_by_heeded_init_is_gated_from_then_on() {
    let lab = Lab::new("stamped-later");
    let session = lab.started(false);
    session.edit("before.txt", "before\n");
    assert_eq!(session.stop().status, 0);

    session.bash("Stamp the repo", || {
        lab.bin(&session.repo, &["init"]);
    });
    assert_eq!(session.stop().status, 0);

    session.prompt();
    session.edit("after.txt", "after\n");
    assert_eq!(session.stop().status, 2);
}

#[test]
fn a_repo_that_never_opted_in_costs_no_git_and_leaves_no_state() {
    let lab = Lab::new("fast");
    let repo = lab.repo(false);
    let events = [
        ("SessionStart", json!({})),
        ("UserPromptSubmit", json!({})),
        (
            "PreToolUse",
            json!({ "tool_name": "Bash", "tool_input": { "command": "git commit -m x" } }),
        ),
        (
            "PostToolUse",
            json!({ "tool_name": "Bash", "tool_input": { "description": "x" } }),
        ),
        ("Stop", json!({})),
    ];

    for (event, extra) in events {
        let ran = lab.hook(&repo, Some("fast"), event, extra);
        assert_eq!(ran.status, 0, "{event}");
        assert_eq!(format!("{}{}", ran.stdout, ran.stderr), "", "{event}");
    }
    assert!(!exists(&lab.state.join("fast.json")));
}

// subagents

#[test]
fn a_subagents_moves_are_the_sessions_and_stop_waits_until_it_is_back() {
    let lab = Lab::new("subagent");
    let session = lab.started(true);
    let file = session.repo.join("sub.txt").display().to_string();

    session.hook(
        "SubagentStart",
        json!({ "agent_id": "a1", "agent_type": "general-purpose" }),
    );
    write(&session.repo, "sub.txt", "sub\n");
    session.hook(
        "PostToolUse",
        json!({ "agent_id": "a1", "tool_name": "Write", "tool_input": { "file_path": file } }),
    );

    assert_eq!(session.stop().status, 0);
    let entries = session.journal();
    assert_eq!(moves(&entries), vec![pair("Write", "sub.txt")]);
    assert_eq!(entries[0]["agent"], json!("a1"));

    session.hook("SubagentStop", json!({ "agent_id": "a1" }));
    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains("- Write sub.txt"));
}

#[test]
fn work_in_a_subagents_worktree_is_journaled_and_owed() {
    let lab = Lab::new("worktree");
    let session = lab.started(true);
    let repo = &session.repo;
    let inside = ".claude/worktrees/agent-a2";
    let tree = repo.join(".claude").join("worktrees").join("agent-a2");
    let cwd = tree.display().to_string();
    let post = |tool: &str, input: serde_json::Value| {
        session.hook(
            "PostToolUse",
            json!({ "agent_id": "a2", "cwd": cwd, "tool_name": tool, "tool_input": input }),
        )
    };
    git(repo, &["worktree", "add", "-q", "-b", "side", &cwd]);

    session.hook("SubagentStart", json!({ "agent_id": "a2", "cwd": cwd }));
    post("Bash", json!({ "description": "Look around" }));
    write(repo, &format!("{inside}/wt.txt"), "wt\n");
    post(
        "Write",
        json!({ "file_path": tree.join("wt.txt").display().to_string() }),
    );
    post("Bash", json!({ "description": "Look again" }));
    git(&tree, &["add", "-A"]);
    git(&tree, &["commit", "-q", "-m", "Side work"]);
    post("Bash", json!({ "description": "Commit it" }));
    session.hook("SubagentStop", json!({ "agent_id": "a2", "cwd": cwd }));

    let entries = session.journal();
    assert_eq!(
        moves(&entries),
        vec![
            pair("Write", &format!("{inside}/wt.txt")),
            pair("Bash", "Commit it")
        ]
    );
    assert!(entries.iter().all(|entry| entry["agent"] == json!("a2")));
    assert!(!exists(&tree.join(JOURNAL)));

    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains(&format!("- Write {inside}/wt.txt")));
    assert!(refused.stderr.contains(&native(repo, HANDOFF)));
    assert!(!refused.stderr.contains("untracked"));

    git(repo, &["merge", "-q", "side"]);
    session.hook(
        "PostToolUse",
        json!({ "tool_name": "Bash", "tool_input": { "description": "Merge side" } }),
    );
    session.edit(HANDOFF, "# Handoff\n\nMerged side.\n");
    assert_eq!(session.stop().status, 0);
}

#[test]
fn a_directory_in_some_other_repo_is_not_a_side_of_this_one() {
    let lab = Lab::new("other");
    let session = lab.started(true);
    let other = lab.repo(false);

    write(&other, "elsewhere.txt", "x\n");
    session.hook(
        "PostToolUse",
        json!({ "cwd": other.display().to_string(), "tool_name": "Write", "tool_input": { "file_path": other.join("elsewhere.txt").display().to_string() } }),
    );

    assert!(session.journal().is_empty());
    assert_eq!(session.stop().status, 0);
}

// directives

#[test]
fn a_stamped_session_is_told_the_directives_after_the_notes() {
    let lab = Lab::new("directives");
    let repo = lab.repo(true);
    let start = |id: &str| lab.session(&repo, id).start().stdout;

    assert!(!start("d1").contains("standing directives"));

    write(&lab.home, "voice.md", "Short sentences.\nNo filler.\n");
    write(
        &lab.home,
        "config.json",
        &json!({ "directives": { "style": "Two spaces.", "voice": { "file": "voice.md" } } })
            .to_string(),
    );
    let told = start("d2");

    assert!(told.contains("- **style**: Two spaces.\n- **voice**:\nShort sentences.\nNo filler."));
    assert!(told.find("working notes are in") < told.find("standing directives"));
    assert!(told.contains("can add to these, not loosen them.\n- **style**"));
    assert_eq!(lab.session(&lab.repo(false), "d3").start().stdout, "");
}

// cursor

#[test]
fn the_same_gate_runs_from_cursor_shaped_hook_calls() {
    let lab = Lab::new("cursor");
    let repo = lab.repo(true);
    let root = repo.display().to_string();
    let call = |event: &str, extra: serde_json::Value| {
        let mut input = json!({ "cursor_version": "3.14.7", "conversation_id": "cursor", "workspace_roots": [root], "cwd": "" });
        input
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        lab.hook(&repo, None, event, input)
    };

    let started = call("sessionStart", json!({ "session_id": "cursor" }));
    assert_eq!(started.status, 0);
    assert!(
        started.json()["additional_context"]
            .as_str()
            .unwrap()
            .contains(&repo.join(NOTES).display().to_string())
    );
    assert_eq!(
        call("beforeSubmitPrompt", json!({})).json(),
        json!({ "continue": true })
    );
    assert_eq!(call("stop", json!({})).json(), json!({}));

    write(&repo, "work.txt", "work\n");
    call(
        "postToolUse",
        json!({ "tool_name": "Write", "tool_input": { "file_path": repo.join("work.txt").display().to_string() } }),
    );
    write(&repo, "more.txt", "more\n");
    call(
        "postToolUse",
        json!({ "tool_name": "Shell", "tool_input": { "command": "SECRET=hunter2 x" } }),
    );

    let refused = call("stop", json!({ "status": "completed", "loop_count": 0 }));
    assert_eq!(refused.status, 0);
    assert_eq!(refused.stderr, "");
    assert!(
        refused.json()["followup_message"]
            .as_str()
            .unwrap()
            .contains("- Write work.txt\n- Bash")
    );
    assert!(!read(&repo.join(JOURNAL)).contains("hunter2"));

    write(&repo, HANDOFF, "# Handoff\n\nDid work.\n");
    call(
        "afterFileEdit",
        json!({ "file_path": repo.join(HANDOFF).display().to_string() }),
    );
    assert_eq!(call("stop", json!({})).json(), json!({}));
}

// the commit and push guard

// assembled here so that this file never holds a line that looks secret
fn fake_aws() -> String {
    ["AKIA", "ABCDEFGHIJKLMNOP"].concat()
}

fn decision(ran: &Ran) -> (Option<String>, String) {
    let out = ran.json();
    let inner = &out["hookSpecificOutput"];

    (
        inner["permissionDecision"].as_str().map(str::to_string),
        inner["permissionDecisionReason"]
            .as_str()
            .unwrap_or("")
            .to_string(),
    )
}

#[test]
fn the_guard_asks_refuses_or_stays_out_of_the_way() {
    let lab = Lab::new("guard");
    let session = lab.started(true);
    let repo = &session.repo;
    let pre = |tool: &str, input: serde_json::Value| {
        let ran = session.hook(
            "PreToolUse",
            json!({ "tool_name": tool, "tool_input": input }),
        );
        assert_eq!(ran.status, 0);
        decision(&ran)
    };
    let bash = |command: &str| pre("Bash", json!({ "command": command }));
    let ask = Some("ask".to_string());
    let deny = Some("deny".to_string());

    assert_eq!(bash("npm run check"), (None, String::new()));
    assert_eq!(bash("git push").0, ask);
    assert_eq!(
        pre(
            "Edit",
            json!({ "file_path": repo.join("lib").join("a.js").display().to_string() })
        )
        .0,
        None
    );
    assert_eq!(
        pre(
            "Edit",
            json!({ "file_path": repo.join(MARKER).display().to_string() })
        )
        .0,
        ask
    );
    assert_eq!(
        pre(
            "Write",
            json!({ "file_path": lab.home.join("config.json").display().to_string() })
        )
        .0,
        ask
    );

    write(repo, "lib/a.js", "export const a = 1\n");
    git(repo, &["add", "-A"]);
    assert_eq!(bash("git commit -m \"Add a\"").0, ask);

    write(
        repo,
        MARKER,
        &json!({ "notes": NOTES, "checks": { "changelog": true } }).to_string(),
    );
    let nag = bash("git commit -m \"Add a\"");
    assert_eq!(nag.0, deny);
    assert!(nag.1.contains("CHANGELOG.md"));
    assert_eq!(bash("git commit -m \"Add a\"").0, ask);

    write(
        repo,
        "CHANGELOG.md",
        "# Changelog\n\n## [Unreleased]\n\n- You can now a.\n",
    );
    write(
        repo,
        "lib/b.js",
        &format!("export const k = \"{}\"\n", fake_aws()),
    );
    let leaked = bash("git add -A && git commit -m \"Add b\"");
    assert_eq!(leaked.0, deny);
    assert!(leaked.1.contains("lib/b.js: an AWS access key"));
    assert!(!leaked.1.contains(&fake_aws()));
    assert_eq!(bash("git commit -m \"Add a\"").0, ask);

    std::fs::remove_file(repo.join("lib").join("b.js")).unwrap();
    write(repo, ".env", "X=1\n");
    assert!(
        bash("git add -A && git commit -m x")
            .1
            .contains(".env: a credentials file")
    );
    std::fs::remove_file(repo.join(".env")).unwrap();

    let marker = json!({
        "notes": NOTES,
        "permission": { "commit": "allow" },
        "identity": { "email": "someone@else.dev", "attribution": "forbidden" }
    });
    write(repo, MARKER, &marker.to_string());
    let wrong = bash("git commit -m \"x\n\nCo-Authored-By: M <m@x.y>\"");
    assert_eq!(wrong.0, deny);
    assert!(wrong.1.contains("git user.email is \"test@example.com\""));
    assert!(wrong.1.contains("Co-Authored-By"));

    git(repo, &["config", "user.email", "someone@else.dev"]);
    assert_eq!(bash("git commit -m \"Add a\""), (None, String::new()));
    assert_eq!(bash("git commit -m \"Add a\" && git push").0, ask);
}

#[test]
fn the_guard_says_nothing_in_a_repo_that_has_not_opted_in() {
    let lab = Lab::new("guard-off");
    let session = lab.started(false);
    write(
        &session.repo,
        "lib/b.js",
        &format!("export const k = \"{}\"\n", fake_aws()),
    );
    let ran = session.hook(
        "PreToolUse",
        json!({ "tool_name": "Bash", "tool_input": { "command": "git add -A && git commit -m x && git push" } }),
    );

    assert_eq!(ran.status, 0);
    assert_eq!(ran.stdout, "");
}

#[test]
fn the_guard_judges_a_git_c_commit_by_the_repo_it_names() {
    let lab = Lab::new("guard-c");
    let session = lab.started(true);
    let repo = &session.repo;
    let inner = repo.join("private");
    write(repo, ".gitignore", "private\n");
    std::fs::create_dir_all(&inner).unwrap();
    git(&inner, &["init", "-q"]);
    write(repo, "lib/a.js", "x\n");
    git(repo, &["add", "-A"]);
    let bash = |command: &str| {
        decision(&session.hook(
            "PreToolUse",
            json!({ "tool_name": "Bash", "tool_input": { "command": command } }),
        ))
        .0
    };
    let ask = Some("ask".to_string());

    assert_eq!(bash("git commit -m x"), ask);
    assert_eq!(bash("git -C private commit -m notes"), None);
    assert_eq!(bash("git -C private push"), None);
    assert_eq!(bash("gh release view 1.0.0 && git tag -l"), None);
    assert_eq!(bash("heeded config --local"), ask);

    write(
        &inner,
        MARKER,
        &json!({ "notes": ".", "permission": { "commit": "allow" } }).to_string(),
    );
    write(&inner, "handoff.md", "# Handoff\n");
    assert_eq!(bash("git -C private commit -m notes"), None);
    assert_eq!(bash("git -C private push"), ask);
}

#[test]
fn the_guard_judges_a_worktree_of_a_hidden_marker_repo_as_that_repo() {
    let lab = Lab::new("guard-wt");
    let session = lab.started(true);
    let repo = &session.repo;
    git(repo, &["rm", "-q", "--cached", MARKER]);
    write(repo, ".git/info/exclude", &format!("{MARKER}\n"));
    git(repo, &["commit", "-q", "-m", "Hide the marker"]);
    let tree = repo.join(".claude").join("worktrees").join("agent-a3");
    let at = tree.display().to_string();
    git(repo, &["worktree", "add", "-q", "-b", "side", &at]);
    assert!(!exists(&tree.join(MARKER)));

    let bash = |cwd: &str, command: &str| {
        decision(&session.hook(
            "PreToolUse",
            json!({ "cwd": cwd, "tool_name": "Bash", "tool_input": { "command": command } }),
        ))
    };
    let here = repo.display().to_string();
    let ask = Some("ask".to_string());
    let deny = Some("deny".to_string());

    write(
        &tree,
        "lib/b.js",
        &format!("export const k = \"{}\"\n", fake_aws()),
    );
    git(&tree, &["add", "-A"]);
    let leaked = bash(&at, "git commit -m \"Add b\"");
    assert_eq!(leaked.0, deny);
    assert!(leaked.1.contains("lib/b.js: an AWS access key"));
    assert_eq!(bash(&here, &format!("git -C \"{at}\" commit -m b")).0, deny);
    assert_eq!(
        bash(&here, &format!("cd \"{at}\" && git commit -m b")).0,
        deny
    );

    write(&tree, "lib/b.js", "export const k = 1\n");
    git(&tree, &["add", "-A"]);
    assert_eq!(bash(&at, "git commit -m \"Add b\"").0, ask);
    assert_eq!(bash(&at, "git push -u origin side").0, ask);
    assert_eq!(bash(&here, &format!("git -C \"{at}\" push")).0, ask);

    write(
        repo,
        MARKER,
        &json!({ "notes": NOTES, "permission": { "commit": "allow" } }).to_string(),
    );
    assert_eq!(bash(&at, "git commit -m \"Add b\""), (None, String::new()));
    assert_eq!(bash(&at, "git push").0, ask);
}

#[test]
fn the_guard_judges_the_notes_checkout_as_the_repo_it_belongs_to() {
    let lab = Lab::new("guard-notes");
    let session = lab.started(true);
    let repo = &session.repo;
    let inner = repo.join("private");
    write(repo, ".gitignore", "private\n");
    std::fs::create_dir_all(&inner).unwrap();
    git(&inner, &["init", "-q"]);
    git(&inner, &["config", "user.email", "test@example.com"]);
    write(&inner, "handoff.md", "# Handoff\n");
    let mark = |extra: serde_json::Value| {
        let mut marker = json!({ "notes": "private" });
        marker
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().cloned().unwrap_or_default());
        write(repo, MARKER, &marker.to_string());
    };
    let bash = |command: &str| {
        decision(&session.hook(
            "PreToolUse",
            json!({ "tool_name": "Bash", "tool_input": { "command": command } }),
        ))
    };
    let ask = Some("ask".to_string());
    let deny = Some("deny".to_string());

    mark(json!({}));
    git(&inner, &["add", "-A"]);
    assert_eq!(bash("git -C private commit -m notes").0, ask);
    assert_eq!(bash("cd private && git push").0, ask);

    write(&inner, "plan.md", &format!("the key was {}\n", fake_aws()));
    let leaked = bash("git -C private add -A && git -C private commit -m notes");
    assert_eq!(leaked.0, deny);
    assert!(leaked.1.contains("plan.md: an AWS access key"));
    std::fs::remove_file(inner.join("plan.md")).unwrap();

    mark(json!({ "permission": { "commit": "allow" }, "checks": { "changelog": true } }));
    write(&inner, "probe.rs", "fn main() {}\n");
    git(&inner, &["add", "-A"]);
    assert_eq!(
        bash("git -C private commit -m notes"),
        (None, String::new())
    );
    assert_eq!(bash("git -C private push").0, ask);

    let outside = lab.dir("notes");
    git(&outside, &["init", "-q"]);
    write(&outside, "handoff.md", "# Handoff\n");
    write(
        repo,
        MARKER,
        &json!({ "notes": outside.display().to_string() }).to_string(),
    );
    let at = outside.display().to_string();
    assert_eq!(bash(&format!("git -C \"{at}\" push")).0, ask);
    assert_eq!(bash("git -C private push").0, None);
}

#[test]
fn a_subagent_is_refused_where_the_session_would_be_asked() {
    let lab = Lab::new("guard-agent");
    let session = lab.started(true);
    let repo = &session.repo;
    write(repo, "lib/a.js", "x\n");
    git(repo, &["add", "-A"]);
    let pre = |agent: Option<&str>, tool: &str, input: serde_json::Value| {
        let mut event = json!({ "tool_name": tool, "tool_input": input });
        if let Some(id) = agent {
            event["agent_id"] = json!(id);
        }
        decision(&session.hook("PreToolUse", event))
    };
    let bash =
        |agent: Option<&str>, command: &str| pre(agent, "Bash", json!({ "command": command }));
    let ask = Some("ask".to_string());
    let deny = Some("deny".to_string());

    assert_eq!(bash(None, "git commit -m x").0, ask);
    let sent = bash(Some("a4"), "git commit -m x");
    assert_eq!(sent.0, deny);
    assert!(sent.1.contains("a commit needs the owner's OK"));
    assert!(sent.1.contains("report back"));
    assert_eq!(bash(Some("a4"), "git push").0, deny);
    assert_eq!(
        pre(
            Some("a4"),
            "Edit",
            json!({ "file_path": repo.join(MARKER).display().to_string() })
        )
        .0,
        deny
    );
    assert_eq!(bash(Some("a4"), "npm run check"), (None, String::new()));
    assert_eq!(bash(Some(""), "git commit -m x").0, ask);

    write(
        repo,
        MARKER,
        &json!({ "notes": NOTES, "permission": { "commit": "allow" } }).to_string(),
    );
    assert_eq!(bash(Some("a4"), "git commit -m x"), (None, String::new()));
}

#[test]
fn a_heredoc_written_to_a_file_is_not_the_command() {
    let lab = Lab::new("guard-heredoc");
    let session = lab.started(true);
    let bash = |command: &str| {
        decision(&session.hook(
            "PreToolUse",
            json!({ "tool_name": "Bash", "tool_input": { "command": command } }),
        ))
        .0
    };

    assert_eq!(
        bash("cat > replay.mjs <<'EOF'\nrun('git commit -m x')\nEOF\nnode replay.mjs"),
        None
    );
    assert_eq!(
        bash("bash <<'EOF'\ngit commit -m x\nEOF"),
        Some("ask".to_string())
    );
}

// private notes

#[test]
fn private_notes_hold_the_handoff_and_never_reach_git() {
    let lab = Lab::new("private-notes");
    let repo = lab.repo(false);
    write(
        &repo,
        MARKER,
        "{ \"notes\": \"docs\", \"privateNotes\": \".private\" }\n",
    );
    write(&repo, "docs/plan.md", "# Plan\n");
    write(&repo, ".private/.gitignore", "*\n");
    write(&repo, ".private/handoff.md", "# Handoff\n");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "Stamp"]);
    let session = lab.session(&repo, "private");
    let handoff = ".private/handoff.md";

    let told = session.start().stdout;
    assert!(told.contains(&format!(
        "working notes are in {}, its private notes in {}",
        repo.join("docs").display(),
        repo.join(".private").display()
    )));

    session.edit("work.txt", "work\n");
    let refused = session.stop();
    assert_eq!(refused.status, 2);
    assert!(refused.stderr.contains(&native(&repo, handoff)));
    assert!(exists(&repo.join(".private/journal.jsonl")));
    assert!(!exists(&repo.join("docs/journal.jsonl")));

    session.edit(handoff, "# Handoff\n\nDid work.\n");
    assert_eq!(session.stop().status, 0);

    git(&repo, &["add", "-f", handoff]);
    let (verdict, why) = decision(&session.hook(
        "PreToolUse",
        json!({ "tool_name": "Bash", "tool_input": { "command": "git commit -m x" } }),
    ));
    assert_eq!(verdict.as_deref(), Some("deny"));
    assert!(why.contains("private notes"));
    assert!(why.contains(handoff));
}
