# Plan: heeded as a portable workflow plugin

Last updated: 2026-10-15T05:45Z

## Why this repo, why now

The strategies that work at work (context assembled by the
environment, not the prompt; codebase as prompt; mechanical
oracles; fresh context plus durable notes; gate on decisions;
permission to stop; measure rework) only hold there because
infrastructure makes them the path of least resistance. In
personal repos each one is a per-session choice, and choices
decay. The fix is to stop treating them as practices and ship
them as a product that gets installed: a Claude Code plugin
plus a repo scaffold, both living here.

## What is here already

- `multi-chat-playbook-human.md` (627 lines) and
  `multi-chat-playbook-agentic.md` (955 lines). Role-based
  orchestration docs from May 2026. Never used.
- Verdict: archive, do not delete. Both now live in
  `archive/`. Nothing in them is enforced, which is the
  whole problem this plan addresses.
- The plugin: the journal and stop hooks, `heeded init` (CLI
  and skill) and the `/handoff` skill, with tests. See
  `docs/handoff.md`.
- Salvage list (conventions worth carrying forward as rules):
  - repo is the source of truth; chat output is scratch
  - verification-gated commits, never trust prose claims
  - owned-files scope on tasks, with explicit scope expansion
  - dated, per-decision records (one decision per file)
  - `date -u` for every timestamp, never a model's sense of
    the date
  - filename references always paired with a content
    descriptor, because renames break filenames

## Target shape

Three layers, ordered by how much enforcement they buy.

### Layer 1: scaffold (makes the defaults exist)

One command, `heeded init` (built), stamps any repo with an
identical interface, never overwriting what is there:

```
.heeded.json             the opt-in, and where the notes live
<notes>/                `.heeded` unless the marker says otherwise
  plan.md               current intent; rewritten, not appended
  handoff.md            what the last session did and left open
  decisions/            NNNN-kebab-title.md, one decision each
    README.md           the record format
  rework.jsonl          one line per finished task (see below)
  journal.jsonl         one line per move, written by a hook;
                        ignored by the `.gitignore` beside it
```

The notes folder is the repo's choice (0011): anywhere in the
repo, a private checkout nested in it, or outside it. The
plugin does not write to `docs/`, `CLAUDE.md` or the repo's
`.gitignore`. In this repo the marker names `docs`.

Hooks come from the installed plugin, not from the repo. A
repo is gated when it has the marker and the handoff the
marker points at. `heeded init` is one implementation
reachable two ways: a global CLI and a plugin skill.

Plus a single oracle entry point whose name never changes
across repos (`check`), backed by whatever the repo needs.
The contract is the interface, not the implementation. A repo
with no `package.json` is stamped with one whose `check`
fails until it is wired up.

### Layer 2: hooks (the session cannot drift)

Enforced by harness hooks, all answered by the one `heeded`
binary (0024), so they run the same on Windows and Linux with
no bash and no Node, at about 11 ms a call where nothing is owed.

- session start (built): say where the notes are and what to
  read first, and list moves earlier sessions left unhanded
- journal (built): after each tool call that changed the
  repo, append one line to the journal
- stop (built): refuse to end a turn while any journaled move
  is newer than the last write to the handoff
- pre-edit: when the edit touches a trigger path (dependency
  manifests, migrations, public interface dirs), require a
  decision record newer than the plan, or block with a
  message naming what to write
- session budget: after N journaled moves or approximate
  context size, inject a nudge to write handoff and restart
- `/retro` skill: appends one line to `rework.jsonl`:
  `{ts, task, outcome, failureClass, note}`; failure classes
  are a closed set (wrong-context, wrong-assumption,
  missing-test, style-mismatch, spec-gap, none)
- `/handoff` skill (built): rewrites the handoff in a fixed
  shape, from the moves `heeded moves` lists. The stop gate's
  refusal names the skill. The gate does not check the shape;
  only this repo's tests do.

### Layer 3: CI (holds when nobody is looking)

Diff-coupled checks, each a `heeded` subcommand, run on every
push regardless of how the code got there:

- dependency manifest changed and the decisions folder did
  not, in the same commit: fail
- migration added without a decision: fail
- new source file with no neighbouring test, where the repo
  convention is co-location: fail

These assume the notes are tracked in the repo CI runs on.
Where they are private (0011 allows it) CI cannot see them,
and the checks have to run somewhere that can.

## What this cannot enforce, on purpose

Judgment. A decision record can be vacuous and pass every
check. The mechanics are enforced; the judgment is inspected,
monthly, by reading `rework.jsonl`. That review stays manual.

## Decisions

One record each in `docs/decisions/`:

1. 0001: this repo is a Claude Code plugin; playbooks archived.
2. 0002: hook and CI scripts are Node, StandardJS (superseded
   by 0024).
3. 0003: the oracle is `check`, run as `npm run check`.
4. 0004: cross-project conventions live in the plugin;
   per-project state lives with the project (amended by 0011).
5. 0005: tests use `mvt` (superseded by 0024).
6. 0006: the stop hook only refuses when the session changed
   the repo, and gives up loudly after 3 refusals.
7. 0007: the human proposes, the model ratifies; every record
   says who did which, and `npm run check` enforces it.
8. 0008: a hook-written journal; the stop gate reads it, so
   work after a handoff makes the handoff stale again.
9. 0009: `heeded init` ships as a global CLI and in the plugin,
   from one implementation.
10. 0010: installed as a plugin from this repo's own
    marketplace; hooks come only from the plugin (amended by
    0011).
11. 0011: a marker file opts a repo in and names its notes
    folder; the plugin takes no folder for itself.
12. 0012: notes are private by default (hidden from git in
    that clone); `--tracked` is the opt-in.
13. 0013: six rules of Andrew's, printed at session start
    (amended by 0021).
14. 0014: a portable core with one thin adapter per harness,
    because this must also run under Cursor. Not built.
15. 0015: permission to commit and push, and commit identity,
    come from config: user default, repo override. Not built.
16. 0016: the project is named heeded.
17. 0017: a subagent's moves are the session's.
18. 0018: the stop gate waits while subagents are out.
19. 0019: a guard runs before every commit and push.
20. 0020: the owner's directives come from config; `heeded
    config` writes the user config.
21. 0021: the rules are Andrew's directives in his own config;
    heeded ships none.
22. 0022: the changelog reminder is a setting, off by
    default.
23. 0023: the guard fixed against gazelle's real commands;
    the hook leaves at once in a repo that never opted in.
24. 0024: heeded is one Rust binary, the CLI and the hook;
    supersedes 0002 and 0005.
25. 0025: git stays a child process; in-process gix measured
    slower with real config.
26. 0026: a linked worktree of a stamped repo is guarded as
    that repo.
27. 0027: so is a repo that holds a stamped repo's notes,
    bar the changelog reminder.
28. 0028: release binaries are built by GitHub Actions on a
    version tag; CI runs the check on three systems.
29. 0029: a subagent is refused where the session would be
    asked.
30. 0030: heredoc bodies that cat or tee write out are not
    classified.

## Constraints added on 2026-10-09

- It must work under Cursor as well as Claude Code (0014).
  Design against the weaker harness first.
- The handoff is for the model, not for Andrew. So it need
  not read well, only be complete, and it need not be
  versioned (0012).
- The project is named heeded (0016). The repo's folder on
  this machine is still `repos/my-ai`.
- Directives print in stamped repos only. Andrew confirmed it
  when they were still shipped rules.

## What the second repo showed

The first try at adopting this in gazelle-audio (public, Rust
plus pnpm plus Android, `docs/` is a user manual, notes in a
private nested checkout, a 15 minute oracle, one long session
fanning out to subagents in worktrees) stopped at the folder.
0011 fixes that. The rest of its report is not fixed. In the
order they block that repo:

1. Where the gate fires. Every turn that changes the repo
   must end in a handoff rewrite (0008). In a session where
   half the turns are a background agent reporting in, that
   is a rewrite every few minutes, each stale at once. The
   candidates are session end (Claude Code gives hooks no
   such signal that fires reliably before the model is gone),
   commit, push, or every turn but only past some amount of
   unhanded work.
2. What a subagent owes. A subagent in a worktree is in a
   different git root. Its moves never reach the session that
   merges its branch, which then owes a handoff for work it
   only saw as one merge. Not known: whether `Stop` or only
   `SubagentStop` fires for it, and what `session_id` its
   tool events carry. That is one probe with a throwaway
   plugin.
3. Two handoffs. Claude Code's own per-project memory already
   carries "start here" in that repo. Which one is the
   handoff, and whether the other shrinks to a pointer,
   depends on who the handoff is for.
4. `check` against a slow oracle. Either `check` is the whole
   truth and rarely run, or fast and partial. A second blessed
   name for the quick subset would settle it. Also where a
   repo records what a check must never do (touch hardware).
5. The pre-edit gate must tell a version bump from a
   dependency change, or every release needs a ritual record.
6. Settled by 0021: rules that are Andrew's everywhere are
   directives in his config. Was: no corpus yet (0004), so they
   (push only with his OK, never commit credentials) still
   live in each repo's `CLAUDE.md`.
7. The stamped decisions template has no `Proposed by:` or
   `Ratified by:`. 0007 was scoped to this repo and to
   decisions about guiding LLM work; in a product repo most
   decisions are the owner's. Left as is until that is argued
   out in a record.

## Non-goals

- No orchestrator, planner/coder/critic roles, or personas.
- No graph store. A flat notes folder with an index is enough
  at personal scale.
- No per-repo customization in the plugin itself; the plugin
  owns the interface, the repo owns what is behind it. Where
  the notes live is the one thing a repo sets.
- Not a general dotfiles repo.

## Acceptance

Done when a fresh repo can be stamped with `heeded init`, a
Claude Code session in it cannot end without a handoff,
cannot touch a trigger path without a decision, logs a
rework line on `/retro`, and CI fails on an unrecorded
dependency change. Each of those is demonstrated by a test,
not by a sentence in a document.

And when gazelle-audio runs under it without Andrew switching
it off.

## Built overnight on 2026-10-09, unreviewed

Andrew was away. These went past what he had seen and each is
his to overturn:

- 0017: a subagent's moves are the session's, worktrees
  included.
- 0018: the stop gate waits while subagents are out.
- 0019: the guard before commits and pushes, the secret scan,
  the changelog reminder, settings.
- The Cursor adapter was drafted from docs after all
  (now `src/harness.rs`, `.cursor-plugin/`). 0014 said to wait for
  a real Cursor. It is installed nowhere and its header says
  it has never run.
- `heeded retro`, with a closed set of three outcomes that
  the plan had not named.

## Next session

1. Read this file and `docs/handoff.md`.
2. Andrew reviews the list above.
3. Commit what is here, if he says to. Nothing since session
   1's first two commits is committed.
4. The pre-edit gate. It needs his answer first on what a
   trigger path is, and a way to tell a version bump from a
   dependency change (item 5 of the second repo's list).
5. Delete `hooks/hook.js`, the shim that hands events from
   sessions started before 0.12.0 to the binary, once none is
   left.
6. Try gazelle-audio again: `heeded init --notes` into its
   private checkout, `permission.commit` set to `allow`
   there, and see what the long oracle and real subagents do.
7. Layer 3, the CI checks, as `heeded` subcommands.

## On the work machine (Cursor)

Before trusting any adapter code: with the Claude plugin
installed and Cursor's third-party import on, see what
already fires. Then check the two reported Windows bugs
(session-start context dropped, stop follow-up lost) on the
real version. 0014 lists the shapes and what was not
confirmed. The `Cursor` adapter in `src/harness.rs` is the draft
to correct.

Blocked as of 2026-10-15: heeded does not run there yet, so the
Cursor adapter cannot be tried there.
