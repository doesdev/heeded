# 0023: The guard, fixed against a real repo's commands

Date: 2026-10-15T04:57Z
Status: accepted
Proposed by: model (the gazelle-audio session's re-assessment, Opus 5.5)
Ratified by: claude-opus-5-5

## Context

The gazelle-audio session re-assessed heeded at 0.9.0 against
that repo, read-only: the hook's cost per call, the secret
scan over all 703 tracked files, the command classifier over
the commands it actually runs, and the changelog reminder over
typical commits. Andrew asked for its findings to be worked
on. This record covers the ones that changed heeded.

## Assessment

Ratified, all of them, because each came with the command or
the number that showed it. One more was found while doing the
work: the guard in this session judged
`cd <scratch repo> && git commit` as a commit to this repo,
and refused it. That is the same bug as `git -C`, from the
other direction.

## Decision

- `git tag` asks only when it makes or deletes a tag. Listing,
  `--points-at`, `--contains` and verifying do not.
- `gh release` asks only for `create`, `upload`, `edit` and
  `delete`. `view`, `download` and `list` do not.
- A git command's repo is the one its `-C` names, else the last
  `cd` before it, else the session's. The guard judges a commit
  or push by that repo's settings, and leaves it alone if that
  repo is not stamped. A `cd` into a path with a variable in it
  cannot be known and counts as the session's repo.
- `heeded config`, other than `--show`, asks the owner, the
  same as editing a settings file with an edit tool (0019).
- `checks.quiet` lists paths the changelog reminder ignores,
  by exact path or folder prefix. Set it in the marker.
- `heeded init` hides the placeholder `package.json` it writes,
  like the marker and the notes (0012).
- The hook leaves at once when there is no marker in the
  project folder or above it and no state for the session. No
  git runs, nothing is written, and only the adapters load.

## Consequences

- Amends 0019 and 0012.
- Measured on this machine afterwards, per hook call: Node
  alone starts in about 40 ms. The hook in a repo that never
  opted in now takes about 46 ms, the same as an empty ES
  module. In a stamped repo it takes about 170 ms, of which
  roughly 70 ms is three git calls.
- A merge makes a commit and is not classified. The merged
  commits were each guarded when they were made, so the gap is
  the merge commit itself.
- Whether a background subagent can raise the owner's prompt,
  or is refused as in a headless run, is still not probed. It
  decides whether `commit: ask` works anywhere agents commit.
- A `cd` in a subshell or an alias the guard cannot see still
  fools it. This stays a guard against drift.
