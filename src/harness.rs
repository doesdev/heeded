//! One adapter per harness: each turns its hook input into a common event,
//! and the answer back into what that harness expects.

use serde_json::{Map, Value, json};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Event {
    pub name: Option<String>,
    pub id: Option<String>,
    pub dir: String,
    pub cwd: Option<String>,
    pub agent: Option<String>,
    pub tool: Option<String>,
    pub input: Value,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Outcome {
    pub context: Option<String>,
    pub refusal: Option<String>,
    pub notice: Option<String>,
    pub ask: Option<String>,
    pub deny: Option<String>,
}

impl Outcome {
    pub fn ask(why: String) -> Self {
        Outcome {
            ask: Some(why),
            ..Outcome::default()
        }
    }

    pub fn deny(why: String) -> Self {
        Outcome {
            deny: Some(why),
            ..Outcome::default()
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Emitted {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub code: i32,
}

fn string(input: &Value, key: &str) -> Option<String> {
    input.get(key).and_then(Value::as_str).map(str::to_string)
}

fn here() -> String {
    std::env::current_dir()
        .unwrap_or_default()
        .display()
        .to_string()
}

pub trait Harness {
    fn parse(&self, input: &Value, env: &dyn Fn(&str) -> Option<String>) -> Event;
    fn emit(&self, event: &Event, outcome: &Outcome) -> Emitted;
}

pub struct Claude;

impl Harness for Claude {
    fn parse(&self, input: &Value, env: &dyn Fn(&str) -> Option<String>) -> Event {
        let name = match input.get("hook_event_name").and_then(Value::as_str) {
            Some("SessionStart") => Some("start"),
            Some("PreToolUse") => Some("pre"),
            Some("UserPromptSubmit") => Some("prompt"),
            Some("PostToolUse") => Some("tool"),
            Some("SubagentStart") => Some("agent-start"),
            Some("SubagentStop") => Some("agent-stop"),
            Some("Stop") => Some("stop"),
            _ => None,
        };

        Event {
            name: name.map(str::to_string),
            id: string(input, "session_id"),
            dir: env("CLAUDE_PROJECT_DIR")
                .or_else(|| string(input, "cwd"))
                .unwrap_or_else(here),
            cwd: string(input, "cwd"),
            agent: string(input, "agent_id"),
            tool: string(input, "tool_name"),
            input: input.get("tool_input").cloned().unwrap_or(Value::Null),
        }
    }

    // context is plain stdout; a refusal is stderr and exit 2
    fn emit(&self, _event: &Event, outcome: &Outcome) -> Emitted {
        let permit = |decision: &str, why: &str| Emitted {
            stdout: Some(
                json!({ "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": decision,
                    "permissionDecisionReason": why
                }})
                .to_string(),
            ),
            ..Emitted::default()
        };

        if let Some(why) = &outcome.deny {
            return permit("deny", why);
        }
        if let Some(why) = &outcome.ask {
            return permit("ask", why);
        }
        if let Some(refusal) = &outcome.refusal {
            return Emitted {
                stderr: Some(refusal.clone()),
                code: 2,
                ..Emitted::default()
            };
        }
        if let Some(notice) = &outcome.notice {
            return Emitted {
                stdout: Some(json!({ "systemMessage": notice }).to_string()),
                ..Emitted::default()
            };
        }

        Emitted {
            stdout: outcome.context.clone(),
            ..Emitted::default()
        }
    }
}

/// Written from Cursor's published docs on 2026-10-09 and never run inside
/// Cursor. Decision 0014 lists what was confirmed and what was not.
pub struct Cursor;

impl Harness for Cursor {
    fn parse(&self, input: &Value, env: &dyn Fn(&str) -> Option<String>) -> Event {
        // lower-cased, so Claude-style names from an imported plugin land here too
        let event = string(input, "hook_event_name")
            .unwrap_or_default()
            .to_lowercase();
        let name = match event.as_str() {
            "sessionstart" => Some("start"),
            "pretooluse" | "beforeshellexecution" => Some("pre"),
            "beforesubmitprompt" | "userpromptsubmit" => Some("prompt"),
            "posttooluse" | "afterfileedit" | "aftershellexecution" => Some("tool"),
            "stop" => Some("stop"),
            _ => None,
        };
        // the older per-tool events carry no tool name
        let (tool, tool_input) = match event.as_str() {
            "afterfileedit" => (
                Some("Write".to_string()),
                json!({ "file_path": input.get("file_path") }),
            ),
            "aftershellexecution" => (Some("Bash".to_string()), json!({})),
            "beforeshellexecution" => (
                Some("Bash".to_string()),
                json!({ "command": input.get("command") }),
            ),
            _ => {
                let tool = string(input, "tool_name").map(|tool| match tool.as_str() {
                    "Shell" => "Bash".to_string(),
                    _ => tool,
                });
                (
                    tool,
                    input.get("tool_input").cloned().unwrap_or(Value::Null),
                )
            }
        };
        let root = input
            .get("workspace_roots")
            .and_then(Value::as_array)
            .and_then(|roots| roots.first())
            .and_then(Value::as_str)
            .map(str::to_string);
        // Cursor's cwd is reported empty for shell calls, so it is not trusted
        let cwd = string(input, "cwd").filter(|cwd| !cwd.is_empty());

        Event {
            name: name.map(str::to_string),
            id: string(input, "conversation_id").or_else(|| string(input, "session_id")),
            dir: env("CURSOR_PROJECT_DIR")
                .or_else(|| env("CLAUDE_PROJECT_DIR"))
                .or(root)
                .or(cwd)
                .unwrap_or_else(here),
            cwd: None,
            agent: None,
            tool,
            input: tool_input,
        }
    }

    // everything is JSON on stdout and exit 0; a refusal is a follow-up message
    fn emit(&self, event: &Event, outcome: &Outcome) -> Emitted {
        let mut out = Map::new();
        let said = outcome.deny.as_ref().or(outcome.ask.as_ref());

        match event.name.as_deref() {
            Some("pre") => {
                let permission = if outcome.deny.is_some() {
                    "deny"
                } else if outcome.ask.is_some() {
                    "ask"
                } else {
                    "allow"
                };
                out.insert("permission".into(), json!(permission));
                if let Some(said) = said {
                    out.insert("user_message".into(), json!(said));
                    out.insert("agent_message".into(), json!(said));
                }
            }
            Some("start") => {
                if let Some(context) = &outcome.context {
                    out.insert("additional_context".into(), json!(context));
                }
            }
            Some("prompt") => {
                out.insert("continue".into(), json!(true));
            }
            Some("stop") => {
                if let Some(refusal) = &outcome.refusal {
                    out.insert("followup_message".into(), json!(refusal));
                }
            }
            _ => {}
        }
        if let Some(notice) = &outcome.notice {
            out.insert("user_message".into(), json!(notice));
        }

        Emitted {
            stdout: Some(Value::Object(out).to_string()),
            ..Emitted::default()
        }
    }
}

/// The harness that sent this input.
pub fn harness_for(input: &Value) -> Box<dyn Harness> {
    let cursor = input
        .get("cursor_version")
        .is_some_and(|version| !version.is_null() && version != "");

    if cursor {
        Box::new(Cursor)
    } else {
        Box::new(Claude)
    }
}
