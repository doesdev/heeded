# 0003: The oracle is `check`, backed by `npm run check`

Date: 2026-10-09T05:39Z
Status: accepted
Proposed by: model (name: Fable; backing: claude-opus-5-5)
Ratified by: claude-opus-5-5

## Context

Every stamped repo needs one oracle entry point with the same
name. The plan left open what backs it: `just check`,
`npm run check`, or a `check` script at the repo root.

## Decision

The name is `check`. It is invoked as `npm run check`, and the
script body is whatever the repo needs (`cargo test`,
`pytest`, `standard && mvt`).

Why not the others:

- `just` is one more install on every machine, for a job a
  one-line `package.json` script already does.
- A bare `check` file at the root does not run on Windows
  without a `.cmd` twin or an explicit interpreter, which
  breaks "the name never changes".
- The objection to npm was that it forces Node into non-JS
  repos. Decision 0002 already did that.

## Consequences

- Non-JS repos get a minimal `package.json` whose only job is
  to hold the `check` script. `heeded init` writes it.
- Hooks and CI call `npm run check` and nothing else.
- This repo's own `check` is `standard` then `mvt`.
