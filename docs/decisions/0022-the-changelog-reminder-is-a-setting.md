# 0022: The changelog reminder is a setting, off by default

Date: 2026-10-10T01:48Z
Status: accepted
Proposed by: human (Andrew), acting on a point raised in 0021
Ratified by: claude-opus-5-5

## Context

0019's guard refuses a commit once when it touches something
a user could see and no `CHANGELOG.md`. 0021 moved Andrew's
rules out of the plugin and noted that this reminder was the
last of them still shipped as behaviour for every user.
Andrew: make it configurable.

## Assessment

Ratified. The default is my call and I set it to off. "Every
repo keeps a changelog" is his rule, not a safety property,
and after 0021 the test for anything in the plugin is whether
a stranger would want it on unasked. The secret scan and the
permission prompt pass that test. This does not.

The setting is named for what it is, one of the guard's
checks, so that the next one has somewhere to go.

## Decision

- `checks.changelog` in the user config or a repo's marker,
  `true` or `false`, default `false`. Layered like every other
  setting, repo over user.
- When it is off the guard says nothing about changelogs.
- Andrew's own config sets it to `true`.

## Consequences

- Amends 0019.
- His `changelog` directive and this check are now two
  switches for one rule. Turning the directive off in a repo
  does not turn the reminder off there, and the other way
  round.
- A repo can switch the reminder off for itself through its
  marker, and an edit to the marker asks the owner (0019).
- The secret scan and the identity check are still always on
  in a stamped repo. Nobody has asked for them to be
  switches.
