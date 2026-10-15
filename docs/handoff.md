# Handoff

Last session: 2026-10-15T06:06Z (Claude Code, claude-opus-5-5,
session 4)

## Did

Sessions 1 to 3, in short: the gate, journal, notes folder and
marker (0001 to 0012); Andrew's rules, then directives in his
own config (0013, 0020, 0021); the rename (0016); subagents and
worktrees (0017, 0018); the commit and push guard (0015, 0019,
0022, 0023); `heeded retro`, `heeded help`, `heeded config`; a
drafted Cursor adapter (0014); the port to one Rust binary
(0024, 0.12.0); git stays a child process with
`GIT_OPTIONAL_LOCKS=0` (0025); the repo moved to
`repos/heeded` and 86e9be0 pushed. The records and git log hold
the detail.

Session 4, from the gazelle-audio session's evaluation of
0.12.0 (pasted by Andrew; not in the repo):

- Fixed the one gap it found: with the marker hidden (the
  `heeded init` default), a commit or push in a linked worktree
  passed the guard unchecked, no secret scan, identity,
  attribution, changelog or permission. The guard judged a
  command by the top of the repo it resolves to, and an
  untracked marker never reaches a worktree.
- `src/git.rs`: new `root_and_common`, the top and the shared
  git folder from one `rev-parse`. `src/guard.rs`: when the
  resolved repo has no marker, the main checkout (parent of the
  shared git folder) is checked; if stamped, the command is
  gated with its settings and notes. No extra git start.
- `tests/hook.rs`:
  `the_guard_judges_a_worktree_of_a_hidden_marker_repo_as_that_repo`
  (hidden marker, worktree under `.claude/worktrees/`; secret
  denied from inside, via `-C` and via `cd`; clean commit asks,
  then is allowed with `commit: allow`; push asks). Run against
  the old guard it failed (stashed `src/`, then restored).
- Decision 0026, listed in `docs/plan.md`. Changelog line under
  Unreleased, Fixed.
- Proof: `npm run check` passed (fmt, clippy, 82 tests). The
  first runs failed on fmt and on 0026's `Proposed by` format,
  both fixed. `cargo install --path .` put the new binary in
  `~/.cargo/bin`.

## Did not

- Commit. Uncommitted: `src/git.rs`, `src/guard.rs`,
  `tests/hook.rs`, `CHANGELOG.md`, `docs/plan.md`, 0026, this
  file. No version bump (its own commit, when wanted).
- Push. Nothing left the machine this session.
- Touch gazelle-audio, or run the evaluation's step 5 (marker,
  `heeded init --notes .agent`, moving its memory into a
  handoff) or step 6 (a live subagent turn). Both wait for
  Andrew's OK.
- Look at the after-hook's cost in gazelle (137 ms p50 against
  68 here, a bigger tree).

## Open

- Andrew's call, from the evaluation: should a repo nested in a
  stamped one (gazelle's private `.agent` notes checkout) get
  `push: ask`? Today `git -C .agent push` is not asked, which
  fits "an unstamped repo is its own business" but not his
  "ask before anything leaves the machine". Options: notes repo
  counts as the stamped repo, or push asks wherever a stamped
  session runs.
- Unknown: can a background subagent raise an `ask` prompt?
  The evaluation's step 6, with `commit: ask` for one turn.
- A worktree outside the stamped repo whose events carry a cwd
  outside it is still unguarded: the guard finds the project by
  walking up without git. Claude Code's worktrees live under
  `.claude/worktrees/`, so not hit today (0026).
- gazelle's `check` is undecided: about 15 min full, 4 without
  the browser suite; root has no `.node-version`; must set
  `GAZELLE_NO_HARDWARE=1`.
- `repos/my-ai` still exists, for Andrew to delete once nothing
  has it open. Claude Code's project folder for it was copied
  to `C--Users-andrew-repos-heeded`; copy once more after the
  last session there ends.
- The plugin calls whatever `heeded` is on `PATH`; a stale
  binary is not detected. After changing `src/`, run
  `cargo install --path .`.
- `hooks/hook.js` (a shim for sessions started before 0.12.0)
  needs deleting once none is left. It needs Node meanwhile.
- The stop gate counts an owner's edit made while a turn is
  finishing as the session's work. Harmless, worth knowing.
- gix was ruled out on one Windows machine; 0025 says when to
  look again.
- His `changelog` directive and `checks.changelog` are two
  switches for one rule.
- His rules and voice live in two unversioned files on this
  machine; the work machine needs copies and a binary.
- Secret patterns are a first guess; one known false positive
  in gazelle (`__GAZELLE_TOKEN__`).
- Session and guard state files pile up; nothing prunes them.
- The Edit and Write tools turn a unicode escape for a dash
  into the character; Rust's `'\u{2013}'` survives.

## Next

Ask Andrew to commit the worktree fix, answer the `.agent`
push question, and OK gazelle's adoption (the evaluation's
steps 5 and 6). Then `docs/plan.md`, "Next session".
