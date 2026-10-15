//! What a shell command is about to do, and what is wrong with a commit.
//! Pattern matching: a guard against drift, not against a determined model.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::paths::{native, resolve};

pub const CHANGELOG: &str = "CHANGELOG.md";

// a shell word: bare, or quoted with either quote
const WORD: &str = r#"(?:[^\s"']|"[^"]*"|'[^']*')+"#;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a valid pattern")
}

// git, then any global options, then the subcommand
fn git_then(rest: &str) -> String {
    format!(r"\bgit(?:\s+(?:-[cC]\s+{WORD}|-{{1,2}}[\w-]+(?:={WORD})?))*\s+{rest}")
}

static COMMIT: LazyLock<Regex> = LazyLock::new(|| re(&git_then(r"commit\b")));
static PUSH: LazyLock<Regex> = LazyLock::new(|| re(&git_then(r"push\b")));
static TAGS: LazyLock<Regex> = LazyLock::new(|| re(&git_then(r"tag\b([^|;&\n]*)")));
static TAG_READS: LazyLock<Regex> = LazyLock::new(|| {
    re(
        r"^(-l|--list|-n\d*|-v|--verify|--points-at|--contains|--no-contains|--merged|--no-merged|--sort|--format|--column)(=|$)",
    )
});
static TAG_WRITES: LazyLock<Regex> =
    LazyLock::new(|| re(r"^(-[adsfm]+|--delete|--annotate|--sign|--force|--message)(=|$)"));
static PUBLISH: LazyLock<Regex> = LazyLock::new(|| re(r"\b(npm|pnpm|yarn|cargo)\s+publish\b"));
static GH: LazyLock<Regex> =
    LazyLock::new(|| re(r"\bgh\s+(release\s+(create|upload|edit|delete)|pr\s+(create|merge))\b"));
static CONFIG: LazyLock<Regex> = LazyLock::new(|| re(r#"\bheeded(?:\.js)?["']?\s+config\b"#));
static SHOW: LazyLock<Regex> = LazyLock::new(|| re(r"--show\b"));
static ADDS: LazyLock<Regex> = LazyLock::new(|| {
    re(&format!(
        r"{}|{}[^|;&\n]*?\s(-[a-zA-Z]*a[a-zA-Z]*|--all)\b",
        git_then(r"add\b"),
        git_then(r"commit\b")
    ))
});
static TARGET: LazyLock<Regex> =
    LazyLock::new(|| re(&format!(r"\bgit\s+(?:[^|;&\n]*?\s)?-C\s+({WORD})")));
static GIT_AT: LazyLock<Regex> = LazyLock::new(|| re(r"\bgit\s"));
static CD: LazyLock<Regex> = LazyLock::new(|| re(&format!(r"^cd\s+({WORD})$")));
static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| re(r"&&|\|\||[;&|(]"));
static OVERRIDES: LazyLock<Regex> =
    LazyLock::new(|| re(r"--author\b|\buser\.(name|email)=|\bGIT_(AUTHOR|COMMITTER)_"));
static TRAILER: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)co-authored-by:"));
static SECRET_FILES: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(^|/)(\.env(\.[\w.-]+)?|id_(rsa|dsa|ecdsa|ed25519)|[^/]*\.(pem|pfx|p12|key))$")
});
static SAFE_FILES: LazyLock<Regex> =
    LazyLock::new(|| re(r"(^|/)\.env\.(example|sample|template)$"));
static QUIET: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(^|/)(test|tests|__tests__|docs|\.github)/|(^|/)test\.[cm]?js$|\.md$|(^|/)\.[^/]+$")
});
static HEREDOC: LazyLock<Regex> = LazyLock::new(|| re(r#"<<-?\s*(?:'(\w+)'|"(\w+)"|(\w+))"#));
const WRITERS: [&str; 2] = ["cat", "tee"];
static SECRETS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        ("a private key", re(r"-----BEGIN [A-Z ]*PRIVATE KEY-----")),
        ("an AWS access key", re(r"\b(AKIA|ASIA)[0-9A-Z]{16}\b")),
        (
            "a GitHub token",
            re(r"\b(gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{40,})\b"),
        ),
        (
            "an Anthropic or OpenAI key",
            re(r"\bsk-[A-Za-z0-9_-]{24,}\b"),
        ),
        ("a Slack token", re(r"\bxox[abprs]-[A-Za-z0-9-]{10,}\b")),
        (
            "a hard-coded secret",
            re(
                r#"(?i)\b(api[_-]?key|secret|token|passw(or)?d)\b["']?\s*[:=]\s*["'][^"'\s]{16,}["']"#,
            ),
        ),
    ]
});

const WAIVER: &str = "heeded: not a secret";

// git tag lists, verifies and creates; only making or deleting one leaves a mark
fn tags_write(command: &str) -> bool {
    TAGS.captures_iter(command).any(|found| {
        let words: Vec<&str> = found[1].split_whitespace().collect();

        if words.iter().any(|word| TAG_READS.is_match(word)) {
            return false;
        }

        words
            .iter()
            .any(|word| TAG_WRITES.is_match(word) || !word.starts_with('-'))
    })
}

fn settings_change(command: &str) -> bool {
    CONFIG.find_iter(command).any(|found| {
        let rest = &command[found.end()..];
        let segment = rest.split(['|', ';', '&', '\n']).next().unwrap_or("");

        !SHOW.is_match(segment)
    })
}

// the command without the bodies of heredocs that cat or tee only write
// out: a file that mentions git commit is not a commit. A heredoc fed to
// anything else (a shell, python, node) may run what it holds, so it stays.
fn spoken(command: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut owed: Vec<(&str, bool)> = Vec::new();

    for line in command.split('\n') {
        if let Some(&(end, drop)) = owed.first() {
            let done = line.trim() == end;
            if !drop || done {
                kept.push(line);
            }
            if done {
                owed.remove(0);
            }
            continue;
        }

        kept.push(line);
        for found in HEREDOC.captures_iter(line) {
            let at = found.get(0).map_or(0, |all| all.start());
            if line[..at].ends_with('<') {
                continue;
            }
            let end = (1..=3)
                .find_map(|group| found.get(group))
                .map_or("", |word| word.as_str());
            let feeds = SEPARATORS
                .split(&line[..at])
                .last()
                .and_then(|piece| piece.split_whitespace().next())
                .unwrap_or("");
            owed.push((end, WRITERS.contains(&feeds)));
        }
    }

    kept.join("\n")
}

pub fn classify(command: &str) -> Vec<&'static str> {
    let command = spoken(command);
    let command = command.as_str();
    let acts: [(&str, bool); 6] = [
        ("commit", COMMIT.is_match(command)),
        ("push", PUSH.is_match(command)),
        ("push", tags_write(command)),
        ("push", PUBLISH.is_match(command)),
        ("push", GH.is_match(command)),
        ("settings", settings_change(command)),
    ];
    let mut found: Vec<&'static str> = Vec::new();

    for (act, hit) in acts {
        if hit && !found.contains(&act) {
            found.push(act);
        }
    }

    found
}

/// Does the command stage things itself, so that the index is not yet the commit.
pub fn stages(command: &str) -> bool {
    ADDS.is_match(command)
}

fn unquote(word: &str) -> &str {
    let quoted = word.len() >= 2
        && ((word.starts_with('"') && word.ends_with('"'))
            || (word.starts_with('\'') && word.ends_with('\'')));

    if quoted {
        &word[1..word.len() - 1]
    } else {
        word
    }
}

/// The folder a git command acts on: its `-C`, after any `cd` before it.
/// One with a variable in it cannot be known, so it counts as here.
pub fn target_of(command: &str, cwd: &Path) -> PathBuf {
    let before = GIT_AT.find(command).map_or("", |at| &command[..at.start()]);
    let mut steps: Vec<&str> = SEPARATORS
        .split(before)
        .filter_map(|piece| CD.captures(piece.trim()))
        .map(|found| unquote(found.get(1).map_or("", |dir| dir.as_str())))
        .collect();

    if let Some(flag) = TARGET.captures(command) {
        steps.push(unquote(flag.get(1).map_or("", |dir| dir.as_str())));
    }
    if steps.iter().any(|dir| dir.contains(['$', '%', '`'])) {
        return cwd.to_path_buf();
    }

    steps
        .iter()
        .fold(cwd.to_path_buf(), |at, dir| resolve(&at, &native(dir)))
}

pub fn secret_in(line: &str) -> Option<&'static str> {
    if line.contains(WAIVER) {
        return None;
    }

    SECRETS
        .iter()
        .find(|(_, pattern)| pattern.is_match(line))
        .map(|(kind, _)| *kind)
}

pub fn secret_file(file: &str) -> bool {
    SECRET_FILES.is_match(file) && !SAFE_FILES.is_match(file)
}

/// Added lines of a unified diff, as `(file, kind)` for each that looks secret.
pub fn scan_diff(diff: &str) -> Vec<(String, &'static str)> {
    let mut found = Vec::new();
    let mut file = "";

    for line in diff.split('\n') {
        if let Some(name) = line.strip_prefix("+++ ") {
            file = name.strip_prefix("b/").unwrap_or(name);
        }
        if !line.starts_with('+') || line.starts_with("+++") {
            continue;
        }
        if let Some(kind) = secret_in(line) {
            found.push((file.to_string(), kind));
        }
    }

    found
}

pub fn attribution_problem(command: &str, policy: &str) -> Option<String> {
    let has = TRAILER.is_match(command);

    match (policy, has) {
        ("forbidden", true) => Some("a Co-Authored-By line, which this repo forbids".to_string()),
        ("required", false) => Some("no Co-Authored-By line, which this repo requires".to_string()),
        _ => None,
    }
}

pub struct Identity<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

pub fn identity_problem(command: &str, actual: &Identity, wanted: &Identity) -> Option<String> {
    let want: Vec<&str> = [wanted.name, wanted.email]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    if want.is_empty() {
        return None;
    }

    let overrides = OVERRIDES.is_match(command);
    if overrides {
        return (!want.iter().all(|part| command.contains(part)))
            .then(|| format!("an author override that is not {}", want.join(" ")));
    }

    let off: Vec<String> = [
        ("user.name", actual.name, wanted.name),
        ("user.email", actual.email, wanted.email),
    ]
    .into_iter()
    .filter(|(_, have, want)| !want.is_empty() && have != want)
    .map(|(setting, have, want)| format!("git {setting} is \"{have}\", not \"{want}\""))
    .collect();

    (!off.is_empty()).then(|| off.join(" and "))
}

/// Files that could change what a user sees, with no changelog beside them.
pub fn changelog_owed(files: &[String], notes: &str, quiet: &[String]) -> bool {
    if files.iter().any(|file| file == CHANGELOG) {
        return false;
    }

    let mut hushed: Vec<&str> = quiet.iter().map(String::as_str).collect();
    if !notes.is_empty() {
        hushed.push(notes);
    }
    let under = |file: &str, prefix: &str| {
        let folder = if prefix.ends_with('/') {
            prefix.to_string()
        } else {
            format!("{prefix}/")
        };
        file == prefix || file.starts_with(&folder)
    };

    files
        .iter()
        .any(|file| !QUIET.is_match(file) && !hushed.iter().any(|prefix| under(file, prefix)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // assembled here so that this file never holds a line that looks secret
    fn fake(kind: &str) -> String {
        match kind {
            "aws" => ["AKIA", "ABCDEFGHIJKLMNOP"].concat(),
            "github" => ["ghp_", &"a".repeat(36)].concat(),
            "key" => ["-----BEGIN RSA", " PRIVATE KEY-----"].concat(),
            _ => ["api_key = \"", &"q".repeat(20), "\""].concat(),
        }
    }

    #[test]
    fn classify_spots_commits_and_anything_that_leaves_the_machine() {
        let cases: &[(&str, &[&str])] = &[
            ("git commit -m \"x\"", &["commit"]),
            ("git -C sub commit --amend", &["commit"]),
            (
                "git add -A && git commit -m x && git push",
                &["commit", "push"],
            ),
            ("git push origin main", &["push"]),
            ("git tag 1.0.0", &["push"]),
            ("git tag -a 1.0.0 -m \"One\"", &["push"]),
            ("git tag -d 1.0.0", &["push"]),
            ("git tag", &[]),
            ("git tag -l", &[]),
            ("git tag --list \"1.*\"", &[]),
            ("git tag --points-at HEAD", &[]),
            ("git tag --contains abc123", &[]),
            ("git tag -v 1.0.0", &[]),
            ("npm publish", &["push"]),
            ("cargo publish --dry-run", &["push"]),
            ("gh release create 1.0.0", &["push"]),
            ("gh release upload 1.0.0 a.zip", &["push"]),
            ("gh release view 1.0.0", &[]),
            ("gh release download 1.0.0", &[]),
            ("gh release list", &[]),
            ("gh pr create --fill", &["push"]),
            ("heeded config --local", &["settings"]),
            ("node \"C:/x/bin/heeded.js\" config", &["settings"]),
            ("heeded config --show", &[]),
            ("heeded help config", &[]),
            ("git status", &[]),
            ("git log --grep commit", &[]),
            ("echo commit", &[]),
            ("npm run check", &[]),
            ("cat > a.mjs <<'EOF'\ngit commit -m x\nEOF", &[]),
            ("cat <<EOF > a.sh\ngit push\nEOF\nnode a.mjs", &[]),
            ("tee a.txt <<-\"END\"\n\tgit commit\n\tEND", &[]),
            ("cat > a.txt <<EOF\ngit commit\nEOF\ngit push", &["push"]),
            ("bash <<'EOF'\ngit commit -m x\nEOF", &["commit"]),
            ("python - <<'EOF'\nrun('git push')\nEOF", &["push"]),
            ("git commit -F - <<'EOF'\nSubject\nEOF", &["commit"]),
            ("cat <<< \"git commit\"", &["commit"]),
            ("", &[]),
        ];

        for (command, acts) in cases {
            assert_eq!(classify(command), *acts, "{command}");
        }
        assert!(stages("git add -A && git commit -m x"));
        assert!(stages("git commit -am x"));
        assert!(stages("git commit --all -m x"));
        assert!(!stages("git commit -m \"add a thing\""));
    }

    #[test]
    fn secrets_are_found_by_kind_and_never_in_ordinary_lines() {
        let waived = format!("+{} // heeded: not a secret", fake("assigned"));
        let cases = [
            (
                format!("+const k = \"{}\"", fake("aws")),
                Some("an AWS access key"),
            ),
            (
                format!("+token: {}", fake("github")),
                Some("a GitHub token"),
            ),
            (format!("+{}", fake("key")), Some("a private key")),
            (
                format!("+{}", fake("assigned")),
                Some("a hard-coded secret"),
            ),
            (waived, None),
            ("+const token = process.env.TOKEN".to_string(), None),
            ("+password = \"hunter2\"".to_string(), None),
            (
                "+const sha = \"0123456789abcdef0123456789abcdef01234567\"".to_string(),
                None,
            ),
        ];
        let diff = [
            "diff --git a/lib/a.js b/lib/a.js",
            "--- a/lib/a.js",
            "+++ b/lib/a.js",
            "@@ -0,0 +1 @@",
            &format!("+const k = \"{}\"", fake("aws")),
            &format!("-const old = \"{}\"", fake("github")),
            "+++ b/lib/b.js",
            "+const fine = 1",
        ]
        .join("\n");

        for (line, kind) in &cases {
            assert_eq!(secret_in(line), *kind, "{}", &line[..line.len().min(20)]);
        }
        assert_eq!(
            scan_diff(&diff),
            vec![("lib/a.js".to_string(), "an AWS access key")]
        );
        let files = [
            ".env",
            "a/.env.local",
            "id_rsa",
            "x.pem",
            ".env.example",
            "env.js",
            "key.js",
        ];
        let found: Vec<bool> = files.iter().map(|file| secret_file(file)).collect();
        assert_eq!(found, [true, true, true, true, false, false, false]);
    }

    #[test]
    fn identity_attribution_and_changelog_checks() {
        let me = Identity {
            name: "A C",
            email: "a@c.dev",
        };
        let other = Identity {
            name: "A C",
            email: "work@corp.com",
        };
        let any = Identity {
            name: "",
            email: "",
        };
        let identity = [
            ("git commit -m x", &me, &me, false),
            ("git commit -m x", &other, &me, true),
            ("git commit -m x", &other, &any, false),
            ("git commit --author=\"X <x@y.z>\" -m x", &me, &me, true),
            (
                "git -c user.email=a@c.dev -c user.name=\"A C\" commit -m x",
                &other,
                &me,
                false,
            ),
        ];
        let trailer = "git commit -m \"x\n\nCo-Authored-By: M <m@x.y>\"";
        let attribution = [
            (trailer, "any", false),
            (trailer, "forbidden", true),
            (trailer, "required", false),
            ("git commit -m x", "required", true),
            ("git commit -m x", "forbidden", false),
        ];
        let owned = |files: &[&str]| {
            files
                .iter()
                .map(|file| file.to_string())
                .collect::<Vec<_>>()
        };
        let owed = [
            (owned(&["lib/a.js"]), ".heeded", owned(&[]), true),
            (
                owned(&["lib/a.js", "CHANGELOG.md"]),
                ".heeded",
                owned(&[]),
                false,
            ),
            (
                owned(&["test.js", "docs/x.md", "README.md", ".gitignore"]),
                ".heeded",
                owned(&[]),
                false,
            ),
            (owned(&["notes/plan.txt"]), "notes", owned(&[]), false),
            (owned(&["notes/plan.txt"]), "", owned(&[]), true),
            (owned(&[]), ".heeded", owned(&[]), false),
            (
                owned(&["web/apps/web/e2e/a.spec.ts"]),
                "",
                owned(&["web/apps/web/e2e/"]),
                false,
            ),
            (owned(&["xtask/src/main.rs"]), "", owned(&["xtask"]), false),
            (owned(&["xtaskish/main.rs"]), "", owned(&["xtask"]), true),
            (owned(&["Cargo.lock"]), "", owned(&["Cargo.lock"]), false),
            (
                owned(&["Cargo.lock", "src/a.rs"]),
                "",
                owned(&["Cargo.lock"]),
                true,
            ),
        ];

        for (command, actual, wanted, bad) in identity {
            assert_eq!(
                identity_problem(command, actual, wanted).is_some(),
                bad,
                "{command}"
            );
        }
        for (command, policy, bad) in attribution {
            assert_eq!(
                attribution_problem(command, policy).is_some(),
                bad,
                "{policy}"
            );
        }
        for (files, notes, quiet, is) in owed {
            assert_eq!(changelog_owed(&files, notes, &quiet), is, "{files:?}");
        }
    }

    #[test]
    fn target_of_follows_cd_and_git_c() {
        let cwd = if cfg!(windows) {
            Path::new("C:\\outer")
        } else {
            Path::new("/outer")
        };
        let abs = if cfg!(windows) { "C:\\abs" } else { "/abs" };
        let cases = [
            ("git commit -m x".to_string(), cwd.to_path_buf()),
            ("git -C .agent commit -m x".to_string(), cwd.join(".agent")),
            ("git -C \"my notes\" push".to_string(), cwd.join("my notes")),
            (format!("git -C {abs} commit"), PathBuf::from(abs)),
            ("git -c user.name=x commit".to_string(), cwd.to_path_buf()),
            ("cd sub && git commit -m x".to_string(), cwd.join("sub")),
            ("cd a; cd ../b && git commit".to_string(), cwd.join("b")),
            (
                "cd \"$S/plain\" && git commit".to_string(),
                cwd.to_path_buf(),
            ),
            ("git commit && cd elsewhere".to_string(), cwd.to_path_buf()),
            (
                format!("cd {abs} && git -C inner push"),
                Path::new(abs).join("inner"),
            ),
        ];

        for (command, expected) in cases {
            assert_eq!(target_of(&command, cwd), expected, "{command}");
        }
    }
}
