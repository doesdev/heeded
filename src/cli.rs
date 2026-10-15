//! The commands a person types: init, moves, retro, config, help.

use std::collections::BTreeMap;
use std::io::{BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::clock::{iso, minute, now_ms};
use crate::consts::{HANDOFF, JOURNAL, MARKER, NOTES, REWORK, SETTINGS, home_dir};
use crate::git::{git, root_of};
use crate::journal::{Journal, since_handoff};
use crate::notes::notes_of;
use crate::paths::resolve;
use crate::rework::{CLASSES, OUTCOMES, check_rework};
use crate::scaffold::scaffold;
use crate::session::listed_values;
use crate::settings::{defaults, directives_of, read_json, settings_for, user_settings};
use crate::wizard::walk;

const SUMMARY: &str = "heeded: good habits, installed rather than remembered

Usage: heeded <command> [dir] [options]

  init     stamp a repo
  moves    what changed since the last handoff
  retro    log how a finished task turned out
  config   step through your settings, global or for this repo
  help     this, or more about one command (heeded help config)

[dir] defaults to the current folder; any folder in a repo means
that repo.";

fn help_for(topic: &str) -> Option<String> {
    let user_file = home_dir().join(SETTINGS);

    Some(match topic {
        "init" => format!(
            "heeded init [dir] [--notes <folder>] [--tracked]

  Stamps a repo so heeded's hooks gate it. Writes {MARKER} at the root,
  the notes folder it names (default {NOTES}) with handoff.md, plan.md,
  decisions/ and rework.jsonl, and a package.json with a failing check
  script if the repo has none. Never overwrites.

  What it creates is hidden from this clone's git (.git/info/exclude)
  unless --tracked. --notes puts the notes somewhere else, such as a
  private checkout inside the repo."
        ),
        "moves" => "heeded moves [dir]

  Prints the time, where the handoff is, and every move the hooks
  logged since the handoff was last written."
            .to_string(),
        "retro" => format!(
            "heeded retro [dir] --task <text> --outcome <outcome> --class <class>
             [--note <text>]

  Logs how a finished task turned out, one line in rework.jsonl.
    outcome  {}
    class    {}
  The class is none when, and only when, the outcome is kept.",
            OUTCOMES.join(", "),
            CLASSES.join(", ")
        ),
        "config" => format!(
            "heeded config [dir] [--global | --local | --show]

  Steps through every setting and asks for each, then writes the file.
  Enter keeps what is there.
    --global  your own config, {}
    --local   this repo's {MARKER}, over yours; inherit drops a value
              so yours applies
    --show    change nothing; write your config if it is missing, and
              print what is in effect here
  With none of them, it asks which file, or shows when nobody is at
  the keyboard.",
            user_file.display()
        ),
        "help" => "heeded help [command]

  Prints this, or more about one command."
            .to_string(),
        _ => return None,
    })
}

/// Parsed arguments: positionals in order, and flags by name.
#[derive(Default, Debug)]
pub struct Args {
    pub positionals: Vec<String>,
    pub values: BTreeMap<String, String>,
    pub flags: Vec<String>,
}

const VALUED: [&str; 5] = ["notes", "task", "outcome", "class", "note"];
const SWITCHES: [&str; 5] = ["tracked", "global", "local", "show", "help"];

pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut rest = argv.iter();

    while let Some(arg) = rest.next() {
        if arg == "-h" {
            args.flags.push("help".into());
            continue;
        }
        let Some(flag) = arg.strip_prefix("--") else {
            args.positionals.push(arg.clone());
            continue;
        };
        let (name, inline) = match flag.split_once('=') {
            Some((name, value)) => (name, Some(value.to_string())),
            None => (flag, None),
        };

        if VALUED.contains(&name) {
            let value = inline.or_else(|| rest.next().cloned());
            let value =
                value.ok_or_else(|| format!("Option '--{name} <value>' argument missing"))?;
            args.values.insert(name.to_string(), value);
        } else if SWITCHES.contains(&name) && inline.is_none() {
            args.flags.push(name.to_string());
        } else {
            return Err(format!("Unknown option '--{name}'"));
        }
    }

    Ok(args)
}

impl Args {
    fn has(&self, flag: &str) -> bool {
        self.flags.iter().any(|own| own == flag)
    }

    fn value(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }
}

fn stamped(root: &Path) -> Result<PathBuf, String> {
    notes_of(root)?.ok_or_else(|| {
        format!(
            "{} has no {MARKER}. Stamp it with: heeded init",
            root.display()
        )
    })
}

fn init(root: &Path, args: &Args) -> Result<(), String> {
    let done = scaffold(root, args.value("notes"), args.has("tracked"))?;

    println!("{}", root.display());
    for (file, did) in &done.files {
        println!("  {did:<8}{file}");
    }
    if !done.check {
        println!("package.json has no \"check\" script. Add one.");
    }

    Ok(())
}

fn moves(root: &Path) -> Result<(), String> {
    let notes = stamped(root)?;
    let all = Journal::new(Some(notes.join(JOURNAL))).read();
    let left = since_handoff(&all);

    println!("Now: {}", minute(now_ms()));
    println!("Handoff: {}", notes.join(HANDOFF).display());
    if left.is_empty() {
        println!("No moves since it was last written.");
    } else {
        println!("Moves since it was last written:\n{}", listed_values(left));
    }

    Ok(())
}

fn retro(root: &Path, args: &Args) -> Result<(), String> {
    let notes = stamped(root)?;
    let (task, outcome, class) = (
        args.value("task").unwrap_or(""),
        args.value("outcome").unwrap_or(""),
        args.value("class").unwrap_or(""),
    );
    let problems = check_rework(task, outcome, class);

    if !problems.is_empty() {
        let help = help_for("retro").unwrap_or_default();
        return Err(format!(
            "Not logged, bad {}.\n\n{help}",
            problems.join(", ")
        ));
    }

    let file = notes.join(REWORK);
    let entry = json!({
        "ts": iso(now_ms()),
        "task": task,
        "outcome": outcome,
        "failureClass": class,
        "note": args.value("note").unwrap_or("")
    });
    Journal::new(Some(file.clone())).append(&entry);
    println!("Logged to {}", file.display());

    Ok(())
}

// config

fn seeded(root: &Path) -> Value {
    let global = |key: &str| {
        git(root, &["config", "--global", key])
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let mut config = defaults();
    config["identity"]["name"] = json!(global("user.name"));
    config["identity"]["email"] = json!(global("user.email"));

    config
}

fn write_json(file: &Path, data: &Value) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|ex| format!("{}: {ex}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(data).unwrap_or_default();

    std::fs::write(file, format!("{text}\n")).map_err(|ex| format!("{}: {ex}", file.display()))
}

fn in_effect(root: &Path) {
    let mut settings = settings_for(root, &home_dir());
    let directives: Vec<String> = directives_of(&settings)
        .iter()
        .map(|(name, said)| format!("{name} ({} characters)", said.chars().count()))
        .collect();
    settings["directives"] = json!(directives);

    println!("In effect in {}:", root.display());
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).unwrap_or_default()
    );
}

fn show(root: &Path) -> Result<(), String> {
    let file = home_dir().join(SETTINGS);
    let missing = !file.exists();
    if missing {
        write_json(&file, &seeded(root))?;
    }

    println!(
        "{} {}",
        if missing { "created" } else { "kept" },
        file.display()
    );
    in_effect(root);

    Ok(())
}

// a person gets a prompt; piped answers are read up front, one per line
fn asker() -> Box<dyn FnMut(&str) -> String> {
    if std::io::stdin().is_terminal() {
        return Box::new(|question: &str| {
            print!("{question}");
            let _ = std::io::stdout().flush();
            let mut line = String::new();
            let _ = std::io::stdin().lock().read_line(&mut line);
            line.trim_end_matches(['\r', '\n']).to_string()
        });
    }

    let mut text = String::new();
    let _ = std::io::stdin().read_to_string(&mut text);
    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect();
    lines.reverse();

    Box::new(move |question: &str| {
        let answer = lines.pop().unwrap_or_default();
        println!("{question}{answer}");
        answer
    })
}

fn config(root: &Path, args: &Args) -> Result<(), String> {
    let (global, local) = (args.has("global"), args.has("local"));
    if args.has("show") || (!global && !local && !std::io::stdin().is_terminal()) {
        return show(root);
    }

    let mut ask = asker();
    let mut say = |text: &str| println!("{text}");
    let user_file = home_dir().join(SETTINGS);
    let mut scope_local = local;

    if !local && !global && notes_of(root)?.is_some() {
        let answer = ask(&format!(
            "Edit [g]lobal ({}) or [l]ocal ({MARKER} here)? ",
            user_file.display()
        ));
        scope_local = answer.trim().to_lowercase().starts_with('l');
    }

    let (file, start, inherited) = if scope_local {
        stamped(root)?;
        let file = root.join(MARKER);
        let start = read_json(&file);
        (file, start, user_settings(&home_dir()))
    } else {
        let start = if user_file.exists() {
            read_json(&user_file)
        } else {
            seeded(root)
        };
        (user_file, start, defaults())
    };

    say(&format!(
        "Editing {}. Enter keeps what is there.",
        file.display()
    ));
    let next = walk(start, &mut *ask, &mut say, scope_local, &inherited);
    write_json(&file, &next)?;
    say(&format!("wrote {}", file.display()));
    in_effect(root);

    Ok(())
}

/// Runs a command line. `Err` is what to tell the person, who gets exit 1.
pub fn run(argv: &[String]) -> Result<(), String> {
    let args = parse(argv)?;
    let name = args.positionals.first().map(String::as_str);
    let arg = args.positionals.get(1).map(String::as_str);
    let unknown = |name: &str| format!("No command \"{name}\".\n\n{SUMMARY}");

    let topic = match (name, args.has("help")) {
        (None, _) => Some(None),
        (Some(name), true) => Some(Some(name)),
        (Some("help"), false) => Some(arg),
        _ => None,
    };
    if let Some(topic) = topic {
        let text = match topic {
            Some(topic) => help_for(topic).ok_or_else(|| unknown(topic))?,
            None => SUMMARY.to_string(),
        };
        println!("{text}");
        return Ok(());
    }

    let name = name.unwrap_or_default();
    if !matches!(name, "init" | "moves" | "retro" | "config") {
        return Err(unknown(name));
    }
    if args.has("global") && args.has("local") {
        return Err("Pick one of --global and --local.".to_string());
    }

    let here = std::env::current_dir().unwrap_or_default();
    let root = root_of(&resolve(&here, arg.unwrap_or(".")));

    match name {
        "init" => init(&root, &args),
        "moves" => moves(&root),
        "retro" => retro(&root, &args),
        _ => config(&root, &args),
    }
}
