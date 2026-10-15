# 0012: Notes are private by default; tracking them is opt-in

Date: 2026-10-09T06:32Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

0011 let a repo choose its notes folder and left "tracked or
not, public or not" to the repo with no default. Asked
whether session notes in a public repo should ever be public,
Andrew answered: private by default, public optional. He also
answered that the handoff is for the model, not for him.

## Assessment

Ratified. A model writes the handoff on every turn with no
review. Publishing that by default means every turn must meet
publication standards, in repos that have "never commit X"
lists. Since the reader is the next model session on the same
machine, nothing is lost by keeping it local.

The mechanism is mine. `.git/info/exclude` hides the marker
and the notes folder from this clone's git without touching
the repo's `.gitignore`, which 0011 promised not to edit and
which would itself publish the folder's name.

The cost I am accepting on his behalf, and he should know it:
by default the notes are no longer versioned and are absent
from a fresh clone and from every git worktree. "The repo is
the source of truth" now means the working copy, not the
history. A repo that wants history for its notes points the
marker at a private checkout (gazelle-audio's arrangement) or
stamps with `--tracked`.

## Decision

- `heeded init` adds what it created (the marker, the notes
  folder) to `.git/info/exclude`. It hides nothing that was
  already there or is already ignored.
- `heeded init --tracked` skips that, and the notes are
  committed with the repo like any other file.
- Outside a git repo there is nothing to hide and nothing is
  done.

## Consequences

- A fresh clone, and a subagent's worktree, of a default
  stamped repo has no marker and is not gated. For worktrees
  that may be the right answer to "what does a subagent owe";
  it is not yet a decision.
- Losing the working copy loses the notes.
- This repo stays tracked: its marker and `docs/` predate the
  default and init leaves existing files alone.
