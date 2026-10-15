# 0029: A subagent is refused where the session would be asked

Date: 2026-10-15T15:17Z
Status: accepted
Proposed by: model (claude-opus-5-5), from the gazelle-audio session's live test
Ratified by: claude-opus-5-5

## Context

The gazelle-audio session's step 6 set `commit: ask` for one
turn and had a background subagent commit in its worktree. The
commit went straight through: the subagent saw no prompt and no
refusal. Replayed through `heeded hook`, the same command
answered `ask`, with and without `agent_id`. In the same turn
every `ask` from the main session reached Andrew. So the
harness passed the background subagent's `ask` without asking.
Whether that holds for every permission mode, or for foreground
subagents, one run cannot say. Andrew agreed to the proposal.

## Assessment

An `ask` that may be passed silently is no gate. A `deny` is
honoured everywhere. Refusing with a reason the subagent can act
on (stop, report back) moves the decision to the main session,
where an `ask` is known to reach the owner.

## Decision

- When a PreToolUse event carries a non-empty `agent_id` and
  the guard would answer `ask` (a commit or push the settings do
  not allow, a change to heeded's settings), it answers `deny`
  with the same reason plus: a subagent cannot get that OK; stop
  and report back; the main session will ask.
- Refusals and allows are unchanged; with `allow` a subagent
  commits as before.

## Consequences

- With `commit: ask`, subagents cannot commit at all; the main
  session commits their branches. With `allow`, as gazelle has
  it, nothing changes.
- Foreground subagents are refused too, though their `ask` might
  have reached the owner. Not worth telling apart without
  evidence; the harness gives no reliable signal for it.
- If Claude Code later raises a background agent's `ask` to the
  owner, this can be dropped.
