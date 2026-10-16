# 0033: The gate after wcmd-deid

Date: 2026-10-16T21:04Z
Status: accepted
Proposed by: model (the wcmd-deid session's report on 0.15.0, Opus 5.5)
Ratified by: claude-opus-5-5

## Context

A long session in wcmd-deid (`commit: allow`, `checks.changelog`
on, notes tracked in `.heeded`) reported four problems. Its
journal bears out each one.

1. Committing the handoff made it stale. The tree fingerprint
   held the HEAD oid, so "write the handoff, commit it" changed
   the fingerprint after the handoff was written. The journal
   shows the lap again and again: "Record X in the handoff"
   (handoff), then "Commit the handoff" (a move).
2. The changelog reminder judged a command before it ran. In
   one call that appended to `CHANGELOG.md`, ran `git add -A`
   and committed, the guard saw no changelog yet and refused.
3. A side session in a worktree of the same repo gated on the
   main checkout's handoff. Its events carry the project folder
   (the main checkout) as `CLAUDE_PROJECT_DIR` and the worktree
   as `cwd`, so the gate's root was the main checkout. The
   session wrote its own worktree's handoff (unseen by the
   gate), then the main one when refused, leaving main dirty.
4. Those sessions' moves landed in the main checkout's journal,
   named by paths under `.claude/worktrees/`, and their Bash
   calls had no description, so `heeded moves` showed a bare
   "Bash".

## Assessment

All four are real. Fixes, against what the report proposed:

1. The report offered two: ignore a HEAD move whose commits
   touch only the notes, or fingerprint content and not HEAD
   at all. The second would stop any commit counting as a
   move, and then a handoff saying "not committed" would pass
   the gate after a commit. A commit is state the next session
   needs. So the narrow one, and narrower than the notes folder:
   a HEAD move counts unless its commits touch only the handoff
   (and journal). Committing work still counts. The lap the
   report saw ends after one turn: write the handoff, commit it
   alone, stop.
2. Taken as a heuristic: a command that stages and names the
   changelog is believed. A git hook at commit time would be
   exact but heeded does not write a repo's git config. The
   refusal now says the check runs before the command.
3. The report's first option: a session first seen in a stamped
   worktree of the project (its `cwd` is that worktree and it
   is not a subagent) keeps that worktree's notes for its whole
   life. The handoff its branch carries is the one it writes
   and is gated on. Pinned at first sight so that a later `cd`
   cannot move it. A worktree with no marker of its own (hidden
   notes, 0012) still shares the project's: there is nowhere
   else to put it.
4. Mostly fixed by 3, since such a session now journals into its
   own worktree. A listing that mixes sessions labels each move
   with the first eight characters of its session. A Bash or
   PowerShell move with no description keeps the program's
   name (`git ...`) and never the rest of the command, which
   can hold secrets (0008).

## Decision

- The snapshot keeps HEAD apart from the fingerprint of changed
  files. When HEAD moves, one `git diff --name-only` between
  the old and new HEAD decides whether it was work.
- A session's checkout is fixed when its first event is seen,
  stored in its state.
- `heeded moves` and the session-start leftover note label moves
  by session when more than one session made them.

## Consequences

- Staging still counts as work, as before.
- Sessions saved by an older binary have no HEAD of their own,
  so the first event after upgrading may log one spurious move.
- A worktree session that writes into the main checkout owes a
  handoff for it, as a subagent would (0017).
- Two sessions in the main checkout still share one handoff.
