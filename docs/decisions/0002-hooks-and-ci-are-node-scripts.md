# 0002: Hook and CI scripts are Node, StandardJS

Date: 2026-10-09T05:39Z
Status: superseded by 0024
Proposed by: model (Fable, in the claude.ai chat that wrote the plan)
Ratified by: claude-opus-5-5

## Context

The hooks have to behave identically on Windows and Linux.
Shell scripts would need bash on Windows, and Claude Code runs
hook commands through whatever shell the host gives it.

## Decision

Every hook and CI script is a Node script, invoked as
`node "<path>"`. ESM, `node:` built-ins only at runtime (no
runtime dependencies), StandardJS, no semicolons, 80 columns.
The only shell-out is to `git`.

## Consequences

- Node is a hard requirement of every stamped repo. Decision
  0003 leans on that.
- No `npm install` is needed for the plugin to run; only to
  develop it (`mvt`, `standard`).
- Each hook costs a Node startup (roughly 100ms, plus a few
  `git` calls) per event.
