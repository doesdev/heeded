# 0034: The oracle is a setting; commit and push can be forbidden

Date: 2026-10-23T03:10Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Andrew asked for two things: make the gate, `npm run check`,
configurable, and add a forbidden option to commit and push if
feasible.

heeded never runs the oracle. It names it in the session start
note, and `heeded init` stamps a `package.json` whose `check`
fails until wired up. Both hardcoded `npm run check` (0003).
`permission.commit` and `permission.push` were `ask` or `allow`.

## Assessment

Ratified, both.

- The oracle. 0003 chose one name in every repo so a session
  never has to guess. A setting keeps that: the session is still
  told one command, it is just the repo's own. The cost of 0003
  was a `package.json` in every non-JS repo, and gazelle's slow
  oracle (plan, second repo item 4) wants a say in what is
  named. The setting is `checks.oracle`, not "gate": in this
  code the gate is the stop hook. Settings layer by section, so
  it has to sit in one; `checks` is the nearest.
- Forbidden is feasible because the guard already sees every
  commit and push it recognises, and can deny as easily as ask.
  It is only as strong as that recognition (0015: a guard
  against drift, not a model set on getting round it). Editing
  `.heeded.json` or the user config already needs the owner's
  OK, so a session cannot loosen it unasked.

## Decision

- `checks.oracle`, default `npm run check`, user config then
  repo marker as for every setting. Empty or blank means the
  default. The start note names it.
- `heeded init` stamps the `package.json` and warns about a
  missing `check` script only while the oracle is
  `npm run check`.
- `permission.commit` and `permission.push` take `forbidden`.
  A forbidden act is denied before any other check, for the main
  session and subagents alike, and the refusal says not to try
  another way. Several acts in one command: forbidden wins over
  ask.

## Consequences

- heeded still does not run the oracle. Nothing checks that the
  named command exists.
- A repo's marker can still loosen a user's `forbidden` back to
  `allow`; that edit needs the owner's OK, as any marker edit.
- A forbidden commit that the classifier misses goes through, as
  an `ask` one would.
