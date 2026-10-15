# 0020: The owner's own directives come from config

Date: 2026-10-09T07:49Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Andrew asked for two things: a CLI command that generates the
user config, and config options for "specific extensions such
as a user voice directive, style directives, etc".

Until now the only text heeded put in front of a session was
its own notes pointer and `corpus/rules.md`, six rules of his
that ship inside the plugin.

## Assessment

Ratified, in a more general form than asked.

- The command is plainly right. 0019 left the user config as
  a file to write by hand from a README, which is the kind of
  remembered step this project exists to remove.
- For directives I did not add a `voice` key and a `style`
  key. I added one map, `directives`, from a name he chooses
  to either text or a file. A fixed list of kinds would be me
  deciding in advance what he will want to say. The names are
  his; `voice` and `style` are just the first two.
- A directive is prose shown to a model, which is the weakest
  thing this project does. Nothing checks that one is
  followed. That is acceptable for voice and style, where no
  check is possible, and it is why the six rules that can be
  checked are getting hooks instead. A directive should not
  be where a rule that matters goes to live.
- Every directive costs context in every session of every
  stamped repo. One taken from a file is cut at 4000
  characters so that a long style guide cannot quietly eat a
  session.
- A repo's marker can add directives too, and a marker can
  arrive in a clone of somebody else's repo. That is the same
  trust a repo's `CLAUDE.md` already gets, with one limit
  added: a marker may only name files inside its own repo, so
  it cannot pull an arbitrary file off the machine into the
  session.

One thing this makes visible and does not settle: the six
rules are Andrew's, including one about dashes, and they ship
in the plugin. If heeded is ever public they are in the wrong
place, and this is the mechanism they would move to.

## Decision

- `heeded config` writes `config.json` in the user's heeded
  folder with every setting at its default, and the identity
  from git's global `user.name` and `user.email`. It never
  overwrites. It then prints what is in effect in the current
  repo, after the repo's marker is layered on.
- `directives` is a map of name to a string, or to
  `{ "file": "<path>" }`. A path is relative to the folder of
  the config file it appears in.
- The user's directives and the repo's are merged by name;
  the repo's wins. Anything that is not text or a file
  (`false`, an empty string) switches that name off.
- A stamped repo's session is shown them at start, after the
  rules.

## Consequences

- Seeding the identity from git turns the author check on:
  after `heeded config`, a commit under any other author is
  refused in a stamped repo until the config or the repo's
  marker says otherwise. On a machine with one identity that
  is a no-op; on one with a work and a personal email it will
  bite in whichever repos use the other.
- The command shows what is in effect but does not change a
  value. Editing is by hand, and through a session's edit
  tools it asks the owner (0019).
- On Cursor directives ride on the same session-start text as
  the rules, with the same doubt about delivery (0014).
- Nothing measures whether a directive changes what a model
  writes.
