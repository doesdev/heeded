# heeded

> Good habits, installed rather than remembered

A Claude Code plugin, with a draft for Cursor, that enforces a
small workflow contract in any repo that opts in. One Rust binary
is both the CLI and the hook. So far: `heeded init` stamps a repo with a
notes folder, a hook logs every move that changes the repo to a
journal there, and a session cannot stop while any of those
moves is newer than the handoff.

The plan is in [docs/plan.md](docs/plan.md) and the reasons in
[docs/decisions](docs/decisions). The state of play is a handoff
kept in private notes, so it is not in the repo.

## Install

```sh
cargo install --path /path/to/heeded
claude plugin marketplace add /path/to/heeded
claude plugin install heeded@heeded
```

The first line puts `heeded` on the `PATH`. Without a Rust
toolchain, download the binary for your machine from the
[releases](https://github.com/doesdev/heeded/releases) instead,
check it against `SHA256SUMS`, rename it `heeded` (`heeded.exe`
on Windows) and put it on the `PATH`. Builds are for Windows x64,
Linux x64 (static, any distro) and macOS on Apple silicon and
Intel. They are not signed: on Windows, `Unblock-File` clears the
downloaded-file mark, and on macOS, `xattr -d
com.apple.quarantine heeded` does. `heeded --version` says which
one you have. The plugin's hooks and
skills call it by name, so both must be installed, and a new build
of the binary takes effect in the next tool call.

To update, run `heeded update`. When a newer release is out it
downloads the binary for your machine, checks it against
`SHA256SUMS` and that it runs, swaps it in, then runs `claude plugin
marketplace update heeded` and `claude plugin update heeded@heeded`.
`heeded update --check` only says whether there is one. It needs
`curl`. A marketplace added from a folder updates from that folder,
so bring that checkout up to date as well.

## Usage

```sh
heeded init [dir]                   # stamp a repo; never overwrites
heeded init --notes private/notes   # with the notes somewhere else
heeded init --tracked               # with the notes committed to the repo
heeded moves [dir]                  # what changed since the last handoff
heeded retro --task ... --outcome ... --class ...   # log how a task went
heeded config                       # step through your settings
heeded help [command]
```

In a session the same things are `/heeded:init`,
`/heeded:handoff` and `/heeded:retro`.

## What a stamped repo has

A repo opts in with a `.heeded.json` at its root, which says where
its working notes live:

```json
{ "notes": ".heeded" }
```

`heeded init` writes that, the folder it names, and a
`package.json` with a failing `check` script if the repo has no
`package.json`. It touches nothing else.

```
.heeded.json
.heeded/
  handoff.md      what the last session did and left open
  plan.md         current intent
  decisions/      one record per decision
  rework.jsonl    one line per finished task
  journal.jsonl   one line per move, written by a hook; ignored
  .gitignore      ignores the journal
```

The notes are private by default: `heeded init` hides the marker
and the folder from git in that clone (`.git/info/exclude`), so
they are not committed. `--tracked` leaves them visible to git.

The folder can be anywhere: a private checkout nested in a public
repo, an absolute path. The plugin never writes to `docs/` unless
the marker says so.

To track the plan and decisions but not the handoff, give the
handoff and journal a folder of their own:

```json
{ "notes": "docs", "privateNotes": ".heeded" }
```

`heeded init --notes docs --private .heeded` writes that. A new
private folder holds a `.gitignore` of `*`, so git never sees it in
any clone, and a commit carrying anything from it is refused. (This
repo is set up this way.)

## Settings

Your defaults go in `~/.heeded/config.json`. A repo's
`.heeded.json` can override any of them. `heeded config` writes
the file if it is missing, with your name and email from git, and
shows what is in effect in the repo you are in.

```json
{
  "permission": { "commit": "ask", "push": "ask" },
  "identity": { "name": "", "email": "", "attribution": "any" },
  "checks": { "changelog": false, "quiet": [] },
  "directives": {}
}
```

Those are the defaults. `ask` means you are asked before a
session commits, or before anything leaves the machine (push,
tag, publish, release); `allow` means you are not. With an
identity set, a commit under any other git author is refused.
`attribution` is `any`, `required` or `forbidden`, and is about
`Co-Authored-By` lines. Commits are also scanned for secrets.
With `checks.changelog` on, a commit that touches no
`CHANGELOG.md` is refused once with a reminder, and goes through
when the same command is run again. `checks.quiet` lists paths that
never need a changelog line, such as test folders.

`directives` are your own standing directions to a session, shown
to it at the start in every stamped repo. Name them as you like.
Each is a line of text or a file beside the config that holds it:

```json
{
  "directives": {
    "voice": "Write code the way the andrew-carpenter-voice skill describes.",
    "style": { "file": "style.md" }
  }
}
```

A repo's `.heeded.json` can add its own, replace one of yours by
name, or switch one off with `false`. A file is cut at 4000
characters, and a repo can only name files inside itself.

heeded comes with no rules of its own. What a session is told
at the start is where the notes are, and your directives.

A repo is gated only when both the marker and the handoff it
points at exist. Everything else is left alone.

## Develop

```sh
npm run check
```

CI runs the same check on Windows, Linux and macOS for every push.
Pushing a tag `vX.Y.Z` that matches `Cargo.toml` builds the release
binaries and publishes them with the notes from that version's
changelog section.

That runs `cargo fmt --check`, `cargo clippy` and `cargo test`; npm
is only the name the check goes by (decision 0003). The Node
version this replaced is in `archive/node-0.11/`.
