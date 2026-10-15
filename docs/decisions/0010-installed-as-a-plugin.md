# 0010: Installed as a plugin; hooks come from the plugin

Date: 2026-10-09T05:51Z
Status: amended by 0011
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

The plan's layer 1 tree had each stamped repo carry a
`.claude/settings.json` with hooks wired to the plugin's
scripts. Session 1 built the hooks into the plugin's own
`hooks/hooks.json` and asked which should win. Andrew:
install it as a plugin, and use it in this repo's sessions.

## Assessment

Ratified. Hooks copied into each repo's settings are a second
copy of the wiring that goes stale when the plugin changes,
which is the drift this repo exists to remove. Wiring both
would also run every hook twice.

## Decision

- This repo is its own marketplace
  (`.claude-plugin/marketplace.json`, one plugin, source
  `./`). The plugin is installed at user scope from it.
- Hooks come only from the plugin. `heeded init` does not
  write hook wiring into a repo.
- A repo opts in by having `docs/handoff.md`. Everywhere else
  the hooks do nothing.
- This repo's sessions run under the installed plugin, the
  same as any other stamped repo.

## Consequences

- The plan's layer 1 tree loses its `.claude/settings.json`
  hook wiring.
- Claude Code runs an installed copy of the plugin, not this
  working tree. After changing a hook, the installed copy
  must be updated or this repo's sessions run old code.
  `--plugin-dir` still runs the working tree for testing.
- Any repo on this machine with a `docs/handoff.md` is now
  gated, whether or not `heeded init` made it.
