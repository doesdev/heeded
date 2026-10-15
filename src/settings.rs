//! Settings: the defaults, then the user's config, then the repo's marker,
//! each layered on the last one section at a time.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::consts::{DIRECTIVE_MAX, MARKER, SETTINGS};
use crate::paths::{resolve, within};

pub fn defaults() -> Value {
    json!({
        "permission": { "commit": "ask", "push": "ask" },
        "identity": { "name": "", "email": "", "attribution": "any" },
        "checks": { "changelog": false, "quiet": [] },
        "directives": {}
    })
}

/// The object in a JSON file, or an empty one.
pub fn read_json(file: &Path) -> Value {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

fn inside(base: &Path, file: &Path) -> bool {
    let rel = within(base, file);

    !rel.starts_with("..") && !Path::new(&rel).is_absolute() && !rel.contains(':')
}

// a directive is text, or a file to read it from; anything else switches
// it off. A repo's marker may only point at files inside that repo.
fn place(value: &Value, base: &Path, confined: bool) -> Value {
    if let Some(text) = value.as_str() {
        let text = text.trim();
        return if text.is_empty() {
            Value::Null
        } else {
            json!({ "text": text })
        };
    }

    let Some(file) = value.get("file").and_then(Value::as_str) else {
        return Value::Null;
    };
    let file = resolve(base, file);

    if confined && !inside(base, &file) {
        Value::Null
    } else {
        json!({ "file": file.display().to_string() })
    }
}

fn read_layer(file: &Path, confined: bool) -> Value {
    let mut raw = read_json(file);
    let base = file.parent().unwrap_or(Path::new("."));
    let placed: Map<String, Value> = raw
        .get("directives")
        .and_then(Value::as_object)
        .map(|own| {
            own.iter()
                .map(|(name, value)| (name.clone(), place(value, base, confined)))
                .collect()
        })
        .unwrap_or_default();

    raw["directives"] = Value::Object(placed);
    raw
}

fn layer(base: Value, over: &Value) -> Value {
    let Value::Object(base) = base else {
        return base;
    };
    let sections = base.into_iter().map(|(key, section)| {
        let mut merged = section.as_object().cloned().unwrap_or_default();
        if let Some(own) = over.get(&key).and_then(Value::as_object) {
            merged.extend(own.clone());
        }
        (key, Value::Object(merged))
    });

    Value::Object(sections.collect())
}

pub fn settings_files(root: &Path, home: &Path) -> [PathBuf; 2] {
    [home.join(SETTINGS), root.join(MARKER)]
}

/// The user's defaults, then the repo's marker on top.
pub fn settings_for(root: &Path, home: &Path) -> Value {
    let [user, repo] = settings_files(root, home);
    let layers = [read_layer(&user, false), read_layer(&repo, true)];

    layers.iter().fold(defaults(), layer)
}

/// What a repo inherits: the defaults with the user's config on top.
pub fn user_settings(home: &Path) -> Value {
    layer(defaults(), &read_layer(&home.join(SETTINGS), false))
}

fn text_of(directive: &Value) -> String {
    if let Some(text) = directive.get("text").and_then(Value::as_str) {
        return text.to_string();
    }

    let file = directive.get("file").and_then(Value::as_str).unwrap_or("");
    match std::fs::read_to_string(file) {
        Ok(text) => text.trim().to_string(),
        Err(_) => format!("(heeded could not read {file})"),
    }
}

fn clip(text: String) -> String {
    if text.chars().count() <= DIRECTIVE_MAX {
        return text;
    }

    let kept: String = text.chars().take(DIRECTIVE_MAX).collect();
    format!("{kept}\n(cut at {DIRECTIVE_MAX} characters)")
}

/// `(name, text)` for each directive still switched on.
pub fn directives_of(settings: &Value) -> Vec<(String, String)> {
    let Some(all) = settings.get("directives").and_then(Value::as_object) else {
        return vec![];
    };

    all.iter()
        .filter(|(_, directive)| directive.is_object())
        .map(|(name, directive)| (name.clone(), clip(text_of(directive))))
        .filter(|(_, text)| !text.is_empty())
        .collect()
}

pub fn get_str<'a>(settings: &'a Value, section: &str, key: &str) -> &'a str {
    settings[section][key].as_str().unwrap_or("")
}
