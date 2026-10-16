# Handoff

Last session: 2026-10-16T06:12Z (Claude Code, claude-opus-5-5,
session 5)

## Did

Sessions 1 to 4, in short: the gate, journal, notes folder and
marker (0001 to 0012); Andrew's rules as directives in his own
config (0013, 0020, 0021); the rename (0016); subagents and
worktrees (0017, 0018); the commit and push guard (0015, 0019,
0022, 0023); `heeded retro`, `help`, `config`; a drafted Cursor
adapter (0014); one Rust binary (0024); git stays a child
process (0025); worktrees and the notes checkout guarded as
their repo (0026, 0027); CI on three systems and release
binaries on a tag (0028); subagents refused where the session
would be asked (0029); heredocs written to files not classified
(0030); v0.13.0 tagged and released. All of it is committed and
pushed (last: a9ee608). The records and git log hold the detail.

Session 5, on Andrew's ask: judged whether the repo (now
PRIVATE on GitHub, doesdev/heeded) can go public, history
included.

- Read every blob in all 14 commits (228 objects, 22 KiB
  packed). No real secret: the only key-shaped strings are the
  secret scan's own patterns and test fakes built by joining
  pieces (`["AKIA", ...].concat()`). Emails in content are test
  fakes, plus `i@izs.me` from a lockfile in `archive/node-0.11`.
  Workflows are fit for public: actions pinned by SHA, `check`
  has `contents: read` on `pull_request`, release runs only on a
  `v*` tag. No `pull_request_target`.
- Wrote `LICENSE` (MIT, 2026, Andrew Carpenter), to match
  `license = "MIT"` in `Cargo.toml`; there was no license file.
  Changelog line under `## [Unreleased]`, Added.
- Proof: `npm run check` passed (fmt, clippy, 86 tests).
- Also rewrote this file, which carried uncommitted fixes to the
  session 4 text.

## Did not

- Commit. `LICENSE`, `CHANGELOG.md` and this file are
  uncommitted; Andrew asked to be asked first.
- Make the repo public. That is Andrew's to do on GitHub.
- Decide on the findings below under Open; they are his.

## Open

- Before going public, Andrew's calls: the history and the
  tracked notes, reviewed this session.
- 0029 refuses foreground subagents too; only a background one
  was observed passing an `ask`. Drop 0029 if Claude Code starts
  raising background agents' asks.
- 0030: a script written and run in one command (`cat > a.sh
  <<EOF` then `bash a.sh`) is no longer seen to commit.
- heeded compares the root found by walking up from the event's
  folder with git's top. A harness sending an aliased folder
  (symlink, 8.3 name) could make them differ. Not seen.
- A push from a stamped session into an unrelated unstamped repo
  is not asked; only Andrew's directive text covers it (0027).
- A worktree outside the stamped repo whose events carry a cwd
  outside it is unguarded (0026).
- heeded does not run on the work machine yet, so the Cursor
  adapter stays untried. `docs/plan.md`, "On the work machine".
- Only the Windows release binary was run; the Linux and macOS
  ones were not.
- The after-hook costs 68 to 90 ms; not looked into further.
- The plugin calls whatever `heeded` is on `PATH`; a stale
  binary is not detected. After changing `src/`, run `cargo
  install --path .`.
- gazelle's secret scan false positive (`__GAZELLE_TOKEN__`)
  bit again on its notes; the waiver comment or rewording works.
- `repos/my-ai` still exists, for Andrew to delete; the `my-ai`
  git remote here still points at doesdev/my-ai.
- `hooks/hook.js`, the shim for sessions started before 0.12.0,
  needs deleting once none is left.
- The stop gate counts an owner's edit made while a turn is
  finishing as the session's work.
- gix ruled out on one Windows machine; 0025 says when to look
  again.
- His `changelog` directive and `checks.changelog` are two
  switches for one rule.
- His rules and voice live in two unversioned files on this
  machine.
- Session and guard state files pile up; nothing prunes them.

## Next

Commit `LICENSE`, the changelog line and this file on Andrew's
yes; then `docs/plan.md`, "Next session": the pre-edit gate,
which needs Andrew's answer on what a trigger path is.
