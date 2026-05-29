# Multi-Chat Project Playbook — Agentic

A starter playbook for running a software project with Claude
in a minimally supervised agentic configuration. Same three
core roles as the human-orchestrated variant (Design,
Implementation, Marketing) plus two additions:

- **Engineers** — Claude Code sessions executing prompts from
  Implementation chat
- **Orchestrator** — fills the human product-owner-and-merge-
  coordinator role, with strategic escalation to the human at
  bounded checkpoints

Read the human-orchestrated playbook first if you haven't;
this document builds on the same patterns and assumes
familiarity with the underlying conventions (spec-as-truth,
worktree sessions, verification gates, cross-chat handoff,
timestamp discipline, etc.).

The agentic additions: explicit role authority, escalation
protocols, audit machinery, and tighter verification because
mistakes compound when no human is in the loop to catch them
mid-stream.

---

## What this gives you

A working agentic multi-chat workflow with:

- **Five role types** with explicit authority boundaries
- **Bounded autonomy** for each role, with escalation to human
  at strategic checkpoints
- **Audit trails** that let the human review periodically
  without recreating context
- **Inter-role verification** where outputs from one role get
  reviewed by another before action
- **Failure mode protocols** for the categories of breakdown
  that agentic systems hit most often

You play strategic director: initial direction, periodic
review, decisions at escalation points, scope changes. The
orchestrator handles routine coordination.

---

## Role architecture

```
                    ┌──────────────────┐
                    │       You        │   strategic direction,
                    │ (strategic       │   escalation handling,
                    │  director)       │   periodic review
                    └────────┬─────────┘
                             │
                             ▼
                    ┌──────────────────┐
                    │   Orchestrator   │   work routing,
                    │                  │   merge coordination,
                    │                  │   tracking files,
                    │                  │   audit
                    └────┬─────┬───┬───┘
                         │     │   │
              ┌──────────┘     │   └────────────┐
              ▼                ▼                 ▼
       ┌─────────────┐  ┌──────────────┐  ┌────────────┐
       │   Design    │  │ Implementation│  │ Marketing  │
       │             │  │               │  │            │
       └──────┬──────┘  └───────┬──────┘  └────────────┘
              │                  │
              │                  ▼
              │           ┌────────────┐
              │           │ Engineers  │ ← session execution
              │           │ (CC)       │   in worktrees
              │           └────────────┘
              │
              └─→ changelogs, spec amendments
```

### Authority matrix

| Decision type | Authority | Escalation point |
|---------------|-----------|-------------------|
| Spec amendments | Design chat | Resolves directly via changelog |
| Implementation approach | Implementation chat | Within design constraints |
| Marketing copy | Marketing chat | Within accuracy constraints |
| Work routing between chats | Orchestrator | Routine |
| Session scheduling | Orchestrator | Routine |
| Merge coordination | Orchestrator | Routine PRs |
| Strategic priorities | **You** | Always |
| External announcements | **You** | Always |
| Production releases | **You** | Always |
| Resource/tooling choices | **You** | Always |
| Conflict between roles | Orchestrator escalates | When roles disagree |
| Repeated session failures | Orchestrator escalates | After N retries |
| Out-of-scope discoveries | Orchestrator escalates | When work reveals new scope |

---

## Role definitions

### Orchestrator role

The orchestrator is the agentic system's product-owner
substitute. It does NOT make strategic decisions — those
escalate. It DOES handle routine coordination.

**Responsibilities:**

- Maintain the work queue across chats (what's pending in
  Design, in Implementation, blocked on dependencies)
- Route handoffs between chats (OPEN from Implementation →
  Design; FOUND from Engineering → Implementation; spec
  changes from Design → Marketing if user-facing)
- Schedule engineering sessions based on Implementation chat's
  prompts and the dependency graph
- Coordinate merges for routine PRs (sessions whose
  verification gates pass cleanly)
- Update tracking files (STATUS.md, ROADMAP.md, follow-up
  lists) as work lands
- Maintain audit log of decisions and actions
- Detect and escalate the situations enumerated above
- Run periodic consistency sweeps (cross-reference integrity,
  timestamp coherence, tracking file alignment)

**Tone:** Same engineering-tone discipline as Design and
Implementation chats. Skeptical, direct, no superlatives.
But adds: explicit reasoning for every routing decision,
because the human will periodically review.

**Authority limits:**

- Cannot resolve design questions (routes to Design)
- Cannot approve external-facing content (escalates)
- Cannot make priority changes (escalates)
- Cannot merge PRs that fail verification (escalates if
  retry-and-fix doesn't resolve)
- Cannot expand project scope (e.g., adding new targets,
  changing tech stack) without escalation

### Engineering sessions (Claude Code)

Sessions are the workers. They receive specific prompts and
execute mechanically within owned-files constraints.

**Responsibilities:**

- Execute the prompt as written
- Run verification gates before commit
- Surface OPEN/FOUND tags for items beyond session scope
- Stop and escalate when verification fails after reasonable
  retry attempts
- Produce PR with clear commit message documenting work,
  scope expansions, and surfaced items

**Authority limits:**

- Cannot expand owned-files without prompt permission (SCOPE
  EXPANSION pattern)
- Cannot make design decisions encountered mid-work (notes as
  OPEN; orchestrator routes to Design)
- Cannot resolve verification failures by lowering the bar
  (e.g., disabling warnings); escalates to Implementation chat
- Cannot rebase, force-push, or alter history beyond the
  current branch's commits

### Design chat (agentic mode)

Same authority and responsibilities as human-orchestrated
version, with one addition: explicit audit-trail discipline.

**Additional responsibilities:**

- Every spec amendment includes explicit reasoning the
  orchestrator can summarize for the human
- Open questions get formal OPEN: tags with categorization
  (deferrable / blocking / strategic)
- Changelogs cross-reference inputs (handoffs, prior
  changelogs) so the audit trail is complete

### Implementation chat (agentic mode)

Same authority and responsibilities. Additions:

- Every session prompt includes the explicit estimated
  duration and resource cost (so orchestrator can size and
  schedule)
- Every prompt explicitly declares which verification gates
  apply and their reach (so orchestrator can flag if a session
  is about to hit scope-expansion territory)
- Surfaced bugs from prior sessions get triaged: ready-to-run,
  needs-design-input, deferrable

### Marketing chat (agentic mode)

Same authority and responsibilities. Additions:

- Any claim about product capability cites the spec section or
  STATUS.md entry that backs it
- "Roadmap" or "planned" framing for unreleased capability is
  explicit, never elided
- External-facing copy is drafted but never published; final
  publication escalates to human

---

## Sub-agent integration

Engineers and the orchestrator can spawn sub-agents to handle
parallelizable, bounded, independent work and aggregate the
results. Sub-agents are the fan-out layer below engineer
sessions. Used well, they parallelize verification and analysis;
used poorly, they fragment accountability and degrade quality
invisibly.

### The sub-agent contract

A sub-agent:

- Receives a **focused task** with explicit inputs and an
  expected output structure
- Operates **within the parent's scope** — does not expand
  owned-files, touch files outside the parent's declaration, or
  alter shared state
- Reports **structured results** the parent aggregates
  mechanically (a table, matrix, or list)
- Does **not commit independently** — the parent session owns
  the commit; sub-agents produce findings and artifacts the
  parent assembles
- Does **not make design decisions** — design questions surface
  as OPEN to the parent, which routes to Design via the
  orchestrator
- Has **bounded fan-out depth** — sub-agents do not spawn their
  own sub-agents beyond one level, avoiding runaway recursion
  and unaccountable work

### Model and effort floor

Sub-agent model and effort selection defaults to the spawning
context's discretion. This has a failure mode observed across
projects: parents select lower-tier models or reduced effort for
delegated work as an implicit cost optimization. The work
sub-agents do is often the substantive work (verification,
codegen, analysis); degrading it while the coordination layer
stays strong is backwards.

**Rule: no agent anywhere in the delegation tree runs below the
orchestrator's model version or effort level.**

- The orchestrator's model and effort (set by the human) is the
  global floor for the entire tree
- Any sub-agent or engineer session: model version ≥ the
  orchestrator's, effort level ≥ the orchestrator's
- The floor is transitive — a sub-agent spawned by an engineer
  session the orchestrator dispatched is also bound by it
  (orchestrator → engineer → sub-agent; the floor holds at every
  step)
- Delegating to an equal-or-higher configuration is allowed;
  delegating to a lower one is not
- Override framework defaults that select lower models or reduced
  effort for delegated work — this floor takes precedence over
  the default delegation heuristic

The orchestrator does not get to make delegated work cheaper by
making it weaker. If cost genuinely becomes a concern, it
surfaces as an escalation (a resource decision for the human),
not a silent downgrade of sub-agent capability. A weaker
sub-agent producing a result the orchestrator then trusts is
worse than an honest escalation about cost, because the quality
loss is invisible until it lands in the repo.

### When to spawn sub-agents

Spawn when work is:

- **Parallelizable** — independent units that can run
  concurrently (per-unit, per-target, per-section)
- **Bounded** — each unit has clear inputs and a clear done
  condition
- **Aggregatable** — results combine cleanly into the parent's
  deliverable

Do NOT spawn when work is:

- **Sequential** — unit B needs unit A's output (run in the
  parent, in order)
- **Shared-state** — units mutate common state (race conditions;
  keep in the parent)
- **Design-bearing** — the work requires a design decision
  (route to Design, not a sub-agent)

### Generalizable patterns

**Per-unit fan-out.** Work that repeats across a fixed set of
units (build targets, platforms, locales, modules, environments)
parallelizes cleanly. One sub-agent per unit performs the work;
the parent aggregates a per-unit result matrix. The aggregation
must surface per-unit failures explicitly — an aggregate
"N-1 of N passed" easily buries the one failure that matters.

**Multi-axis verification.** When verifying one artifact along
independent axes (e.g., static review vs runtime execution;
correctness vs performance; spec-conformance vs style), each
axis is a sub-agent. The parent merges the axes into one
findings set. Independent axes catch different failure classes;
running them in parallel is both faster and more thorough than a
single combined pass.

**Sweep delegation.** Bounded, read-mostly checks (cross-
reference integrity, dependency audits, dead-code scans) run as
a sub-agent that returns a findings list while the parent
proceeds with other work.

**Blast-radius analysis.** Before a cross-cutting change,
sub-agents inspect different parts of the codebase in parallel
and report what each would need to change. The parent assembles
the impact map before editing anything.

### Sub-agent output contract

Sub-agents return structured results the parent parses:

```
## Sub-agent: <task identifier>
**Status:** complete | blocked | partial
**Inputs processed:** <what was examined>
**Findings:**
<structured result — table, matrix, or list>
**Surfaced items:**
- OPEN: <design questions, if any>
- FOUND: <bugs, if any>
**Citations:** <file:line references>
```

The parent aggregates these into its own deliverable (the PR
description, verification report, or analysis).

### Sub-agents and the worktree pattern

Sub-agents run within an engineer session's worktree. They
share the parent's worktree path and test namespace but must
namespace their own scratch under a per-sub-agent subpath to
avoid collisions. Sub-agents do not create branches, push, or
open PRs — the parent session's teardown handles all of that. A
sub-agent that needs to write a file writes it to the worktree;
the parent includes it in the commit.

---

## Setup checklist

Adapt the human-orchestrated setup with these additions:

### Orchestrator workspace

The orchestrator needs:

- **Work queue document** at `tracking/queue.md` listing
  pending work across all chats, with status and dependencies
- **Audit log document** at `tracking/audit.md` recording
  decisions and actions
- **Escalation register** at `tracking/escalations.md` for
  items awaiting human response
- **Routing rules** at `tracking/routing.md` specifying how
  the orchestrator handles common patterns

The orchestrator instance reads/writes these files; the human
reviews them periodically.

### Engineering session output contract

Sessions produce structured PR descriptions the orchestrator
can parse:

```
## Summary
[1-2 sentences]

## Changes
[What changed]

## Verification
[Each gate; passed/failed]

## Surfaced items
- OPEN: [items needing design routing]
- FOUND: [bugs needing follow-up]
- SCOPE EXPANSION: [if applicable; what expanded and why]

## Tracking file updates needed
- [STATUS.md / ROADMAP.md / etc.]
```

### Periodic human checkpoints

Define explicit moments when the human reviews:

- **Daily** (or per work-block): audit log + escalation
  register; orchestrator status report
- **Weekly**: roadmap progress, scope health, agent
  performance metrics
- **At escalation**: immediate review of the specific
  escalation
- **At external-facing moments**: announcements, releases,
  pivot points

---

## Project instructions templates

Each role gets its own project instructions. The shared
content (authoritative sources, timestamps, handoff patterns,
DO NOT lists) is identical to the human-orchestrated variant.
Below are the role-specific additions.

### Orchestrator project instructions

```markdown
You are the Orchestrator for this project. You fill the
product-owner-and-merge-coordinator role with bounded
autonomy. Strategic direction comes from the human; you
handle routine coordination.

[INCLUDE: all shared content from human-orchestrated
playbook — authoritative sources, timestamps, DO NOT, etc.]

ORCHESTRATOR RESPONSIBILITIES:

- Maintain work queue at tracking/queue.md across all chats
- Route handoffs:
  - OPEN: from Implementation → Design chat
  - FOUND: from Engineering → Implementation triage
  - Spec changes from Design → Marketing (if user-facing)
  - Marketing → Design for accuracy verification
- Schedule engineering sessions per Implementation chat's
  prompts; respect dependency graph
- Coordinate merges for PRs with passing verification gates
- Update tracking files when work lands
- Maintain audit log at tracking/audit.md
- Run consistency sweeps (cross-reference integrity,
  timestamp coherence)

ESCALATE TO HUMAN:

- Strategic priority decisions (which features matter more)
- External-facing content (announcements, releases, copy
  going to audiences)
- Resource/tooling choices (new third-party deps, new
  targets, new languages)
- Conflicts between roles that don't resolve via the
  established handoff patterns
- Session failures after [configured threshold] retries
- Discoveries that expand project scope (new requirements
  surfacing from work, not from strategy)
- Anything genuinely ambiguous about whether you have
  authority

REASONING DISCIPLINE:

Every routing or scheduling decision logs reasoning:
- What input triggered the decision
- What options were considered
- Which option was chosen and why
- What follow-up actions queue from the decision

The human reviews the audit log periodically. If reasoning
is consistently sparse or wrong, that's a signal the
escalation thresholds need adjustment.

ROUTINE WORKFLOWS:

[Define the common routing patterns for your project, e.g.:]

1. New feature flow: Design → Implementation prompt → Session
   → Merge → STATUS update → (Marketing if user-facing)
2. Bug fix flow: FOUND tag → Implementation triage →
   Session → Merge
3. Spec alignment flow: Implementation chat audit → Design
   chat batch resolution → per-batch changelogs land →
   downstream phases unblock
4. Dependency update: Implementation prompt → Session →
   Verification gate → Merge or escalate

AUDIT LOG ENTRY FORMAT:

[YYYY-MM-DD HH:MM UTC] <action>
  Input: <what triggered this>
  Options: <what was considered>
  Decision: <what was chosen>
  Reasoning: <why>
  Follow-up: <next actions queued>

CONSISTENCY SWEEP CHECKLIST:

Run periodically (or on-demand):

- [ ] Every changelog filename reference resolves to an
      existing file
- [ ] Changelog filenames sort consistently with content
      "Date:" headers
- [ ] STATUS.md "Last updated" matches recent commits
- [ ] ROADMAP.md items align with STATUS.md state
- [ ] No tracking file references files that don't exist
- [ ] All escalation register items have human responses or
      are explicitly deferred
```

### Engineer (session) project instructions

```markdown
You are an Engineer running in a Claude Code session. You
execute a specific prompt from the Implementation chat in a
worktree. Your work is bounded by the prompt's owned-files
declaration and produces a single PR.

[INCLUDE: all shared content from human-orchestrated
playbook]

ENGINEER RESPONSIBILITIES:

- Execute the prompt as written; do not expand scope without
  explicit permission
- Read the prompt's Context Loading section in full before
  any edits
- Run verification gates before commit (build, lint, test,
  docs)
- Produce PR with structured description (Summary, Changes,
  Verification, Surfaced items)
- Use FIXED:/FOUND:/SCOPE EXPANSION: tags in commit messages
  per project conventions

DO NOT:

- Expand owned-files without prompt permission
- Make design decisions encountered mid-work (note as OPEN
  in the PR description; Implementation chat triages)
- Resolve verification failures by lowering bars (disabling
  warnings, skipping tests). Escalate instead.
- Force-push or alter history beyond the current branch
- Touch files outside owned-files unless the prompt
  explicitly permits scope expansion

ESCALATE (stop and produce a "blocked" PR description):

- Verification gate fails for reasons the prompt didn't
  anticipate
- Owned-files declaration too narrow for the work as
  described
- Spec ambiguity that affects how to implement (note as OPEN)
- Tooling failures (missing dependencies, environment
  issues)
- Conflict between prompt instructions and observable repo
  state (e.g., prompt says edit X, but X doesn't exist)

PR DESCRIPTION TEMPLATE:

## Summary
[1-2 sentences on what this PR does]

## Changes
[What files changed and what changed in them]

## Verification
- [ ] build: [passed/failed/details]
- [ ] lint: [passed/failed/details]
- [ ] tests: [passed/failed/details]
- [ ] docs: [passed/failed/details]
- [ ] prompt-specific verification: [passed/failed/details]

## Surfaced items
- OPEN: [items needing design routing, if any]
- FOUND: [bugs needing follow-up, if any]
- SCOPE EXPANSION: [if applicable; what expanded and why]

## Tracking file updates needed
- [STATUS.md / ROADMAP.md / CLAUDE.md updates needed
  post-merge, if any]
```

### Design chat (agentic mode) — additional instructions

[Same as human-orchestrated version, plus:]

```markdown
AGENTIC MODE ADDITIONS:

- Every spec amendment includes Rationale and Implications
  sections so the orchestrator can summarize for the human
- Open questions get formal OPEN: tags with categorization:
  - OPEN-D (deferrable; can wait for later resolution)
  - OPEN-B (blocking; gates current work)
  - OPEN-S (strategic; escalates to human via orchestrator)
- Changelogs cross-reference inputs (handoffs, prior
  changelogs, related sessions) so audit trail is complete
- Batched changelogs explicitly named as such in the file
  header ("Batch N of M") for orchestrator-friendly parsing
```

### Implementation chat (agentic mode) — additional instructions

[Same as human-orchestrated version, plus:]

```markdown
AGENTIC MODE ADDITIONS:

- Every session prompt includes:
  - Estimated duration (rough hours)
  - Resource scale (small/medium/large) for orchestrator
    scheduling
  - Verification gate reach declaration: does the gate scan
    files outside owned-files? If so, prompt permits scope
    expansion explicitly
- Surfaced bugs from prior sessions get triaged into one of
  three states:
  - Ready-to-run (clear scope, ready for next session
    scheduling)
  - Needs-design-input (route via orchestrator to Design)
  - Deferrable (logged in tracking/queue.md, not actively
    blocking)
- Prompts that span multiple work items split into
  independent sessions when possible; mega-sessions are
  harder to roll back if verification fails
```

### Marketing chat (agentic mode) — additional instructions

[Same as human-orchestrated version, plus:]

```markdown
AGENTIC MODE ADDITIONS:

- Every product-capability claim cites the spec section or
  STATUS.md entry that backs it
- "Roadmap" or "planned" framing for unreleased capability
  is always explicit:
  - "X works today" only if STATUS.md confirms
  - "X is planned" for roadmap items
  - "We're exploring X" for items not on roadmap
- External-facing copy is drafted but never published
  directly. Final publication queues at the orchestrator's
  escalation register for human approval.
```

---

## Escalation protocols

The orchestrator surfaces items to the human via the
escalation register at `tracking/escalations.md`. Each entry:

```
## [YYYY-MM-DD HH:MM UTC] <short title>

**Type:** [strategic / external / resource / conflict / failure / scope]
**Severity:** [low / medium / high / blocking]
**Trigger:** <what caused this to escalate>
**Context:** <relevant inputs the human needs>
**Options:** <what the orchestrator considered>
**Recommendation:** <orchestrator's informal read>
**Awaiting:** <what response the orchestrator needs>
**Status:** [pending / responded / resolved]
```

### Escalation categories

- **Strategic**: Direction decisions. The orchestrator
  doesn't have the strategic context to choose between
  reasonable options.
- **External**: Anything going to audiences (announcements,
  releases, copy). Always escalates regardless of how routine
  it seems.
- **Resource**: Tooling choices, third-party dependencies,
  target additions. The orchestrator doesn't budget; the
  human does.
- **Conflict**: Roles produce conflicting outputs the
  orchestrator can't reconcile via established handoff
  patterns (e.g., Design says X, Implementation says X is
  impossible).
- **Failure**: Sessions failing after configured retry
  thresholds; consistency sweeps surfacing systemic
  problems.
- **Scope**: Work reveals new requirements that expand the
  project's surface beyond what the human directed.

### Response protocols

The human responds in the escalation register entry:

```
**Response (YYYY-MM-DD HH:MM UTC):**
<decision and reasoning>

**Authorized actions:**
<what the orchestrator may now do>
```

The orchestrator marks the entry resolved and proceeds.

### Bounded retry before escalation

Sessions failing verification get [configured number of]
retry attempts before escalating. Default suggestion: 2
retries with explicit different approaches, then escalate.
Don't retry indefinitely; that hides the failure mode.

Configure retry count per session type:

| Session type | Default retries |
|--------------|-----------------|
| Mechanical (rename, format, lint fix) | 1 |
| Spec-content edit | 2 |
| Feature implementation | 2 |
| Cross-cutting refactor | 1 (escalate fast) |
| Verification/audit | 1 (likely scope problem) |

---

## Audit machinery

### Audit log

Every orchestrator decision and action logs to
`tracking/audit.md`. The format:

```
[YYYY-MM-DD HH:MM UTC] <action>
  Input: <trigger>
  Options: <considered>
  Decision: <chosen>
  Reasoning: <why>
  Follow-up: <queued>
```

The human reviews this periodically. Sparse or wrong
reasoning is a signal to adjust escalation thresholds.

### Periodic review cadence

Suggested:

- **End-of-block review**: After each work block (~2-4 hour
  agent session), human reviews audit log highlights
- **Weekly review**: Full audit log walk, roadmap progress
  check, escalation register cleanup
- **Pre-release review**: Comprehensive consistency sweep,
  external-facing content review, scope health check

### Health metrics the orchestrator tracks

- Session pass rate (verification gates / total attempts)
- Average sessions before escalation
- Open OPEN-D / OPEN-B / OPEN-S counts
- Time-since-last-escalation per category
- Cross-reference integrity (broken refs found in sweeps)
- Tracking file alignment (STATUS/ROADMAP/CLAUDE.md drift)

Metrics inform the human's review and the periodic
adjustment of escalation thresholds.

---

## Failure modes and protocols

### Orchestrator makes bad routing decision

**Symptom:** Work routed to wrong chat; design decision routed
to implementation; spec ambiguity routed to marketing.

**Detection:** The receiving chat surfaces "this isn't my
domain" with a redirect suggestion.

**Recovery:** Orchestrator re-routes with explicit log entry:
"Re-routing: original routing was wrong because [reason].
Lesson: [pattern to remember]."

**Prevention:** Periodic human review of audit log catches
patterns of bad routing; adjust routing rules.

### Chat drifts from project context

**Symptom:** Chat produces outputs referencing fictional
paths, deprecated names, or pre-decision framing.

**Detection:** Other chats or sessions encounter the
inconsistency; orchestrator's consistency sweep catches it.

**Recovery:** Orchestrator escalates as "drift detected" with
context about what drifted. Human decides whether to refresh
the chat's project knowledge or treat the chat output as
scratch.

### Session fails verification repeatedly

**Symptom:** Two or more retries fail.

**Recovery:** Orchestrator escalates with the session prompt,
failure modes, and what was tried. Human decides: re-scope,
abandon, or accept partial work.

### Multiple chats produce conflicting outputs

**Symptom:** Design and Implementation disagree about whether
something can be implemented; Marketing and Design disagree
about how to describe a feature.

**Detection:** Orchestrator notices conflicting handoffs.

**Recovery:** Orchestrator routes to the authoritative chat
per the authority matrix. If genuine conflict (neither chat
has authority), escalate to human.

### Tracking file drift

**Symptom:** STATUS.md says X is implemented; impl says it
isn't. ROADMAP.md says Y is planned for v1; STATUS.md ships Y.

**Detection:** Consistency sweep.

**Recovery:** Orchestrator routes to whichever chat owns the
tracking file domain (typically Implementation for STATUS,
Design for ROADMAP) with the specific drift surfaced.

---

## Bootstrap sequence

To start a new project with this playbook:

1. **Human work (initial):**
   - Create repository skeleton
   - Write initial spec (first version)
   - Set up Claude Code with `/project:session` pattern
   - Configure tracking/ directory with queue, audit,
     escalations, routing files
   - Set escalation thresholds and retry counts
   - Set up four chat instances with role-specific
     project instructions: Design, Implementation, Marketing,
     Orchestrator

2. **First orchestrator run:**
   - Orchestrator reads project state, populates initial
     queue, identifies first priorities
   - Escalates: "ready to begin. First work blocks suggested:
     A, B, C. Confirm priorities."

3. **Human response:**
   - Confirm priorities or adjust
   - Authorize first work block

4. **First work block:**
   - Orchestrator routes per priorities
   - Design produces first spec amendments (if needed)
   - Implementation produces first session prompts
   - Engineer (CC) runs first sessions
   - Orchestrator coordinates merges, updates tracking,
     audits each decision
   - Escalates anything outside authority

5. **First review:**
   - Human reviews audit log
   - Checks escalation handling quality
   - Adjusts thresholds or routing rules if needed

6. **Iterate:**
   - Subsequent blocks run with progressively less human
     touch as the orchestrator's pattern matching improves

---

## What to customize per project

In addition to the human-orchestrated customization knobs:

- **Escalation thresholds** — retry counts, escalation
  categories sensitive to your tolerance for autonomy
- **Audit verbosity** — how much reasoning per decision is
  worth the token cost
- **Review cadence** — daily / per-block / weekly depending
  on project pace and stakes
- **Authority matrix** — adjust who decides what based on
  team / project structure (e.g., maybe Marketing has more
  autonomy in some projects; less in others)
- **Tracking file structure** — adapt the queue / audit /
  escalation register format to your tools
- **Health metrics** — which signals matter for your project
  (e.g., a research project might care about novelty;
  a maintenance project about regression rate)

---

## Anti-patterns specific to agentic mode

In addition to the human-orchestrated anti-patterns:

- **Orchestrator making strategic decisions.** It doesn't
  have the context. Always escalate genuine strategic calls.
- **Routing without reasoning logged.** Sparse audit means
  the human can't review effectively. Reasoning per decision
  is non-negotiable.
- **Indefinite retry on failing sessions.** Hides the
  failure mode. Bounded retries with mandatory escalation.
- **Marketing publishing without escalation.** External-
  facing content always goes through the human, even when
  routine.
- **Cross-role audits skipped.** Inter-role verification is
  the agentic system's substitute for the human's continuous
  oversight. Skipping it removes the safety net.
- **Trusting role outputs without verification.** Design says
  X; verify against spec before Implementation acts on X.
  Implementation says Y; verify against session results
  before Marketing claims Y.
- **Letting the orchestrator's audit log grow unread.**
  Reading the log is how the human stays in the loop with
  minimal touch. Skipping it removes the entire premise.
- **Downgrading sub-agent model or effort to save cost.** The
  default delegation heuristic drifts toward cheaper models for
  delegated work, which is exactly the substantive work
  (verification, codegen, analysis). No agent in the tree runs
  below the orchestrator's model and effort. Cost concerns
  escalate as resource decisions; they are never solved by
  silently weakening sub-agents, because the quality loss is
  invisible until it lands in the repo.
- **Burying per-unit failures in fan-out aggregates.** When N
  sub-agents verify N units, an aggregate "N-1 passed" hides the
  one failure. The parent surfaces each per-unit failure
  explicitly via FOUND tags; it does not average them away.

---

## Closing notes

The agentic variant trades the human's continuous attention
for the orchestrator's bounded autonomy plus audit machinery.
The trade only pays off if:

1. The orchestrator's authority is genuinely bounded (it
   escalates the right things)
2. The audit log is readable and reviewed (reasoning per
   decision, periodic human review)
3. The inter-role verification catches the failures the
   human would have caught
4. The escalation thresholds are calibrated for your
   tolerance

If any of these breaks down, the agentic system degrades
faster than a human-orchestrated one because there's nobody
catching errors mid-stream. The protocols above are designed
to catch the common failure modes before they compound, but
they require ongoing calibration.

Start with conservative thresholds (more escalation, more
human review) and loosen as the patterns prove themselves.
The cost of an over-escalating orchestrator is the human's
time; the cost of an under-escalating one is uncaught errors
landing in main.
