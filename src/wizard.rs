//! Steps through every setting in one config file. `ask` returns the
//! answer; Enter keeps what is there. Locally, `inherit` drops the repo's
//! own value so the user's applies.

use serde_json::{Map, Value, json};

pub struct Step {
    pub key: &'static str,
    pub say: &'static str,
    pub choices: &'static [&'static str],
    pub on_off: bool,
    pub list: bool,
}

const fn choice(key: &'static str, say: &'static str, choices: &'static [&'static str]) -> Step {
    Step {
        key,
        say,
        choices,
        on_off: false,
        list: false,
    }
}

const fn text(key: &'static str, say: &'static str) -> Step {
    Step {
        key,
        say,
        choices: &[],
        on_off: false,
        list: false,
    }
}

pub const STEPS: [Step; 7] = [
    choice(
        "permission.commit",
        "Commit without asking you first",
        &["ask", "allow"],
    ),
    choice(
        "permission.push",
        "Push, tag, publish or release without asking you first",
        &["ask", "allow"],
    ),
    text("identity.name", "Author name every commit must carry"),
    text("identity.email", "Author email every commit must carry"),
    choice(
        "identity.attribution",
        "Co-Authored-By lines on commits",
        &["any", "required", "forbidden"],
    ),
    Step {
        key: "checks.changelog",
        say: "Remind once when a commit has no changelog line",
        choices: &["off", "on"],
        on_off: true,
        list: false,
    },
    Step {
        key: "checks.quiet",
        say: "Paths the changelog reminder ignores, comma separated",
        choices: &[],
        on_off: false,
        list: true,
    },
];

/// What an answer means.
#[derive(Debug, PartialEq)]
pub enum Pick {
    Keep,
    Inherit,
    Set(Value),
    Invalid,
}

fn split(key: &str) -> (&str, &str) {
    key.split_once('.').unwrap_or((key, ""))
}

pub fn get<'a>(config: &'a Value, key: &str) -> Option<&'a Value> {
    let (head, tail) = split(key);

    config.get(head)?.get(tail)
}

/// A copy with one setting set, or dropped when `value` is `None`. A
/// section left empty is dropped too.
pub fn set(config: &Value, key: &str, value: Option<Value>) -> Value {
    let (head, tail) = split(key);
    let mut next = config.as_object().cloned().unwrap_or_default();
    let mut inner = next
        .get(head)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    match value {
        Some(value) => {
            inner.insert(tail.to_string(), value);
        }
        None => {
            inner.shift_remove(tail);
        }
    }

    if inner.is_empty() {
        next.shift_remove(head);
    } else {
        next.insert(head.to_string(), Value::Object(inner));
    }

    Value::Object(next)
}

fn shown(step: &Step, value: Option<&Value>) -> Option<String> {
    let value = value?;

    if step.on_off {
        return value
            .as_bool()
            .map(|on| if on { "on" } else { "off" }.to_string());
    }
    if step.list {
        let items: Vec<&str> = value.as_array()?.iter().filter_map(Value::as_str).collect();
        return Some(if items.is_empty() {
            "(none)".to_string()
        } else {
            items.join(", ")
        });
    }

    let said = value.as_str()?;
    Some(if said.is_empty() {
        "(any)".to_string()
    } else {
        said.to_string()
    })
}

pub fn pick(step: &Step, answer: &str, local: bool) -> Pick {
    let said = answer.trim();

    if said.is_empty() {
        return Pick::Keep;
    }
    if local && said == "inherit" {
        return Pick::Inherit;
    }
    if step.list {
        let items: Vec<Value> = if said == "-" {
            vec![]
        } else {
            said.split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(|item| json!(item))
                .collect()
        };
        return Pick::Set(Value::Array(items));
    }
    if step.choices.is_empty() {
        return Pick::Set(json!(if said == "-" { "" } else { said }));
    }

    let lower = said.to_lowercase();
    let found: Vec<&&str> = step
        .choices
        .iter()
        .filter(|choice| choice.starts_with(&lower))
        .collect();
    match found.as_slice() {
        [one] if step.on_off => Pick::Set(json!(**one == "on")),
        [one] => Pick::Set(json!(**one)),
        _ => Pick::Invalid,
    }
}

fn options_for(step: &Step, local: bool) -> String {
    let base = if step.choices.is_empty() {
        "text, - for none".to_string()
    } else {
        step.choices.join("/")
    };

    if local {
        format!("{base}/inherit")
    } else {
        base
    }
}

fn question(step: &Step, own: Option<&Value>, inherited: &Value, local: bool) -> String {
    let theirs = shown(step, get(inherited, step.key)).unwrap_or_else(|| "(any)".to_string());
    let now = match shown(step, own) {
        Some(mine) => mine,
        None if local => format!("inherited: {theirs}"),
        None => theirs,
    };

    format!("{} [{}] (now {now}): ", step.say, options_for(step, local))
}

fn describe(value: &Value) -> String {
    if value == &Value::Bool(false) {
        return "off here".to_string();
    }
    if let Some(file) = value.get("file").and_then(Value::as_str) {
        return format!("from {file}");
    }
    let Some(said) = value.as_str() else {
        return "(not usable)".to_string();
    };

    if said.chars().count() > 60 {
        format!("{}...", said.chars().take(57).collect::<String>())
    } else {
        said.to_string()
    }
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();

    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn directive(answer: &str, local: bool) -> Pick {
    match answer.trim() {
        "" => Pick::Keep,
        "remove" => Pick::Inherit,
        "off" if local => Pick::Set(json!(false)),
        "off" => Pick::Invalid,
        said if said.starts_with('@') && said.len() > 1 => Pick::Set(json!({ "file": &said[1..] })),
        said if said.starts_with('@') => Pick::Invalid,
        said => Pick::Set(json!(said)),
    }
}

fn directives(
    start: Value,
    ask: &mut dyn FnMut(&str) -> String,
    say: &mut dyn FnMut(&str),
    local: bool,
) -> Value {
    let mut own: Map<String, Value> = start
        .get("directives")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    loop {
        let listed: Vec<String> = own
            .iter()
            .map(|(name, value)| format!("  {name}: {}", describe(value)))
            .collect();
        let listed = if listed.is_empty() {
            "  (none)".to_string()
        } else {
            listed.join("\n")
        };
        say(&format!("Directives in this file:\n{listed}"));

        let answer = ask("Name one to add or change, or Enter to finish: ");
        let name = answer.trim();
        if name.is_empty() {
            break;
        }
        if !valid_name(name) {
            say("A name is letters, digits and dashes.");
            continue;
        }

        let off = if local {
            ", off to switch it off here"
        } else {
            ""
        };
        let prompt = format!(
            "{name}: new text, @path to read it from a file{off}, remove to delete, Enter to keep: "
        );
        match directive(&ask(&prompt), local) {
            Pick::Invalid => say("Not an answer; nothing changed."),
            Pick::Inherit => {
                own.shift_remove(name);
            }
            Pick::Set(value) => {
                own.insert(name.to_string(), value);
            }
            Pick::Keep => {}
        }
    }

    let mut next = start.as_object().cloned().unwrap_or_default();
    if own.is_empty() && local {
        next.shift_remove("directives");
    } else {
        next.insert("directives".to_string(), Value::Object(own));
    }

    Value::Object(next)
}

pub fn walk(
    start: Value,
    ask: &mut dyn FnMut(&str) -> String,
    say: &mut dyn FnMut(&str),
    local: bool,
    inherited: &Value,
) -> Value {
    let mut config = start;

    for step in &STEPS {
        let picked = loop {
            let answer = ask(&question(step, get(&config, step.key), inherited, local));
            match pick(step, &answer, local) {
                Pick::Invalid => say(&format!("One of: {}.", options_for(step, local))),
                picked => break picked,
            }
        };

        config = match picked {
            Pick::Set(value) => set(&config, step.key, Some(value)),
            Pick::Inherit => set(&config, step.key, None),
            _ => config,
        };
    }

    directives(config, ask, say, local)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::defaults;

    fn scripted(answers: &[&str]) -> (impl FnMut(&str) -> String, Vec<String>) {
        let mut queue: Vec<String> = answers.iter().rev().map(|a| a.to_string()).collect();
        (move |_: &str| queue.pop().unwrap_or_default(), vec![])
    }

    #[test]
    fn pick_reads_an_answer_for_each_kind_of_step() {
        let [commit, _, name, _, attribution, changelog, quiet] = &STEPS;
        let cases = [
            (commit, "allow", false, Pick::Set(json!("allow"))),
            (commit, "al", false, Pick::Set(json!("allow"))),
            (commit, "a", false, Pick::Invalid),
            (commit, "nope", false, Pick::Invalid),
            (commit, "inherit", true, Pick::Inherit),
            (commit, "inherit", false, Pick::Invalid),
            (attribution, "f", false, Pick::Set(json!("forbidden"))),
            (changelog, "on", false, Pick::Set(json!(true))),
            (changelog, "off", false, Pick::Set(json!(false))),
            (name, "A C", false, Pick::Set(json!("A C"))),
            (name, "-", false, Pick::Set(json!(""))),
            (name, "inherit", true, Pick::Inherit),
            (name, "   ", false, Pick::Keep),
            (
                quiet,
                "a/, b ,,c",
                false,
                Pick::Set(json!(["a/", "b", "c"])),
            ),
            (quiet, "-", false, Pick::Set(json!([]))),
        ];

        for (step, answer, local, expected) in cases {
            assert_eq!(pick(step, answer, local), expected, "{} {answer}", step.key);
        }
    }

    #[test]
    fn set_and_get_walk_one_level() {
        let start = json!({ "notes": "n", "permission": { "commit": "ask" } });

        assert_eq!(get(&start, "permission.commit"), Some(&json!("ask")));
        assert_eq!(get(&start, "identity.name"), None);
        assert_eq!(
            set(&start, "identity.name", Some(json!("A"))),
            json!({ "notes": "n", "permission": { "commit": "ask" }, "identity": { "name": "A" } })
        );
        assert_eq!(
            set(&start, "permission.commit", None),
            json!({ "notes": "n" })
        );
    }

    #[test]
    fn walk_steps_through_every_setting_and_the_directives_globally() {
        let mut start = defaults();
        start["directives"] = json!({ "tone": "Dry." });
        start["extra"] = json!(1);
        let (mut ask, _) = scripted(&[
            "nope",
            "allow",
            "",
            "A C",
            "-",
            "f",
            "on",
            "e2e/, xtask/",
            "voice",
            "Short.",
            "style",
            "@style.md",
            "tone",
            "remove",
            "bad name!",
            "x",
            "off",
            "",
        ]);
        let mut said: Vec<String> = vec![];
        let next = walk(
            start,
            &mut ask,
            &mut |text| said.push(text.to_string()),
            false,
            &defaults(),
        );

        assert_eq!(
            next,
            json!({
                "permission": { "commit": "allow", "push": "ask" },
                "identity": { "name": "A C", "email": "", "attribution": "forbidden" },
                "checks": { "changelog": true, "quiet": ["e2e/", "xtask/"] },
                "directives": { "voice": "Short.", "style": { "file": "style.md" } },
                "extra": 1
            })
        );
        assert!(said.contains(&"One of: ask/allow.".to_string()));
        assert!(said.contains(&"A name is letters, digits and dashes.".to_string()));
        assert!(said.contains(&"Not an answer; nothing changed.".to_string()));
    }

    #[test]
    fn walk_locally_offers_inherit_and_keeps_the_marker_small() {
        let mut inherited = defaults();
        inherited["permission"]["commit"] = json!("allow");
        let start = json!({ "notes": ".agent", "permission": { "commit": "ask" }, "directives": { "voice": "x" } });
        let mut asked: Vec<String> = vec![];
        let mut answers: Vec<&str> = vec![
            "inherit", "", "", "", "", "", "web/e2e/", "voice", "off", "tone", "remove", "",
        ];
        answers.reverse();
        let mut ask = |question: &str| {
            asked.push(question.to_string());
            answers.pop().unwrap_or("").to_string()
        };
        let next = walk(start, &mut ask, &mut |_| {}, true, &inherited);

        assert_eq!(
            next,
            json!({ "notes": ".agent", "checks": { "quiet": ["web/e2e/"] }, "directives": { "voice": false } })
        );
        assert!(asked[0].contains("[ask/allow/inherit] (now ask)"));
        assert!(asked[1].contains("(now inherited: ask)"));

        let (mut ask, _) = scripted(&[]);
        let empty = walk(
            json!({ "notes": "n" }),
            &mut ask,
            &mut |_| {},
            true,
            &inherited,
        );
        assert_eq!(empty, json!({ "notes": "n" }));
    }
}
