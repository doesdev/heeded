# 0008: A journal, written by a hook, that the stop gate reads

Date: 2026-10-09T05:51Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Andrew keeps a `journal.jsonl` at work as a running log of
each move, and offered it here, unsure whether the design
already covered it or whether it was excess.

Record 0006 had two gaps. One handoff write satisfied the
whole session, so the demand landed at the first stop after
work and never again. And a session that gave up, crashed or
ran out of context left nothing behind for the next one.

## Assessment

Ratified in a different form from the one proposed.

Turned down: a journal the model writes. A line per move,
authored by the model, is the kind of per-step practice this
repo exists to get rid of. It costs attention on every move,
it decays, and the transcript already holds the same
information.

Kept: the journal itself, because written by a hook it costs
the model nothing and closes both gaps in 0006. It gives the
gate an ordering (did work happen after the handoff was
written?) that comparing two snapshots cannot, and it is the
only record that survives a session that never handed off.

## Decision

- A `PostToolUse` hook on the tools that change things
  (`Edit`, `Write`, `NotebookEdit`, `Bash`, `PowerShell`)
  compares the repo with its last snapshot. If the repo
  changed, it appends one line to `docs/journal.jsonl`:
  `{ts, session, tool, what, handoff}`. A tool call that
  changed nothing is not a move and is not logged.
- `what` is the file path for edits and the tool's
  description for commands. The command text is never
  stored; commands carry secrets.
- The split with the stop gate: the journal records, the gate
  decides. `Stop` refuses while any move is newer than the
  last handoff write, and lists those moves in the refusal.
  Work after a handoff makes it stale again.
- Changes made while the session is idle are the human's.
  `UserPromptSubmit` and `SessionStart` log them as `outside`
  and they are not owed a handoff.
- `SessionStart` prints any moves earlier sessions made after
  the last handoff write, so the next session knows the
  handoff is behind.
- The journal is gitignored, excluded from the tree
  fingerprint, and trimmed to its newest 500 lines.

## Consequences

- Every turn that changes the repo ends with a handoff
  rewrite. That is a few hundred tokens a turn, and the only
  end-of-session signal Claude Code gives a hook.
- What a session owes lives in the plugin's data directory,
  not the journal, so deleting the journal does not get a
  session past the gate.
- The hook runs after every edit and command, at roughly 0.4s
  each on Windows (Node start plus three `git` calls).
- Parallel tool calls race on the session state file. The
  worst outcomes are a duplicate journal line or a move
  missing from the refusal's list; the gate still refuses.
  No lock yet.
- Session budget and `/handoff` can read the journal when
  they are built. Neither exists yet.
