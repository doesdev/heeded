# 0027: A repo holding a stamped repo's notes is guarded as that repo

Date: 2026-10-15T06:13Z
Status: accepted
Proposed by: model (claude-opus-5-5), from the gazelle-audio evaluation's open question
Ratified by: claude-opus-5-5

## Context

gazelle-audio keeps its notes in `.agent`, a private git
checkout nested in the repo and ignored by it. 0023 made a
command's repo its `-C` or last `cd`, and left an unstamped
repo to itself, so `.agent` commits stopped being judged as
gazelle's. That also meant `git -C .agent push` left the
machine unasked, against Andrew's directive to ask before
anything leaves it, and notes commits got no secret scan,
though notes are where a session pastes a token while writing
up a failure.

Two options were put to Andrew: treat the repo that holds the
notes as part of the stamped repo, or ask before every push
from a stamped session whatever repo it lands in. He chose
the first.

## Assessment

The marker already ties the notes to the repo, so judging
their checkout by the repo's settings follows from 0011 rather
than adding a rule. The second option would reach repos the
session only passes through and still leave notes commits
unscanned.

## Decision

- When a command lands in a repo with no marker that is not a
  worktree of a stamped one (0026), and that repo holds the
  folder the marker's `notes` names, and does not itself hold
  the stamped repo, the guard judges it with the stamped repo's
  settings: secret scan, identity, attribution, and permission
  for commit and push.
- The changelog reminder is skipped there: notes never change
  what the repo's users see.
- Works for notes nested in the repo or outside it; a folder on
  another drive is never taken for inside.

## Consequences

- With gazelle's planned `commit: allow`, `.agent` commits
  stay quiet unless they hold a secret; its pushes ask.
- A repo that holds the notes but is otherwise unrelated (a
  big monorepo of notes, say) is judged by the stamped repo's
  settings too. Unlikely, and the stricter side.
- Reads the marker once more, only for commands that commit,
  push or change settings, in a repo with no marker of its own.
