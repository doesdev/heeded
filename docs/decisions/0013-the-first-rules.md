# 0013: The corpus starts as one rules file, said at session start

Date: 2026-10-09T06:32Z
Status: amended by 0021
Proposed by: human (Andrew) for each rule; model for the format
Ratified by: claude-opus-5-5

## Context

0004 said conventions that hold across projects live in the
plugin as a corpus, and left its format undecided. The
gazelle-audio report pointed out that some of that repo's
hard rules are Andrew's everywhere and had nowhere to move
to. Andrew went through them one at a time.

His answers:

- No em or en dashes: everywhere, in all text, not only
  product text.
- Tests never touch hardware: everywhere, generalised to real
  devices, production data and live services.
- Never commit credentials: everywhere, and heeded should
  enforce it, not leave it to each repo.
- Push only with his OK, commit freely: not as written. It
  goes in config, the default is that everything needs
  permission, and commit identity goes in config too (0015).
- A changelog line with every user-visible change: everywhere,
  always, not only where a changelog already exists.
- Changelog lines written for the user: everywhere.

## Assessment

What Andrew wants written and what may leave his machine are
his as owner; 0007 gives the model no vote on them. Recorded
as given, including two where I recommended otherwise (dashes
in product text only; changelog only where one exists).

Where the model does have a say is how a rule reaches a
session and whether it holds:

- One file, six short rules, printed whole at session start.
  0004 pictured a searchable corpus. At six rules a search is
  machinery with nothing to find; every rule applies to every
  turn. It becomes searchable when it is too long to print.
- A rule in prose is the thing this repo exists to replace.
  Each of these needs a check or it will decay. Only one has
  one so far: this repo's tests fail on a dash. The rest are
  listed below as owed.
- "Everywhere" stops at stamped repos for now. The plugin
  says nothing in a repo that has not opted in. Printing the
  rules in every repo on the machine is one line to change and
  is Andrew's call.

## Decision

- `corpus/rules.md` holds the rules, each with a short id.
- The session-start hook prints it in a stamped repo.
- A repo's own instructions may add rules. They may not
  loosen one, except `permission`, until 0015 is built.

## Consequences

- About 250 tokens a session.
- Owed enforcement: a secret scan heeded ships (credentials),
  a commit and push gate (permission, 0015), a changelog
  check (a commit that changes behaviour and not
  `CHANGELOG.md`), a dash check stamped repos can run.
  `live-systems` has no general check; each repo records how
  its tests stay off the real thing.
- This repo now needs a `CHANGELOG.md` and has one.
- On Cursor the session-start text is not reliably delivered
  (0014), so there the rules need a second route.
