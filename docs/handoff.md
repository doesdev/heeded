# Handoff

Last session: 2026-10-15T05:46Z (Claude Code, claude-opus-5-5,
session 3, in the same long conversation as session 2)

## Did

Sessions 1 and 2, in short: the gate, journal, notes folder and
marker (0001 to 0012); Andrew's rules, then moved into his own
config as directives (0013, 0020, 0021); the rename to heeded
(0016); subagents and worktrees (0017, 0018); the commit and
push guard and its settings (0015, 0019, 0022); `heeded retro`;
a drafted Cursor adapter (0014). The records hold the detail.

Session 3:

- `heeded help` and an interactive `heeded config` (global,
  local with inherit, show). 0.10.0.
- 0023, from the gazelle-audio re-assessment: `git tag` and
  `gh release` ask only when they change something; a git
  command's repo is its `-C` or last `cd`; `heeded config`
  from a shell asks the owner; `checks.quiet`; init hides its
  placeholder `package.json`; the hook leaves early in repos
  that never opted in. `~/.heeded/voice.md` now defers to a
  codebase's established style. 0.11.0.
- 0024, on Andrew's yes: heeded is one Rust binary. `src/`
  holds the crate (paths, clock, git, notes, snapshot, session,
  journal, store, settings, commit, guard, gate, harness,
  scaffold, rework, wizard, cli). `heeded hook` answers both
  harnesses; both manifests and the three skills call `heeded`
  by name. Dependencies: serde, serde_json (preserve_order),
  regex.
- The Node suite (75 tests) was pointed at the binary with
  `HEEDED_BIN` and passed before any Node code was removed.
  Then the tests were ported: `cargo test` runs 81 (unit tests
  in `src/`, and `tests/hook.rs`, `tests/cli.rs`,
  `tests/library.rs`, `tests/repo.rs`). `npm run check` now
  runs `cargo fmt --check`, `cargo clippy -D warnings` and
  `cargo test`.
- After the port, cut git calls: the tree fingerprint reads
  HEAD from `git status --porcelain=v2 --branch`; no side
  check when the event's folder is the project's; the guard
  finds the marker by walking up, not with git.
- The Node code is in `archive/node-0.11/`. `node_modules` is
  gone. `hooks/hook.js` is now a shim that hands events to
  `heeded hook`, for sessions started before 0.12.0 (this one).
- Installed: `cargo install --path .` put `heeded.exe` in
  `~/.cargo/bin`. `npm uninstall -g heeded` removed the Node
  package, and its three stale shims in fnm's Node 24 folder
  were deleted by hand. `heeded` resolves to the binary in Git
  Bash and PowerShell. Plugin updated to 0.12.0.
- 0002 and 0005 marked superseded. README, changelog, plan
  updated.
- 0025, from a git-latency task Andrew passed on: measured
  every git call heeded makes in four repos (this one,
  gazelle-audio, wcmd-main, a synthetic 30,000-file repo). The
  cost is starting git (19 to 23 ms a call), not its work. A
  throwaway gix 0.89 prototype in scratch matched git's
  results but was about twice as slow with real config
  loaded, and fast only with system and global config
  ignored. Not ported. Scripts and prototype are in this
  session's scratch folder, not the repo.
- Every git call now sets `GIT_OPTIONAL_LOCKS=0`
  (`src/git.rs`), so heeded's status never takes
  `index.lock` from under the agent's own git command.
- A real headless Haiku session on the binary, in a freshly
  stamped scratch repo: the stop hook (`[heeded hook]`)
  refused after an edit, the session ran the handoff skill,
  which ran `heeded moves`, and it wrote a dated handoff.
- Removed a `timing-state` folder an earlier benchmark had
  left in `repos/`.

Proof, median of 15 per hook call on this machine: 11 to 12 ms
in a repo that never opted in (Node was 46); 11 ms before an
ordinary command in a stamped repo; 36 ms before a push; 68 ms
after a tool call in a stamped repo (Node was 171). This
session's own journal kept growing after the switch, through
the shim.

## Did not

- Commit anything. The two archive renames from session 1 are
  staged; everything else is untracked.
- Run anything in Cursor, or build the binary for the work
  machine.
- Probe whether a background subagent can raise an `ask`.
- Touch gazelle-audio.
- Build the pre-edit gate, the session budget, or any CI.

## Open

- The plugin calls whatever `heeded` is on the `PATH`. A
  stale binary or a version mismatch with the plugin is not
  detected. After changing `src/`, run `cargo install --path .`
  or this repo's own sessions run the old binary.
- `hooks/hook.js` (the shim) needs deleting once no session
  started before 0.12.0 is left. It needs Node while it exists.
- Session state written by the Node hook used sha256 tree
  fingerprints; the binary uses FNV-1a. A session spanning the
  switch saw one spurious change, which is why this session's
  journal shows moves the binary logged right after the swap.
- gix was ruled out on numbers from one Windows machine with
  Git for Windows' `core.fscache` on. On Linux or a much larger
  repo the result could differ; 0025 says when to look again.
- His `changelog` directive and `checks.changelog` are two
  switches for one rule.
- His rules and voice live in two unversioned files on this
  machine; the work machine needs copies, and a binary.
- The secret patterns are a first guess. One known false
  positive in gazelle (`__GAZELLE_TOKEN__`).
- Session and guard state files pile up; nothing prunes them.
- The Edit and Write tools turn a unicode escape for a dash in
  JavaScript into the character, and heredocs passed to
  `node -` lose backslashes. Rust's `'\u{2013}'` survived.

## Next

`docs/plan.md`, "Next session": Andrew reviews the overnight
list, then the first commit if he says so, which will be the
guard's first real use under the binary.
