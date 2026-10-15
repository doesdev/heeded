# 0015: Permission and commit identity come from config

Date: 2026-10-09T06:32Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

gazelle-audio's rule is "push, tag or publish only with the
user's OK; commit freely on a branch". Asked whether that is
his everywhere, Andrew said it belongs in config: the default
is that everything needs permission, and commit identity is
config too. Asked which config: a user-level default that a
repo can override. Asked what identity covers: both the git
author (name and email) and how commits attribute a model.

## Assessment

Ratified. What may leave his machine and under whose name is
his call, and config is the right home: the same person wants
different answers at work and at home, and in gazelle than in
a scratch repo. A sentence in a rules file cannot vary by
machine.

Two things I am adding:

- Config that is only read aloud to the model is still a rule
  in prose. The point of putting it in config is that a hook
  can read it and refuse a `git commit` or `git push` that it
  does not allow. That gate is what gets built, not a
  paragraph generated from the config.
- A repo's config can tighten or loosen the user default, and
  a repo's config is a file a model can edit. The gate must
  treat a change to these keys as needing the owner's OK, or
  it gates nothing.

## Decision

- User config holds the defaults: permission to commit and to
  push (which covers tag, publish and release), each `ask` by
  default, and the commit identity: author name, author email
  and the attribution policy.
- A repo's `.heeded.json` may override any of them.
- A pre-tool hook checks commits and pushes against the
  effective config, and checks the author on a commit.

## Consequences

- Not built. Until it is, the `permission` rule in
  `corpus/rules.md` states the default and lets a repo's
  written instructions loosen it.
- The file's name and place wait on the project's rename.
- On Cursor this needs `beforeShellExecution` or
  `preToolUse`, which were not part of the research for 0014.
- Recognising a commit or push in an arbitrary shell command
  is pattern matching and will miss some. It is a guard
  against drift, not against a model set on getting round it.
