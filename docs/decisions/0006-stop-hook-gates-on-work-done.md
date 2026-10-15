# 0006: The stop hook gates on work done, and gives up loudly

Date: 2026-10-09T05:39Z
Status: amended by 0008
Proposed by: model (claude-opus-5-5)
Ratified by: claude-opus-5-5

## Context

The plan says: refuse to end the session if `docs/handoff.md`
was not modified this session. Claude Code has no blockable
"session is ending" event. `Stop` fires at the end of every
turn, and `SessionEnd` cannot block. Taken literally, the
rule would demand a handoff after answering a question.

A hook that always refuses can also trap a session that
cannot write (plan mode, denied permissions) in a loop.

## Decision

- `SessionStart` records a baseline: a hash of the handoff
  and a fingerprint of the tree (HEAD, `git status`, and the
  contents of every changed or untracked file, handoff
  excluded).
- `Stop` refuses only when the tree differs from the baseline
  and the handoff does not. No work, no demand.
- A repo with no `docs/handoff.md` is not stamped and is left
  alone. Deleting the handoff mid-session is refused.
- After 3 consecutive refusals the hook lets the session stop
  and prints a warning to the user instead.
- The refusal names the handoff by absolute path. With a
  relative path, the first real run had the model go looking
  in the plugin's own repo.

## Consequences

- One handoff write satisfies the whole session. Work done
  after it is not caught; the handoff can be stale by the
  real end. Tightening this (re-arm after each pass) is a
  small change and is deliberately left until it bites.
- Outside a git repo there is no tree fingerprint, so the
  hook never refuses.
- If the plugin is enabled mid-session there is no baseline;
  the first stop records one and passes.
- Baselines accumulate in the plugin data directory, one
  small file per session. Nothing prunes them yet.
