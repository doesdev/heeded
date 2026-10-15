# 0026: A linked worktree is guarded as the repo it belongs to

Date: 2026-10-15T06:03Z
Status: amended by 0027
Proposed by: model (the gazelle-audio session's evaluation of 0.12.0, Opus 5.5)
Ratified by: claude-opus-5-5

## Context

The gazelle-audio evaluation of 0.12.0 found that a commit or
push made in a subagent's worktree passed the guard unchecked
when the repo's marker is hidden, which `heeded init` does by
default (0012). The guard counted a command as stamped only
when the repo it resolves to (0023: `-C`, the last `cd`) is
the project or holds a marker. A linked worktree resolves to
its own top, and an untracked marker is never checked out
there. In gazelle every feature is committed in a worktree,
so the secret scan, identity, attribution, changelog and
permission checks all missed the commits that matter most.
Reproduced with a test before the fix.

The stop gate was not affected: it already treats a worktree
that shares the project's git folder as a side of it (0017).

## Assessment

The evaluation's fix is right. A worktree is the same repo
with another checkout; its commits land in the same history
and its pushes go to the same remotes. Judging it by its own
top was an accident of 0023, not a choice.

## Decision

- When the repo a command resolves to has no marker, the
  guard looks at the git folder its worktrees share. If that
  folder's parent (the main checkout) is stamped, the command
  is gated with the main checkout's settings and notes.
- The top and the shared folder come from one `rev-parse`, so
  a guarded command costs no extra git start.

## Consequences

- `permission.commit` now governs subagents' commits. Whether
  a background subagent can raise an `ask` at all is still
  unknown; one live turn with `commit: ask` answers it.
- A worktree whose own session starts outside the stamped
  repo (its cwd is not under the marker) is still not seen:
  the guard finds the project by walking up from the event's
  folder, without git, so that unstamped repos cost nothing.
  Claude Code puts its worktrees under `.claude/worktrees/`,
  inside the repo, so this does not bite there.
- A repo nested in a stamped one, like gazelle's `.agent`
  notes checkout, is still its own business: its push is not
  asked. Whether it should be is Andrew's call. (Settled by
  0027: it is guarded as the stamped repo.)
