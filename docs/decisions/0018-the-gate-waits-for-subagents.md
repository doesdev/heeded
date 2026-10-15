# 0018: The stop gate waits while subagents are still out

Date: 2026-10-09T06:50Z
Status: accepted
Proposed by: model (claude-opus-5-5), from the gazelle-audio report's finding
Ratified by: claude-opus-5-5

## Context

0008 made every turn that changes the repo end in a handoff
rewrite. The gazelle-audio report said that does not fit a
session where work fans out to background subagents and half
the turns are an agent reporting in. Its two cases: a turn
that only merges, stale minutes later when the next agent
reports; and a turn that ends while agents are still running,
where the honest handoff is "three agents are still out",
which is a status. It suggested gating at session end, at a
commit or before a push instead.

Andrew was away when this was decided; it is his to overturn.

## Decision

Keep the gate at stop, and let it pass while any subagent
started in this session has not stopped. The session state
counts them from `SubagentStart` and `SubagentStop`. What is
owed stays owed, so the first stop after the last agent is
back is refused as before.

A subagent whose stop was never seen stops counting after six
hours, so a crash cannot hold the gate open for good.

Why not the alternatives:

- Session end. Claude Code's `SessionEnd` hook cannot refuse
  or speak to the model, and on a crash or a closed window
  nothing fires at all. Stop is the only point where a hook
  can still make the model write.
- Commit or push. Plenty of sessions end without either, and
  under 0015 both wait on Andrew.
- A budget of moves before the gate bites. It trades a known
  cost for a silent gap, on a number nobody has measured.

Both of the report's cases are turns that end with agents
out. Waiting for them is the smallest change that covers
both, and it asks for the handoff at the one moment in a
fan-out when it can be complete.

## Consequences

- Amends 0008. A turn can now end with unhanded work.
- If the session dies while agents are out, the handoff is
  behind. The journal still has every move, and the next
  session is told what was left unhanded.
- A session that never uses subagents sees no change, and
  still rewrites the handoff every turn that changes the
  repo. Whether that is too heavy is still unmeasured.
- Assumes background subagents fire the same two hooks as
  foreground ones. Not probed.
- No effect on Cursor until its subagent hooks are mapped.
