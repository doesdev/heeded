# Handoff

Last session: 2026-10-15T15:18Z (Claude Code, claude-opus-5-5,
session 4)

## Did

Sessions 1 to 3, in short: the gate, journal, notes folder and
marker (0001 to 0012); Andrew's rules as directives in his own
config (0013, 0020, 0021); the rename (0016); subagents and
worktrees (0017, 0018); the commit and push guard (0015, 0019,
0022, 0023); `heeded retro`, `help`, `config`; a drafted Cursor
adapter (0014); one Rust binary (0024, 0.12.0); git stays a
child process (0025); the repo moved to `repos/heeded`. The
records and git log hold the detail.

Session 4, from two reports by the gazelle-audio session (both
pasted by Andrew, not in the repo) and Andrew's own asks. Each
commit and push below was on Andrew's yes.

- 6d996fa, pushed (0026): a linked worktree of a stamped repo is
  guarded with the main checkout's settings; a hidden marker
  never reaches a worktree, so subagents' commits there skipped
  every check. `root_and_common` in `src/git.rs`.
- 2a0e263, pushed (0027, Andrew chose this of two options): a
  repo holding the stamped repo's notes (gazelle's `.agent`) is
  guarded with its settings, minus the changelog reminder.
- 38be4d0, pushed (0028, Andrew's proposal, macOS on his ask):
  `.github/workflows/check.yml` runs `npm run check` on Windows,
  Linux, macOS; `release.yml` on a `v*` tag checks the tag equals
  `Cargo.toml` and the changelog has its section, drafts a
  release, builds Windows x64 (static CRT, checked: no
  `vcruntime140.dll`), Linux x64 musl, macOS arm64 and x64,
  uploads with `SHA256SUMS`, publishes. `heeded version` /
  `--version`. README install section.
- First CI run (38757674317): Linux passed, Windows and macOS
  failed in `tests/cli.rs` (cargo stopped there; the other suites
  have not run on those two yet). Temp paths: macOS `/var` is
  `/private/var`, the Windows runner has `RUNNER~1`, and heeded
  prints git's real path. Line endings: the Windows checkout made
  the compiled-in templates CRLF. 17076c5, committed, NOT pushed:
  the lab canonicalizes its temp folder (`real` in
  `tests/common/mod.rs`); `.gitattributes` `* text=auto eol=lf`.
  Reproduced locally with an 8.3 temp path: old harness 3
  failures, new all pass. It also undid CRLF that this session's
  Python edits had written into six files (Python on Windows
  needs `newline=''`).
- gazelle's second report: adopted there (notes `.agent`,
  `commit: allow`, `check` runs a private `.agent/check.mjs`,
  about 6 min); 0026 and 0027 confirmed live; the stop gate
  waited for a real background subagent and owed its moves after;
  after-hook 90 ms p50. Two findings, both now built,
  uncommitted, on Andrew's yes:
  - 0029: a background subagent's commit went through although
    heeded answered `ask`; the harness passed it. Now, with a
    non-empty `agent_id`, an `ask` becomes `deny` with "stop and
    report back". `check` wraps the old body, now `judge`, in
    `src/guard.rs`.
  - 0030: a heredoc written to a file with `cat >` that mentioned
    `git commit` asked Andrew. Now `spoken` in `src/commit.rs`
    drops heredoc bodies fed to `cat` or `tee` before
    classifying; a shell, python, node or git reading one keeps
    it; here-strings stay.
  - Tests: classify unit cases for heredocs;
    `a_subagent_is_refused_where_the_session_would_be_asked` and
    `a_heredoc_written_to_a_file_is_not_the_command` in
    `tests/hook.rs`; both failed on the old code. Changelog lines,
    plan entries.
- Proof: `npm run check` passed (fmt, clippy, 86 tests).
  `cargo install --path .` reinstalled; `heeded --version` says
  0.12.0.

## Did not

- Commit 0029, 0030 and this file: `src/guard.rs`,
  `src/commit.rs`, `tests/hook.rs`, `CHANGELOG.md`,
  `docs/plan.md`, the two records.
- Push 17076c5, so CI has not run with the fixes. No green run
  on Windows or macOS yet; the release workflow has never run.
- Bump the version or cut a release (bump is its own commit, the
  tag push needs Andrew's yes).
- Look into the after-hook cost further (90 ms in gazelle, 68
  here).

## Open

- 0029 refuses foreground subagents too; only a background one
  was observed passing an `ask`. Drop 0029 if Claude Code starts
  raising background agents' asks.
- 0030: a script written and run in one command (`cat > a.sh
  <<EOF` then `bash a.sh`) is no longer seen to commit.
- heeded compares the root found by walking up from the event's
  folder with git's top. A harness sending an aliased folder
  (symlink, 8.3 name) could make them differ. Not seen; Node's
  cwd is the real path.
- A push from a stamped session into an unrelated unstamped repo
  is not asked; only Andrew's directive text covers it (0027).
- A worktree outside the stamped repo whose events carry a cwd
  outside it is unguarded (0026); Claude Code's worktrees are
  inside the repo.
- Release binaries are unsigned; a managed work machine may
  refuse them.
- The plugin calls whatever `heeded` is on `PATH`; a stale
  binary is not detected (`--version` now makes it possible).
  After changing `src/`, run `cargo install --path .`.
- gazelle's secret scan false positive (`__GAZELLE_TOKEN__`)
  bit again on its notes; the waiver comment or rewording works.
- `repos/my-ai` still exists, for Andrew to delete; copy Claude
  Code's project folder for it once more after its last session.
- `hooks/hook.js`, the shim for sessions started before 0.12.0,
  needs deleting once none is left.
- The stop gate counts an owner's edit made while a turn is
  finishing as the session's work.
- gix ruled out on one Windows machine; 0025 says when to look
  again.
- His `changelog` directive and `checks.changelog` are two
  switches for one rule.
- His rules and voice live in two unversioned files on this
  machine; the work machine needs copies and a binary.
- Session and guard state files pile up; nothing prunes them.

## Next

Ask Andrew to commit 0029 and 0030, then push both commits and
read CI on all three systems; fix what the suites that have not
yet run on Windows and macOS turn up.
