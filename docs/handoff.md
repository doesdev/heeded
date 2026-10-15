# Handoff

Last session: 2026-10-15T15:12Z (Claude Code, claude-opus-5-5,
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
  both fixed. Committed as 6d996fa and pushed, each on Andrew's
  yes.
- Then the evaluation's `.agent` question. Two options were put
  to Andrew (notes checkout counts as the stamped repo, or push
  asks from a stamped session wherever it lands); he chose the
  first. `src/guard.rs`: a repo with no marker, not a worktree
  of a stamped one, that holds the folder the marker's `notes`
  names and does not hold the stamped repo, is judged with the
  stamped repo's settings (scan, identity, attribution,
  permission), with the changelog reminder skipped. New
  `inside` helper: a path on another drive is never inside.
- `tests/hook.rs`:
  `the_guard_judges_the_notes_checkout_as_the_repo_it_belongs_to`
  (nested notes repo: commit and push ask; secret in notes
  denied; with `commit: allow` and `checks.changelog`, a
  non-doc notes commit passes; notes outside the repo: push
  asks, and the old nested repo is then left alone). It failed
  on the old guard, and with the changelog skip disabled (the
  first version did not, because `.md` files are quiet anyway;
  a `.rs` file was added).
- Decision 0027; 0026 marked amended by it. Changelog line and
  plan entry.
- Proof: `npm run check` passed (fmt, clippy, 83 tests).
  Committed as 2a0e263 and pushed, each on Andrew's yes. A brief
  for the gazelle session (steps 5 and 6, OK'd by Andrew) was
  given in chat for him to carry.
- Then release binaries, Andrew's proposal (0028), macOS on his
  ask. `heeded version` / `--version` (`src/cli.rs`, test
  `version_says_which_build_this_is`).
  `.github/workflows/check.yml`: `npm run check` on Windows,
  Linux, macOS per push and PR. `.github/workflows/release.yml`:
  on a `v*` tag, checks tag equals `Cargo.toml` and the changelog
  has the section, drafts a release with it as notes, builds
  Windows x64 (static CRT), Linux x64 musl, macOS arm64 and x64,
  uploads, adds `SHA256SUMS`, publishes. Only `actions/checkout`
  v7.0.1, pinned to 3d3c42e5. README install and develop
  sections, changelog lines, plan entry.
- Proof, locally only: tag check and changelog awk dry-run for
  v0.12.0 (match, section printed) and v9.9.9 (empty);
  static-CRT Windows release build runs and references no
  `vcruntime140.dll`; `npm run check` passed (84 tests);
  binary reinstalled, `heeded --version` prints `heeded 0.12.0`.
- First CI run (38757674317, on 38be4d0): Linux passed; Windows
  and macOS failed in `tests/cli.rs` (cargo stopped there, so the
  other suites have not run on either). Causes: (1) the tests
  build paths from the temp folder, which on macOS is behind a
  symlink (`/var` is `/private/var`) and on the Windows runner
  has a short name (`RUNNER~1`), while heeded prints git's real
  path; (2) the Windows runner checks out with CRLF, so the
  compiled-in templates were CRLF. Fixes, uncommitted: the lab
  canonicalizes its temp folder (`real` in
  `tests/common/mod.rs`); `.gitattributes` with
  `* text=auto eol=lf`. Reproduced (1) locally with an 8.3 temp
  path: old harness 3 failures, new harness all suites pass.
- Found while fixing: this session's Python edits wrote CRLF into
  CHANGELOG.md, README.md, docs/handoff.md, docs/plan.md,
  src/cli.rs and src/guard.rs, and some of it was committed.
  Converted back and renormalized; `git diff --cached
  --ignore-cr-at-eol` shows only this file's real edits. Python
  on Windows needs `newline=''` when writing.

## Did not

- Commit the CI fixes below. 38be4d0 (Build release binaries in
  CI) was committed and pushed on Andrew's yes. No version bump.
- See a green CI run. The release workflow has never run.
- Cut a release. Needs a version bump commit, then a tag push,
  each on Andrew's yes.
- Touch gazelle-audio, or run the evaluation's step 5 (marker,
  `heeded init --notes .agent`, moving its memory into a
  handoff) or step 6 (a live subagent turn). Both wait for
  Andrew's OK.
- Look at the after-hook's cost in gazelle (137 ms p50 against
  68 here, a bigger tree).

## Open

- A push from a stamped session into some other unstamped repo
  (not its notes, not a worktree) is still not asked; Andrew's
  directive text is all that covers it (0027 chose not to).
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
- heeded compares the project root it finds by walking up from
  the event's folder with the top git reports. If a harness ever
  sends an aliased folder (a symlink, an 8.3 name), the two would
  differ and the guard could misjudge `repo == root`. Not seen in
  practice; Node's cwd is already the real path.
- The release binaries are unsigned; a managed work machine may
  refuse them. Check there before relying on it.
- The plugin calls whatever `heeded` is on `PATH`; a stale
  binary is not detected (`--version` now makes it possible). After changing `src/`, run
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

Ask Andrew to commit and push the CI fixes, then read the run on
all three systems; fix what the suites that have not yet run on
Windows and macOS turn up. Gazelle's adoption is with the gazelle
session.
