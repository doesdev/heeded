//! A scratch folder per test, real git repos in it, and the built binary
//! run the way Claude Code runs it.

#![allow(dead_code)]

use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

pub const BIN: &str = env!("CARGO_BIN_EXE_heeded");
pub const MARKER: &str = ".heeded.json";
pub const NOTES: &str = ".heeded";
pub const HANDOFF: &str = ".heeded/handoff.md";
pub const JOURNAL: &str = ".heeded/journal.jsonl";
pub const MARKED: &str = "{ \"notes\": \".heeded\" }\n";
pub const MAX_BLOCKS: usize = 3;

pub struct Ran {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    pub fn json(&self) -> Value {
        serde_json::from_str(self.stdout.trim()).unwrap_or(Value::Null)
    }
}

pub struct Lab {
    pub tmp: PathBuf,
    pub state: PathBuf,
    pub home: PathBuf,
    seq: Cell<u32>,
}

impl Drop for Lab {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.tmp);
    }
}

pub fn git(cwd: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());

    assert!(ok, "git {args:?} in {}", cwd.display());
}

pub fn git_out(cwd: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("git");

    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn write(dir: &Path, file: &str, text: &str) {
    let path = dir.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_| panic!("read {}", path.display()))
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

fn run(cwd: &Path, args: &[&str], env: &[(&str, &Path)], input: Option<&str>) -> Ran {
    let mut command = Command::new(BIN);
    command
        .args(args)
        .current_dir(cwd)
        .env_remove("CLAUDE_PROJECT_DIR")
        .env_remove("CURSOR_PROJECT_DIR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in env {
        command.env(name, value);
    }

    let mut child = command.spawn().expect("heeded runs");
    let mut stdin = child.stdin.take().unwrap();
    let _ = stdin.write_all(input.unwrap_or("").as_bytes());
    drop(stdin);
    let out = child.wait_with_output().unwrap();

    Ran {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&out.stderr).replace("\r\n", "\n"),
    }
}

impl Lab {
    pub fn new(name: &str) -> Lab {
        let tmp = std::env::temp_dir().join(format!("heeded-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        Lab {
            state: tmp.join("state"),
            home: tmp.join("home"),
            tmp,
            seq: Cell::new(0),
        }
    }

    pub fn next(&self) -> u32 {
        self.seq.set(self.seq.get() + 1);
        self.seq.get()
    }

    pub fn dir(&self, name: &str) -> PathBuf {
        let dir = self.tmp.join(format!("{name}-{}", self.next()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    pub fn repo(&self, stamped: bool) -> PathBuf {
        let repo = self.dir("repo");
        git(&repo, &["init", "-q"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        git(&repo, &["config", "user.name", "test"]);
        write(&repo, "README.md", "hi\n");
        if stamped {
            write(&repo, MARKER, MARKED);
            write(&repo, HANDOFF, "# Handoff\n");
        }
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-q", "-m", "Initial commit"]);

        repo
    }

    pub fn bin(&self, cwd: &Path, args: &[&str]) -> Ran {
        run(cwd, args, &[("HEEDED_HOME", &self.home)], None)
    }

    pub fn bin_at(&self, home: &Path, cwd: &Path, args: &[&str], input: Option<&str>) -> Ran {
        run(cwd, args, &[("HEEDED_HOME", home)], input)
    }

    pub fn hook(&self, repo: &Path, id: Option<&str>, event: &str, extra: Value) -> Ran {
        let mut input = json!({ "cwd": repo.display().to_string(), "hook_event_name": event });
        if let Some(id) = id {
            input["session_id"] = json!(id);
        }
        if let Value::Object(extra) = extra {
            input.as_object_mut().unwrap().extend(extra);
        }
        let env = [
            ("CLAUDE_PLUGIN_DATA", self.state.as_path()),
            ("CLAUDE_PROJECT_DIR", repo),
            ("HEEDED_HOME", self.home.as_path()),
        ];

        run(&self.tmp, &["hook"], &env, Some(&input.to_string()))
    }

    pub fn session(&self, repo: &Path, id: &str) -> Session<'_> {
        Session {
            lab: self,
            repo: repo.to_path_buf(),
            id: id.to_string(),
        }
    }

    /// A new repo with a session already started in it.
    pub fn started(&self, stamped: bool) -> Session<'_> {
        let repo = self.repo(stamped);
        let id = format!("session-{}", self.next());
        let session = self.session(&repo, &id);
        session.start();
        session
    }
}

/// Plays Claude Code: every tool call is followed by the PostToolUse hook.
pub struct Session<'a> {
    pub lab: &'a Lab,
    pub repo: PathBuf,
    pub id: String,
}

impl Session<'_> {
    pub fn hook(&self, event: &str, extra: Value) -> Ran {
        self.lab.hook(&self.repo, Some(&self.id), event, extra)
    }

    pub fn edit(&self, file: &str, text: &str) -> Ran {
        write(&self.repo, file, text);
        let file = self.repo.join(file).display().to_string();

        self.hook(
            "PostToolUse",
            json!({ "tool_name": "Edit", "tool_input": { "file_path": file } }),
        )
    }

    pub fn bash(&self, description: &str, run: impl FnOnce()) -> Ran {
        run();
        let input = json!({ "command": "SECRET=hunter2 do-it", "description": description });

        self.hook(
            "PostToolUse",
            json!({ "tool_name": "Bash", "tool_input": input }),
        )
    }

    pub fn start(&self) -> Ran {
        self.hook("SessionStart", json!({}))
    }

    pub fn prompt(&self) -> Ran {
        self.hook("UserPromptSubmit", json!({}))
    }

    pub fn stop(&self) -> Ran {
        self.hook("Stop", json!({}))
    }

    pub fn journal(&self) -> Vec<Value> {
        journal(&self.repo.join(JOURNAL))
    }
}

pub fn journal(file: &Path) -> Vec<Value> {
    std::fs::read_to_string(file)
        .unwrap_or_default()
        .split('\n')
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// `[tool, what]` for each journal entry.
pub fn moves(entries: &[Value]) -> Vec<(String, String)> {
    let field = |entry: &Value, key: &str| entry[key].as_str().unwrap_or("").to_string();

    entries
        .iter()
        .map(|entry| (field(entry, "tool"), field(entry, "what")))
        .collect()
}

pub fn pair(tool: &str, what: &str) -> (String, String) {
    (tool.to_string(), what.to_string())
}

pub fn stamped_in(notes: &str) -> Vec<String> {
    let files = [
        ".gitignore",
        "decisions/README.md",
        "handoff.md",
        "plan.md",
        "rework.jsonl",
    ];
    let mut all = vec![MARKER.to_string()];
    all.extend(files.iter().map(|file| format!("{notes}/{file}")));
    all.push("package.json".to_string());
    all
}

/// The file at `rel` under `dir`, as the platform writes paths.
pub fn native(dir: &Path, rel: &str) -> String {
    rel.split('/')
        .fold(dir.to_path_buf(), |at, part| at.join(part))
        .display()
        .to_string()
}
