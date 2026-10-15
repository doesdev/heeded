//! The commands a person types, run as a person runs them.

mod common;

use std::path::Path;

use common::*;
use serde_json::json;

fn status(repo: &Path) -> Vec<String> {
    let mut seen: Vec<String> = git_out(repo, &["status", "--porcelain", "-uall"])
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line[3..].to_string())
        .collect();
    seen.sort();
    seen
}

#[test]
fn init_stamps_the_repo_root_from_anywhere_inside_it() {
    let lab = Lab::new("init-root");
    let repo = lab.repo(false);
    write(&repo, "deep/down/file.txt", "x\n");

    let ran = lab.bin(&repo.join("deep").join("down"), &["init"]);
    assert_eq!(ran.status, 0, "{}", ran.stderr);
    assert!(ran.stdout.starts_with(&format!("{}\n", repo.display())));
    assert!(ran.stdout.contains(&format!("created {HANDOFF}")));
    assert!(!ran.stdout.contains("no \"check\" script"));
    for file in stamped_in(NOTES) {
        assert!(exists(&repo.join(&file)), "{file}");
    }
    assert!(!exists(&repo.join("deep").join("down").join(NOTES)));

    let again = lab.bin(&lab.tmp, &["init", &repo.display().to_string()]);
    assert_eq!(again.status, 0);
    assert!(!again.stdout.contains("created"));
}

#[test]
fn init_stamps_an_empty_folder_and_a_second_run_changes_nothing() {
    let lab = Lab::new("init-plain");
    let dir = lab.dir("plain");

    let first = lab.bin(&dir, &["init"]);
    let lines: Vec<&str> = first.stdout.lines().skip(1).collect();
    let expected: Vec<String> = stamped_in(NOTES)
        .iter()
        .map(|file| format!("  created {file}"))
        .collect();
    assert_eq!(lines, expected);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&read(&dir.join(MARKER))).unwrap(),
        json!({ "notes": NOTES })
    );
    assert_eq!(read(&dir.join(NOTES).join(".gitignore")), "journal.jsonl\n");
    assert_eq!(
        read(&dir.join(HANDOFF)),
        read(Path::new("templates/notes/handoff.md"))
    );

    write(&dir, ".heeded/plan.md", "# Plan\n\nMine now.\n");
    let second = lab.bin(&dir, &["init", "--notes", "somewhere/else"]);
    assert!(
        second
            .stdout
            .lines()
            .skip(1)
            .all(|line| line.starts_with("  kept"))
    );
    assert_eq!(
        read(&dir.join(NOTES).join("plan.md")),
        "# Plan\n\nMine now.\n"
    );
    assert!(!exists(&dir.join("somewhere")));
}

#[test]
fn init_puts_the_notes_where_it_is_told_and_keeps_what_is_there() {
    let lab = Lab::new("init-owned");
    let dir = lab.dir("owned");
    let manifest = "{ \"name\": \"theirs\", \"scripts\": { \"test\": \"x\" } }\n";
    let theirs = [".gitignore", "CLAUDE.md", "docs/manual.md", "package.json"];
    write(&dir, ".gitignore", "# curated\nnode_modules\n");
    write(&dir, "CLAUDE.md", "theirs\n");
    write(&dir, "docs/manual.md", "a product\n");
    write(&dir, "package.json", manifest);
    write(&dir, "private/notes/plan.md", "theirs\n");
    let before: Vec<String> = theirs.iter().map(|file| read(&dir.join(file))).collect();

    let ran = lab.bin(&dir, &["init", "--notes", "private/notes"]);
    let done: Vec<(String, String)> = ran
        .stdout
        .lines()
        .skip(1)
        .filter(|line| line.starts_with("  "))
        .map(|line| {
            let (did, file) = line.trim().split_once(' ').unwrap();
            (file.trim().to_string(), did.to_string())
        })
        .collect();
    let files: Vec<String> = done.iter().map(|(file, _)| file.clone()).collect();
    let did = |file: &str| {
        done.iter()
            .find(|(own, _)| own == file)
            .map(|(_, did)| did.clone())
    };

    assert_eq!(files, stamped_in("private/notes"));
    assert_eq!(did("private/notes/handoff.md").as_deref(), Some("created"));
    assert_eq!(did("private/notes/plan.md").as_deref(), Some("kept"));
    assert_eq!(did("package.json").as_deref(), Some("kept"));
    assert!(ran.stdout.contains("no \"check\" script"));
    assert_eq!(
        read(&dir.join("private").join("notes").join("plan.md")),
        "theirs\n"
    );
    let after: Vec<String> = theirs.iter().map(|file| read(&dir.join(file))).collect();
    assert_eq!(after, before);
    let docs: Vec<_> = std::fs::read_dir(dir.join("docs"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(docs, vec!["manual.md"]);
}

#[test]
fn init_notes_elsewhere_keep_the_journal_ignored() {
    let lab = Lab::new("init-notes");
    let repo = lab.repo(false);
    let ran = lab.bin(&repo, &["init", "--notes", "agent/notes"]);

    assert_eq!(ran.status, 0);
    assert!(ran.stdout.contains("created agent/notes/handoff.md"));
    assert!(!exists(&repo.join(NOTES)));

    let session = lab.session(&repo, "notes");
    session.start();
    session.edit("work.txt", "work\n");

    assert!(exists(
        &repo.join("agent").join("notes").join("journal.jsonl")
    ));
    assert!(!git_out(&repo, &["status", "--porcelain", "-uall"]).contains("journal.jsonl"));
    assert!(
        session
            .stop()
            .stderr
            .contains(&native(Path::new("agent"), "notes"))
    );
}

#[test]
fn init_keeps_what_it_creates_out_of_git_unless_told_tracked() {
    let lab = Lab::new("init-private");
    let mut tracked: Vec<String> = stamped_in(NOTES)
        .into_iter()
        .filter(|file| file != JOURNAL)
        .collect();
    tracked.sort();
    let cases: [(&[&str], Vec<String>, bool); 3] = [
        (&[], vec![], true),
        (&["--notes", "agent/notes"], vec![], true),
        (&["--tracked"], tracked, false),
    ];

    for (args, seen, hidden) in cases {
        let repo = lab.repo(false);
        let mut argv = vec!["init"];
        argv.extend(args);
        let ran = lab.bin(&repo, &argv);

        assert_eq!(ran.status, 0);
        assert_eq!(status(&repo), seen, "{args:?}");
        assert_eq!(
            ran.stdout.contains("updated .git/info/exclude"),
            hidden,
            "{args:?}"
        );
        assert!(!exists(&repo.join(".gitignore")));
        assert!(!lab.bin(&repo, &argv).stdout.contains("updated"));
    }
}

#[test]
fn init_hides_nothing_the_repo_already_had_or_already_ignores() {
    let lab = Lab::new("init-had");
    let repo = lab.repo(false);
    write(&repo, ".gitignore", "private\n");
    write(&repo, MARKER, "{ \"notes\": \"private/notes\" }\n");
    write(
        &repo,
        "package.json",
        "{ \"scripts\": { \"check\": \"x\" } }\n",
    );
    assert!(!lab.bin(&repo, &["init"]).stdout.contains("exclude"));

    let stamped = lab.repo(true);
    write(
        &stamped,
        "package.json",
        "{ \"scripts\": { \"check\": \"x\" } }\n",
    );
    let again = lab.bin(&stamped, &["init"]);
    assert_eq!(again.status, 0);
    assert!(!again.stdout.contains("exclude"));
}

#[test]
fn init_says_so_when_the_repo_has_no_check() {
    let lab = Lab::new("init-check");
    let repo = lab.repo(false);
    write(&repo, "package.json", "{ \"name\": \"theirs\" }\n");

    assert!(
        lab.bin(&repo, &["init"])
            .stdout
            .contains("no \"check\" script")
    );
}

#[test]
fn heeded_fails_with_a_reason_not_a_stack() {
    let lab = Lab::new("fails");
    let repo = lab.repo(false);
    let broken = lab.repo(false);
    write(&broken, MARKER, "not json");
    let marker = broken.join(MARKER).display().to_string();
    let cases: [(&Path, &[&str], String); 7] = [
        (&lab.tmp, &["nope"], "No command \"nope\".".to_string()),
        (
            &lab.tmp,
            &["help", "nope"],
            "No command \"nope\".".to_string(),
        ),
        (
            &lab.tmp,
            &["config", "--global", "--local"],
            "Pick one of".to_string(),
        ),
        (
            &repo,
            &["moves"],
            format!("{} has no {MARKER}", repo.display()),
        ),
        (&repo, &["init", "--nope"], "Unknown option".to_string()),
        (&broken, &["moves"], format!("{marker} must be JSON")),
        (&broken, &["init"], format!("{marker} must be JSON")),
    ];

    for (cwd, args, said) in cases {
        let ran = lab.bin(cwd, args);
        assert_eq!(ran.status, 1, "{said}");
        assert!(ran.stderr.starts_with(&said), "{said}: {}", ran.stderr);
    }
}

#[test]
fn moves_lists_what_is_owed_with_the_time_and_the_file() {
    let lab = Lab::new("moves");
    let session = lab.started(true);
    let quiet = lab.bin(&session.repo, &["moves"]).stdout;
    let lines: Vec<&str> = quiet.lines().collect();
    let now = lines[0].strip_prefix("Now: ").unwrap();
    assert!(now.len() == 17 && now.ends_with('Z') && now.as_bytes()[10] == b'T');
    assert_eq!(
        lines[1],
        format!("Handoff: {}", native(&session.repo, HANDOFF))
    );
    assert_eq!(lines[2], "No moves since it was last written.");

    session.edit("one.txt", "one\n");
    session.edit(HANDOFF, "# Handoff\n\nAdded one.\n");
    session.edit("two.txt", "two\n");
    session.bash("Commit it", || {
        git(&session.repo, &["add", "-A"]);
        git(&session.repo, &["commit", "-q", "-m", "Work"]);
    });

    let owed = lab.bin(&session.repo, &["moves"]);
    assert_eq!(owed.status, 0);
    assert!(
        owed.stdout
            .ends_with("written:\n- Edit two.txt\n- Bash Commit it\n")
    );
    assert!(!owed.stdout.contains("one.txt"));
}

#[test]
fn retro_appends_one_line_or_refuses_and_says_why() {
    let lab = Lab::new("retro");
    let repo = lab.repo(true);
    let good = [
        "retro",
        "--task",
        "Add init",
        "--outcome",
        "reworked",
        "--class",
        "spec-gap",
    ];
    let mut with_note = good.to_vec();
    with_note.extend(["--note", "docs/ was not ours"]);
    let ran = lab.bin(&repo, &with_note);
    let bad = lab.bin(
        &repo,
        &[
            "retro",
            "--task",
            "x",
            "--outcome",
            "kept",
            "--class",
            "spec-gap",
        ],
    );
    let lines = journal(&repo.join(NOTES).join("rework.jsonl"));

    assert_eq!(ran.status, 0);
    assert_eq!(bad.status, 1);
    assert!(bad.stderr.starts_with("Not logged, bad class-for-outcome."));
    assert_eq!(lines.len(), 1);
    let mut line = lines[0].clone();
    assert!(line["ts"].as_str().is_some_and(|ts| ts.ends_with('Z')));
    line["ts"] = json!(0);
    assert_eq!(
        line,
        json!({ "ts": 0, "task": "Add init", "outcome": "reworked", "failureClass": "spec-gap", "note": "docs/ was not ours" })
    );
    assert_eq!(lab.bin(&lab.repo(false), &good).status, 1);
}

#[test]
fn help_prints_the_summary_or_one_command_and_succeeds() {
    let lab = Lab::new("help");
    let cases: [&[&str]; 6] = [
        &[],
        &["help"],
        &["-h"],
        &["--help"],
        &["help", "config"],
        &["config", "--help"],
    ];

    for args in cases {
        let ran = lab.bin(&lab.tmp, args);
        assert_eq!(ran.status, 0, "{args:?}");
        assert_eq!(ran.stderr, "", "{args:?}");
    }
    assert!(
        lab.bin(&lab.tmp, &[])
            .stdout
            .contains("config   step through your settings")
    );
    assert!(
        lab.bin(&lab.tmp, &["config", "-h"])
            .stdout
            .starts_with("heeded config [dir]")
    );
    for topic in ["init", "moves", "retro", "config", "help"] {
        assert!(
            lab.bin(&lab.tmp, &["help", topic])
                .stdout
                .starts_with(&format!("heeded {topic}")),
            "{topic}"
        );
    }
}

#[test]
fn version_says_which_build_this_is() {
    let lab = Lab::new("version");
    let wanted = format!(
        "heeded {}
",
        env!("CARGO_PKG_VERSION")
    );

    for args in [&["version"][..], &["--version"], &["init", "--version"]] {
        let ran = lab.bin(&lab.tmp, args);
        assert_eq!(ran.status, 0, "{args:?}");
        assert_eq!(ran.stdout, wanted, "{args:?}");
    }
}

fn global(key: &str) -> String {
    git_out(Path::new("."), &["config", "--global", key])
        .trim()
        .to_string()
}

#[test]
fn config_writes_the_user_config_once_and_shows_what_is_in_effect() {
    let lab = Lab::new("config-show");
    let home = lab.dir("home");
    let repo = lab.repo(true);
    let file = home.join("config.json");

    std::fs::remove_dir_all(&home).unwrap();
    let first = lab.bin_at(&home, &repo, &["config"], Some(""));
    let written: serde_json::Value = serde_json::from_str(&read(&file)).unwrap();
    assert_eq!(first.status, 0, "{}", first.stderr);
    assert!(
        first
            .stdout
            .starts_with(&format!("created {}\n", file.display()))
    );
    assert_eq!(
        written,
        json!({
            "permission": { "commit": "ask", "push": "ask" },
            "identity": { "name": global("user.name"), "email": global("user.email"), "attribution": "any" },
            "checks": { "changelog": false, "quiet": [] },
            "directives": {}
        })
    );

    write(
        &home,
        "config.json",
        &json!({ "permission": { "commit": "allow" }, "directives": { "voice": "Dry." } })
            .to_string(),
    );
    let second = lab.bin_at(&home, &repo, &["config"], Some(""));
    let shown: serde_json::Value =
        serde_json::from_str(&second.stdout[second.stdout.find('{').unwrap()..]).unwrap();

    assert!(
        second
            .stdout
            .starts_with(&format!("kept {}\n", file.display()))
    );
    assert_eq!(
        shown["permission"],
        json!({ "commit": "allow", "push": "ask" })
    );
    assert_eq!(shown["directives"], json!(["voice (4 characters)"]));
}

#[test]
fn config_global_and_local_write_the_file_from_answers() {
    let lab = Lab::new("config-answers");
    let home = lab.dir("home");
    let repo = lab.repo(true);

    let global = lab.bin_at(
        &home,
        &repo,
        &["config", "--global"],
        Some("allow\n\n\n\n\non\n\nvoice\nShort.\n"),
    );
    let written: serde_json::Value =
        serde_json::from_str(&read(&home.join("config.json"))).unwrap();
    assert_eq!(global.status, 0, "{}", global.stderr);
    assert!(
        global
            .stdout
            .contains(&format!("wrote {}", home.join("config.json").display()))
    );
    assert_eq!(written["permission"]["commit"], json!("allow"));
    assert_eq!(written["checks"]["changelog"], json!(true));
    assert_eq!(written["directives"], json!({ "voice": "Short." }));

    let local = lab.bin_at(
        &home,
        &repo,
        &["config", "--local"],
        Some("ask\n\n\n\n\n\n\nvoice\noff\n"),
    );
    let marker: serde_json::Value = serde_json::from_str(&read(&repo.join(MARKER))).unwrap();
    assert_eq!(local.status, 0, "{}", local.stderr);
    assert_eq!(
        marker,
        json!({ "notes": NOTES, "permission": { "commit": "ask" }, "directives": { "voice": false } })
    );
    assert!(local.stdout.contains("(now inherited: allow)"));
    assert!(local.stdout.contains("\"directives\": []"));

    let unstamped = lab.bin_at(&home, &lab.repo(false), &["config", "--local"], Some(""));
    assert_eq!(unstamped.status, 1);
    assert!(unstamped.stderr.contains(&format!("has no {MARKER}")));
}

// a script line the way npm runs one; cmd takes its line unescaped
#[cfg(windows)]
fn shell(line: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;

    let mut command = std::process::Command::new("cmd");
    command.arg("/C").raw_arg(line);
    command
}

#[cfg(not(windows))]
fn shell(line: &str) -> std::process::Command {
    let mut command = std::process::Command::new("sh");
    command.args(["-c", line]);
    command
}

#[test]
fn the_stamped_check_fails_until_someone_wires_it_up() {
    let manifest: serde_json::Value =
        serde_json::from_str(&read(Path::new("templates/root/package.json"))).unwrap();
    let check = manifest["scripts"]["check"].as_str().unwrap();
    let ran = shell(check).output().unwrap();

    assert_eq!(ran.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&ran.stderr).contains("not wired up"));
}
