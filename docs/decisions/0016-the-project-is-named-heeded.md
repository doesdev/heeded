# 0016: The project is named heeded

Date: 2026-10-09T06:41Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

The project was `my-ai`. Andrew does not like a dash in
plugin and config names and wanted a single word, ideally one
that could ship publicly. He considered Archon and dropped
it: the name is already used by another project in the same
space. From a list checked against the npm registry he chose
`heeded`.

## Assessment

His to choose. Two facts for the record, neither an
objection:

- `heeded` was free on npm on 2026-10-09. Only npm was
  checked, not GitHub or other AI tooling.
- It sits beside his existing `heedful` package, which helps
  if this ships and could confuse if the two are unrelated.

The rename was done before anything was committed or adopted
by a second repo, which is the cheap time.

## Decision

Everything that carried the old name carries the new one: the
npm package, the Claude Code plugin and its marketplace, the
CLI (`heeded`), the marker (`.heeded.json`), the default notes
folder (`.heeded`), the skills (`/heeded:init`,
`/heeded:handoff`), and the text the hooks print.

Earlier decision records were edited to say `heeded` so that
the commands and file names in them still exist. Nothing else
in them changed.

## Consequences

- A repo stamped under the old name has a marker the plugin
  no longer reads. The only ones were this repo and scratch.
- The repo's folder on this machine is still `repos/my-ai`,
  and Claude Code's per-project memory is keyed to that path.
  Renaming the folder is separate and loses nothing but needs
  the memory folder moved with it.
- The old plugin and marketplace entries were removed and the
  new ones installed. The old global `my-ai` command was
  unlinked.
- `docs/journal.jsonl` keeps its old lines as written.
