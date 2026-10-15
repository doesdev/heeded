//! Checks on this repo itself: its records, its manifests, its writing.

mod common;

use std::path::Path;

use common::{git_out, read};
use heeded::snapshot::{SECTIONS, check_handoff};
use regex::Regex;
use serde_json::Value;

const STATUSES: [&str; 4] = ["accepted", "rejected", "amended", "superseded"];

fn field<'a>(text: &'a str, name: &str) -> &'a str {
    let prefix = format!("{name}: ");

    text.lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map_or("", str::trim)
}

/// A record is ratified by a model id, whoever proposed it.
fn check_record(text: &str) -> Vec<&'static str> {
    let proposer = field(text, "Proposed by");
    let date = Regex::new(r"^\d{4}-\d\d-\d\dT\d\d:\d\dZ$").unwrap();
    let ratifier = Regex::new(r"^[a-z][a-z0-9.-]+$").unwrap();
    let ratified = field(text, "Ratified by");
    let human = proposer.starts_with("human");
    let checks = [
        ("date", date.is_match(field(text, "Date"))),
        (
            "status",
            STATUSES.contains(&field(text, "Status").split(' ').next().unwrap_or("")),
        ),
        ("proposed-by", human || proposer.starts_with("model")),
        (
            "ratified-by",
            ratifier.is_match(ratified) && !ratified.starts_with("human"),
        ),
        (
            "assessment",
            !human || text.lines().any(|line| line.trim_end() == "## Assessment"),
        ),
    ];

    checks
        .into_iter()
        .filter(|(_, ok)| !ok)
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn check_record_enforces_who_proposed_and_who_ratified() {
    let record =
        |lines: &[&str], body: &str| format!("# 0000: x\n\n{}\n\n{body}", lines.join("\n"));
    let (date, status) = ("Date: 2026-10-09T05:39Z", "Status: accepted");
    let model = "Proposed by: model (claude-opus-5-5)";
    let human = "Proposed by: human (Andrew)";
    let ratified = "Ratified by: claude-opus-5-5";
    let cases: [(Vec<&str>, &str, &[&str]); 8] = [
        (vec![date, status, model, ratified], "## Decision", &[]),
        (vec![date, status, human, ratified], "## Assessment", &[]),
        (
            vec![date, status, human, ratified],
            "## Decision",
            &["assessment"],
        ),
        (
            vec![date, status, human, "Ratified by: human"],
            "## Assessment",
            &["ratified-by"],
        ),
        (vec![date, status, human], "## Assessment", &["ratified-by"]),
        (vec![date, status, ratified], "", &["proposed-by"]),
        (
            vec!["Date: yesterday", "Status: vibes", model, ratified],
            "",
            &["date", "status"],
        ),
        (
            vec![date, "Status: amended by 0008", model, ratified],
            "",
            &[],
        ),
    ];

    for (lines, body, problems) in cases {
        assert_eq!(check_record(&record(&lines, body)), problems, "{lines:?}");
    }
}

#[test]
fn every_decision_record_in_this_repo_is_ratified() {
    let name = Regex::new(r"^\d{4}-[a-z0-9-]+\.md$").unwrap();
    let files: Vec<_> = std::fs::read_dir("docs/decisions")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();

    assert!(!files.is_empty());
    for file in files {
        let base = file.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.is_match(&base), "{base}");
        assert_eq!(check_record(&read(&file)), Vec::<&str>::new(), "{base}");
    }
}

#[test]
fn the_handoff_shape_is_the_same_everywhere_it_is_written_down() {
    let skill = read(Path::new("skills/handoff/SKILL.md")).replace("\r\n", "\n");
    let shape: Vec<String> = SECTIONS.iter().map(|name| format!("## {name}\n")).collect();

    assert!(check_handoff(&read(Path::new("templates/notes/handoff.md"))).is_empty());
    assert!(check_handoff(&read(Path::new("docs/handoff.md"))).is_empty());
    assert!(skill.contains(&shape.join("\n")));
}

fn cargo_version() -> String {
    let manifest = read(Path::new("Cargo.toml"));
    let line = manifest
        .lines()
        .find(|line| line.starts_with("version = "))
        .unwrap();

    line.trim_start_matches("version = ")
        .trim_matches('"')
        .to_string()
}

fn json(file: &str) -> Value {
    serde_json::from_str(&read(Path::new(file))).unwrap()
}

#[test]
fn both_plugins_are_the_crates_version_and_call_the_binary() {
    let version = cargo_version();
    let claude = json(".claude-plugin/plugin.json");
    let cursor = json(".cursor-plugin/plugin.json");

    assert_eq!(claude["version"], Value::String(version.clone()));
    assert_eq!(cursor["version"], Value::String(version));

    let claude_hooks = json("hooks/hooks.json");
    let commands: Vec<&str> = claude_hooks["hooks"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|groups| groups.as_array().unwrap())
        .flat_map(|group| group["hooks"].as_array().unwrap())
        .map(|hook| hook["command"].as_str().unwrap())
        .collect();
    assert_eq!(commands.len(), 7);
    assert!(
        commands.iter().all(|command| *command == "heeded hook"),
        "{commands:?}"
    );

    let cursor_hooks = json(cursor["hooks"].as_str().unwrap());
    let hooks = cursor_hooks["hooks"].as_object().unwrap();
    let names: Vec<&str> = hooks.keys().map(String::as_str).collect();
    assert_eq!(cursor_hooks["version"], Value::from(1));
    assert_eq!(
        names,
        [
            "beforeShellExecution",
            "sessionStart",
            "beforeSubmitPrompt",
            "postToolUse",
            "stop"
        ]
    );
    for hook in hooks.values().flat_map(|all| all.as_array().unwrap()) {
        assert_eq!(hook["command"], Value::String("heeded hook".to_string()));
    }
    assert!(hooks["stop"][0]["loop_limit"].as_u64().unwrap() > 3);
}

#[test]
fn every_skill_runs_the_binary_by_name() {
    for (name, command) in [
        ("init", "heeded init"),
        ("handoff", "heeded moves"),
        ("retro", "heeded retro"),
    ] {
        let skill = read(&Path::new("skills").join(name).join("SKILL.md")).replace("\r\n", "\n");

        assert!(
            skill.starts_with(&format!("---\nname: {name}\ndescription: ")),
            "{name}"
        );
        assert!(skill.contains(command), "{name}");
        assert!(!skill.contains("node "), "{name}");
    }
}

#[test]
fn nothing_written_here_has_an_em_dash_or_an_en_dash_in_it() {
    let dashes = ['\u{2013}', '\u{2014}'];
    let listed = git_out(
        Path::new("."),
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    );
    let files: Vec<&str> = listed
        .split('\0')
        .filter(|file| !file.is_empty() && !file.starts_with("archive/") && *file != "Cargo.lock")
        .filter(|file| Path::new(file).is_file())
        .collect();

    assert!(files.contains(&"README.md"));
    for file in files {
        let text = String::from_utf8_lossy(&std::fs::read(file).unwrap()).into_owned();
        assert!(!text.contains(dashes), "{file}");
    }
}
