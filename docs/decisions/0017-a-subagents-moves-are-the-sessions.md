# 0017: A subagent's moves are the session's; it owes nothing itself

Date: 2026-10-09T06:50Z
Status: accepted
Proposed by: model (claude-opus-5-5), from the gazelle-audio report's question
Ratified by: claude-opus-5-5

## Context

The gazelle-audio report asked what a subagent is to this
plugin, guessing that one in a git worktree has its own root,
its own journal, and maybe its own stop gate. Andrew was away
when this was decided; it is his to overturn.

A probe with a throwaway plugin that logged every hook input,
in a real Claude Code session on 2026-10-09 with one plain
subagent and one worktree subagent, both foreground, showed:

- A subagent's tool events carry the parent's `session_id`,
  plus `agent_id` and `agent_type`.
- `Stop` does not fire for a subagent. `SubagentStart` and
  `SubagentStop` fire, under the parent's session id.
- `CLAUDE_PROJECT_DIR` stays the parent's repo even when the
  subagent's `cwd` is a worktree under `.claude/worktrees/`.

So the guess was wrong in the plugin's favour. A subagent is
not a second session, and its events already arrive at the
parent's gate. What was missing: an edit inside a worktree
does not change the parent's tree, so it was never journaled,
and the parent later saw the whole branch as one merge.

## Decision

- A subagent writes no handoff and is never refused. Its
  moves are journaled as the session's, tagged with `agent`.
- When an event's `cwd` is another worktree of the same repo,
  that worktree's tree is fingerprinted too. A change there
  is a move, journaled in the parent's journal and owed a
  handoff like any other.
- `SubagentStart` records the worktree's starting fingerprint
  so that a first command that changes nothing is not logged.
- A nested checkout or worktree no longer counts as a change
  to the parent's tree just by appearing or disappearing.

## Consequences

- The handoff for a merged branch can now account for what
  the subagent did, move by move.
- Work on an unmerged branch is owed a handoff. That is
  intended: the next session needs to know the branch exists.
- Each hook call from inside a worktree costs three more git
  calls.
- Background subagents were not probed. Their events are
  assumed to carry the same ids.
- Nothing here exists on Cursor. Its `subagentStart` and
  `subagentStop` payloads were not researched.
- An untracked nested repo that the parent does not ignore is
  now invisible to the gate.
