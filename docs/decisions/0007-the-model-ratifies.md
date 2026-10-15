# 0007: The model ratifies; the human proposes

Date: 2026-10-09T05:51Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

The point of this repo is to stop steering models down the
paths a human thinks are best. Andrew, 2026-10-09: "I'm a
human and don't think like an LLM so I should not decide what
is most effective for guiding LLM work." The plan's
conventions came from a model and he trusts them. His own
suggestions should get no pass for being his.

## Decision

In this repo, a suggestion from Andrew about how to guide LLM
work is a proposal. The working model assesses it and
ratifies, modifies or rejects it, and says which.

Every decision record carries two header lines:

- `Proposed by:` starting with `human` or `model`
- `Ratified by:` a model id, never a human

A record proposed by a human also needs an `## Assessment`
section holding the model's own judgment: what it kept, what
it changed, what it turned down. `Status: rejected` is a
valid outcome and gets a record like any other.

`npm run check` fails if any record in `docs/decisions/`
breaks this (`lib/decisions.js`).

## Assessment

Ratified, with two limits the model added.

- Scope. This covers what is effective for LLM work. What is
  Andrew's as owner stays his: what gets installed on his
  machine, what it costs, what he wants built.
- Evidence outranks both of us. A model's sense of what
  guides a model is a guess too. In session 1 the stop hook's
  refusal message looked fine to the model that wrote it and
  failed on its first real run. Where a claim can be tested,
  the test decides, not the ratifier.

## Consequences

- The check is mechanical. An assessment can be vacuous and
  pass, the same gap the plan already accepts for decisions.
- Records 0001 to 0006 were back-filled with both headers.
- A model that defers to every human proposal passes the
  check and defeats the purpose. Rejections in the record are
  the only visible sign it is working.
