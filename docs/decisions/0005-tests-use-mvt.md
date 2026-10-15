# 0005: Tests use mvt

Date: 2026-10-09T05:39Z
Status: superseded by 0024
Proposed by: model (Fable, in the claude.ai chat that wrote the plan)
Ratified by: claude-opus-5-5

## Context

The plan's acceptance bar is that each guarantee is
demonstrated by a test. The hooks are small process-level
scripts; the test runner needs to do very little.

## Decision

Tests use `mvt@^5` in a flat `test.js`. Pure logic gets
table-driven cases. Each hook is also tested as a real child
process against a throwaway git repo, with the same stdin
JSON Claude Code sends, asserting on exit code and output.

## Consequences

- `npm test` needs `git` on the PATH.
- The process-level tests cost about half a second each.
- They do not prove Claude Code calls the hook the way the
  tests do. That was checked by hand in session 1 with a
  headless `claude --plugin-dir` run, and is not automated.
