# 0011: A marker file opts a repo in and names its notes folder

Date: 2026-10-09T06:18Z
Status: accepted
Proposed by: human (Andrew), from a model's migration report
Ratified by: claude-opus-5-5

## Context

The first attempt to adopt the plugin in a second repo
(gazelle-audio) stopped at step one. That repo's `docs/` is a
product: a user manual that builds to a PDF, checked on every
CI run. The plugin hard-coded `docs/handoff.md` and
`docs/journal.jsonl`, stamped a plan, decisions and rework
log beside them, and used `docs/handoff.md` as the opt-in
signal. Andrew called putting the plugin's files in `docs/` a
design issue and paused the migration.

The report from that session (a Claude Code session in
gazelle-audio, which read the plugin at 0.2.0 and changed
nothing) proposed a per-repo notes root named by a marker
file, with a default folder that is plainly the tool's. It
also pointed out that the repo is public and keeps its notes
in a separate private repository checked out inside it and
ignored.

## Assessment

Ratified, and it would have been right without the owner's
objection. 0004 said per-project state lives in the project's
`docs/`. That confused two things: that the state belongs to
the project (still true) and that the plugin gets to pick the
folder (never justified). `docs/` only looked free because in
this repo it was empty.

Three things the report did not propose, added here because
the same change forces them:

- The stamped `CLAUDE.md` goes. A static file cannot name a
  folder chosen per repo, and `heeded init` never overwrites,
  so a repo with its own `CLAUDE.md` was stamped with no
  pointer to the notes at all. The session-start hook knows
  the folder, so it says so. That is the plan's own first
  principle: context assembled by the environment.
- The journal's ignore line moves into the notes folder. The
  scaffold no longer edits the repo's `.gitignore`, and the
  line is right whichever repository the notes turn out to
  be in.
- A marker alone does not gate. Both the marker and the
  handoff it points at must exist. A fresh clone of a public
  repo whose notes are a private checkout has the marker and
  no notes; it must be left alone, not refused at every stop.

Turned down: the report's suggestion that a repo "declare an
existing convention as the implementation" of plan, handoff
or decisions. Pointing the notes folder at where a repo
already keeps such things covers the folder. Mapping file
names and formats inside it is a second mechanism with one
example behind it. It waits for a second.

## Decision

- A repo opts in with `.heeded.json` at its root:
  `{ "notes": "<folder>" }`, relative to the root or
  absolute. A broken marker is an error, not an opt-out.
- The notes folder holds `handoff.md`, `plan.md`,
  `decisions/`, `rework.jsonl`, `journal.jsonl` and a
  `.gitignore` that ignores the journal. `heeded init` stamps
  it at `.heeded` unless told otherwise with `--notes`; an
  existing marker wins.
- `heeded init` writes the marker, the notes folder and, if
  there is none, a `package.json` for `check`. Nothing else.
- On session start in a stamped repo the hook says where the
  notes are and what to read first.
- A session that deletes the marker is refused at stop, the
  same as one that deletes the handoff.
- This repo's own marker names `docs`. Its `docs/` holds
  nothing else.

## Consequences

- Amends 0004 (the folder is the repo's choice) and 0010 (the
  opt-in is the marker, and the handoff behind it).
- `docs/handoff.md` on its own no longer opts a repo in. The
  only repos affected were this one and test scratch.
- Whether notes are tracked, public or private is now the
  repo's choice. The plugin has no opinion and no check. A
  marker tracked in a public repo publishes the notes
  folder's path, though not its contents.
- Notes in a nested checkout the outer repo ignores are read
  by path and work; a test covers it. Edits to them are not
  work the outer repo's gate sees.
- A git worktree of a repo whose marker is tracked is itself
  stamped, with its own notes folder if that is tracked and
  none if it is ignored. What a subagent in a worktree owes
  is still undefined.
- The plan's CI check on `CLAUDE.md` length loses its reason
  and is dropped from the plan.
- The session-start note costs about 60 tokens a session and
  is one more thing only a test, not a reader, will notice
  going stale.
