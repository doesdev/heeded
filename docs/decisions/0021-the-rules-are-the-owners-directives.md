# 0021: The rules are the owner's directives; heeded ships none

Date: 2026-10-10T01:20Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

0013 put six rules of Andrew's in `corpus/rules.md`, inside
the plugin, printed to every stamped session. 0020 added
`directives` to config and noted that if heeded ever ships
publicly those rules are in the wrong place. Andrew: "Move my
rules to directives."

## Assessment

Ratified. The rules were always his and never the tool's. A
rule about dashes shipped to every user of a plugin is the
same mistake as the plugin choosing `docs/` (0011): the tool
deciding something that belongs to its owner. It also fixes a
practical gap. His work machine needs different answers, and
a file inside the plugin could not vary by machine.

What moving them does not change, and I would not want it to
quietly: three of the six have a hook behind them (0019), and
the hooks stay in the plugin, on for everyone.

- `credentials` and `permission` are safe defaults for any
  user, and `permission` is already a setting.
- The changelog reminder is not. "Every repo keeps a
  changelog" is his rule, and the guard still reminds every
  user of a stamped repo once per commit. It is one refusal
  and then passes, so it is cheap, but it is the last opinion
  of his that ships as behaviour. It should become a setting
  before heeded is public. Not done here.

Two costs of the move:

- The rules now live in a file outside any repo, unversioned
  and on one machine. They have to be copied to the work
  machine by hand, and nothing notices if the copies drift.
- A repo's marker can switch off a directive by name (0020).
  As a shipped rule that was not possible. The text shown to
  a session says a repo may add to the directives and not
  loosen them, but for these six nothing enforces that beyond
  the three hooks.

## Decision

- `corpus/rules.md` is deleted. The plugin prints no rules of
  its own; the session-start text is the notes pointer and
  the owner's directives.
- Andrew's six rules are six named directives in his user
  config, under the same ids, with the same wording.
- 0004's corpus, as something the plugin ships, is retired. A
  convention that holds across one person's projects lives in
  that person's config.

## Consequences

- Amends 0013 and 0004.
- A new user of heeded gets the gate, the journal and the
  guard, and no prose rules until they write some.
- A stamped repo on a machine with no user config is told
  nothing but where its notes are.
- This repo's own test for dashes stays. It is this repo's
  check, not a shipped rule.
- On Cursor the directives ride on session-start text whose
  delivery is in doubt (0014), as the rules did.
