# Multi-Chat Project Playbook — Human-Orchestrated

A starter playbook for running a software project with Claude
using multiple chat instances coordinated by a human product
owner. Distills the patterns that proved durable across the
Bock project iteration: spec-as-source-of-truth, role-separated
chats, worktree-based engineering sessions, verification-gated
handoffs.

---

## What this gives you

A working multi-chat workflow with:

- **Three chat roles** (Design, Implementation, Marketing) with
  distinct responsibilities, tones, and authority
- **Claude Code sessions** for actual engineering work, routed
  by Implementation chat
- **Spec-as-truth** with changelogs recording every design
  decision
- **Cross-chat handoff conventions** that prevent context drift
  between chat instances
- **Process safeguards** against the failure modes that bite
  every multi-chat project: timestamp drift, dual-source
  divergence, scope creep, factually-incorrect committed
  records

You play product owner: strategic direction, routing between
chats, merge coordination, escalation handling.

---

## Conceptual framework

### Authoritative source hierarchy

```
Repository (canonical)
  │
  ├─ Specification document (single source of truth for design)
  ├─ Status document (current implementation state)
  ├─ Roadmap document (forward plans)
  ├─ Changelogs/ (dated decision history)
  └─ Per-subtree CLAUDE.md (contribution conventions)
       │
       ▼
Project knowledge sync (per-chat surface of repo)
  │
  ▼
Chat outputs (working drafts — scratch until landed in repo)
```

**Repo wins.** When chat output disagrees with repo state, the
repo is canonical. Chat outputs (including outputs of the
chat producing them right now) are scratch until they hit a
commit.

### Three chat roles

| Role | Authority | Tone | Output |
|------|-----------|------|--------|
| **Design** | Spec amendments, design decisions | Engineering: skeptical, direct, pushback welcome | Spec edits + changelogs |
| **Implementation** | Session prompts, work scheduling, troubleshooting | Engineering: skeptical, direct, pushback welcome | Session prompts + handoff documents |
| **Marketing** | Positioning, voice, copy for audience surfaces | Persuasive within accuracy constraints | Copy + announcement drafts |
| **You** (product owner) | Strategy, priorities, merging, escalation | Whatever works | Decisions + direction |

**Never resolve a design question outside design chat.** This
keeps the spec as the genuine single source of truth.
Implementation chats noting an OPEN question and routing it to
design is the only acceptable path.

### Engineering sessions (Claude Code)

Implementation chat designs session prompts; you run them via
Claude Code. Sessions execute in isolated worktrees with
verification gates before commit/push.

Pattern:
```
/project:session <branch-name> [owned-files...]
```

The session:
1. Creates a worktree at a stable path
2. Sets `$WORKTREE` and `$BOCK_TEST_NAMESPACE` environment
3. Operates within owned-files declaration
4. Runs verification gate before push (build, lint, test, docs)
5. Commits, pushes, opens PR
6. You review and merge

---

## Setup checklist

### Repository skeleton

Create the canonical structure:

```
{{project-root}}/
├─ {{spec-file}}.md          ← single source of truth for design
├─ STATUS.md                  ← current state
├─ ROADMAP.md                 ← forward plans
├─ CLAUDE.md                  ← root contribution conventions
├─ ARCHITECTURE.md            ← system overview
├─ changelogs/                ← dated decision history
│   └─ <YYYYMMDD-HHMM>-<type>-changes.md
├─ {{source-tree}}/           ← actual code
│   └─ CLAUDE.md              ← subtree conventions
├─ docs/                      ← user-facing documentation
│   └─ CLAUDE.md
└─ tools/
    └─ scripts/               ← shared tooling
```

Adjust paths to your project's conventions.

### Claude Code slash command

Define `/project:session` at `.claude/commands/session.md` (or
wherever your CC setup expects). It should:

- Accept a branch name and optional owned-files list
- Create a worktree under a stable directory
- Set environment variables for the session
- Run the prompt against the worktree
- On success: run verification, commit, push, open PR
- On failure: preserve worktree for inspection

The exact implementation depends on your CC setup. The pattern
matters more than the specific shell.

### Project instructions for each chat

Set up project instructions (or system prompt material) that
each chat instance reads. The next section is a template.

---

## Project instructions template

Copy this into each chat's project knowledge. Customize the
`{{placeholders}}`.

```markdown
This project is {{project-description}}.

NAMING:
{{naming-conventions-specific-to-project}}
- Historical records preserve former names as accurate at
  their date; do not retroactively rename them.

REPOSITORY:
- {{repo-url}}
- {{source-tree-summary}}
- See ARCHITECTURE.md for system overview

AUTHORITATIVE SOURCES:

The repository — accessible through this project's knowledge
sync — is the canonical source for every question about
current state. Specifically:

- {{spec-file}} — single source of truth for design
- STATUS.md — current implementation state
- ROADMAP.md — forward-looking plans
- changelogs/ — dated decision history
- CLAUDE.md (root and per-subtree) — session and contribution
  conventions

Project knowledge sync is maintained manually and may lag the
repo. If a chat suspects the synced state is stale relative
to recent activity, it should ask rather than assume the
synced state is current.

Chat outputs are working drafts, not persistent state. This
includes files this very chat produces in
/mnt/user-data/outputs/ or equivalent ephemeral directories.
They are useful within the session that produced them and for
immediate handoff to other chats, but the moment work is
meant to persist, it must land in the repo through a commit.
If a chat output disagrees with the repo, the repo wins.

When referencing prior work, prefer:
- Specification section numbers over filenames
- Content descriptors over date-based references
- Repo state over chat memory

Filename references between artifacts are sometimes
unavoidable. When used, pair them with content descriptors so
the reference survives any commit-time rename.

ROLES OF DIFFERENT CHATS:

- "Design" chats: Discuss design, amend the spec, resolve open
  questions. Reference {{spec-file}} as the source of truth.
  Any design changes must be reflected back into the spec with
  a corresponding changelog.

- "Implementation" chats: Plan and orchestrate engineering
  sessions, design session prompts, troubleshoot session
  results, evaluate parallel execution opportunities.
  Reference STATUS.md and ROADMAP.md for current state and
  forward plans.

- "Marketing" chats: Develop positioning, messaging, audience
  framing, announcement copy. Reference the spec for what's
  true; the goal is communication and audience fit, not
  technical accuracy alone. Marketing chats may simplify or
  stylize for impact — but must not:
  - Invent capabilities the product does not have
  - Promise specific future features without design chat
    confirmation
  - Make competitive claims without verification

CROSS-CHAT HANDOFF:

Work flows between chat types in predictable patterns:

- When an implementation chat surfaces a question the spec
  doesn't answer, it notes the question with "OPEN: ..." and
  brings it to a design chat. The design chat publishes a
  changelog; the implementation chat picks up the resolved
  version.

- When a design chat makes a decision that affects in-flight
  implementation work, the changelog should call out which
  packages or sessions are affected.

- When a marketing chat needs technical accuracy verification,
  it cites the relevant spec section. If unclear or
  overstated, the question goes to a design chat before the
  marketing claim is finalized.

- Never resolve a design question inside an implementation or
  marketing chat. Note as OPEN and route to design.

Handoff documents have two states:

- **Ephemeral handoffs** drafted in chat, used once, discarded.
  Treat as any other chat output: scratch.
- **Persistent handoffs** committed to changelogs/ for future
  reference. Follow the same authority rules as the changelogs
  they accompany.

When a handoff is acted on, the acting chat verifies against
current repo state rather than assuming prior chat output —
ephemeral or persistent — is still current.

TIMESTAMPS:

Every chat that produces dated artifacts anchors timestamps
against an external clock, not against inferred draft
context. Clock drift in inference has caused changelog
filenames to be days or weeks off from actual commit dates.

Resolution order:

1. If shell access is available:
   ```
   date -u +"%Y%m%d-%H%M"
   ```
   UTC is the project standard — eliminates DST ambiguity and
   produces filenames that sort consistently.

2. If shell access is not available, use the current date
   from chat system context.

3. Web search ("current date today UTC") is the fallback.

When multiple changelogs land as a coordinated batch, they
should share a single timestamp or use distinct timestamps
with explicit batch cross-references.

The standard format for changelog filenames is
`YYYYMMDD-HHMM-<type>-changes.md` where `<type>` is the work
type (e.g., specs, impl, verification).

RULES:

1. {{spec-file}} is the single source of truth for design.
   When the spec is silent on something, note it as a
   clarification needed, not as license to invent.

2. STATUS.md and ROADMAP.md are the source of truth for
   implementation state and direction.

3. When amending the spec, always create a changelog file
   documenting what changed and why. Routine bug fixes and
   feature additions don't need changelogs.

4. Track open decisions. If raised but not resolved, note as
   "OPEN: ..." so it can be addressed before the relevant
   work phase.

5. Keep responses focused. Reference relevant spec sections
   or documents; avoid re-explaining architecture.

6. Engineering sessions use the worktree-based session
   pattern. When emitting session prompts:
   - Don't `cd` into the project root — the slash command
     places the session at the worktree
   - Don't manually create branches or push — the slash
     command does this
   - Use the test namespace variable for scratch paths to
     avoid collisions across concurrent sessions
   - Use FIXED:/FOUND: tags in commit messages to surface
     bugs for the merge coordinator

7. Tracking file alignment: When proposing changes, check
   that STATUS.md, ROADMAP.md, and CLAUDE.md remain
   consistent.

8. Tone (engineering chats): Honest skepticism is more
   valuable than enthusiastic agreement. Push back when an
   idea has problems. Direct corrections are welcome. Bugs
   found are information, not setbacks. Avoid superlatives.

   Tone (marketing chats): Persuasive language is appropriate.
   The constraint is accuracy — claims must be true, framing
   is deliberately different from engineering chats.

DO NOT:

- Web-search for project-internal questions. The spec,
  changelogs, and project knowledge are authoritative.
  Web search is appropriate for competitive verification,
  trademark/SEO, audience research, external landscape.

- Improvise spec answers when the spec is silent. Note as OPEN.

- Resolve design decisions outside design chats.

- Modify historical changelogs to reflect newer naming or
  decisions. They are records of what was true at their date.
  Exception: a changelog committed with factually incorrect
  content (claims work was done that wasn't, references paths
  that don't exist) is not a "historical record" and may be
  corrected — surface this case to the user before acting.

- Anchor timestamps against draft-context inference.

- Reference prior chat outputs as authoritative when the repo
  has a current version. Applies to outputs of the current
  chat too.

- Cite changelog files by filename alone when a content
  descriptor or section number would survive any commit-time
  rename.
```

---

## Common workflows

### Starting a new feature

1. **Design chat:** Discuss the feature; produce a changelog
   amending the spec. Note any OPEN sub-questions.
2. **You:** Commit the changelog and spec changes.
3. **Implementation chat:** Read the changelog; produce a
   session prompt for the work, sized to a single worktree
   session with verification gates.
4. **Claude Code session:** Run the prompt; produce PR.
5. **You:** Review and merge.
6. **Marketing chat** (if user-facing): Update audience copy
   to reflect the new capability, citing the spec section.

### Resolving an OPEN question

1. **Implementation chat** flags the OPEN.
2. **You:** Route to design chat with the implementation chat's
   context.
3. **Design chat:** Resolve, produce a changelog.
4. **You:** Commit changelog; update implementation chat with
   the resolution.
5. **Implementation chat** continues from the resolved version.

### Cross-cutting refactor

1. **Design chat** (if it touches design): produces a changelog.
2. **Implementation chat:** scopes the work, identifies
   affected files; produces session prompt with owned-files
   covering the verification gate's reach.
3. **Claude Code session:** runs with SCOPE EXPANSION
   permission for the verification gate's repo-wide scan.
4. **You:** review the PR carefully; the SCOPE EXPANSION note
   in the commit message documents what expanded and why.

### Spec alignment audit

When you suspect drift between spec and implementation:

1. **Implementation chat:** Walks both axes — normative review
   (read spec, compare to impl) AND example execution (run
   every spec example through tools).
2. Produces a row-per-divergence matrix.
3. Implementation chat **batches by topic** before routing to
   design chat; consolidates 20+ rows into ~8-10 decisions.
4. **Design chat:** Resolves the batches; produces per-batch
   changelogs.
5. **You:** Commit changelogs; coordinate any follow-on impl
   work surfaced.

---

## Conventions that matter

### Session prompt structure

```
/project:session <branch-name> [owned-files...]

## Mission
[Brief: what the session accomplishes]

## Context Loading
[Files to read; verification commands to run]

## Implementation
[Numbered, mechanical steps]

## Test
[Verification commands the session runs before commit]

## Completion
[git add/commit/push pattern]

## Acceptance Criteria
[Checkboxes the merge coordinator confirms]

## Notes
[Caveats, scope expansions, anticipated surprises]
```

### Owned-files discipline

The owned-files declaration MUST cover everything the
verification gate scans. If the gate runs `grep -r 'X' .`,
owned-files must include every file that `grep -r 'X' .`
might find. Two paths to handle this:

1. **Predict and include.** Run the grep at prompt-design
   time and include every reachable file.
2. **Permit scope expansion.** Allow the session to expand
   owned-files mid-run with a SCOPE EXPANSION note in the
   commit message.

The second is appropriate for rename/deletion sweeps where
the reachable set isn't fully knowable.

### Verification by example execution

Spec-alignment work needs both axes:

- **Normative walk:** read sections, compare to impl
- **Example execution:** run every example through the tool

The two catch different failures. Normative walks catch
architectural-level claims that don't match impl. Example
execution catches feature-level gaps embedded in
illustrative code that wouldn't appear as section-level
assertions.

Adopt a convention like `<!-- verify: <tool> -->` in doc
source so verification can be automated per-block.

### Batched changelogs

When multiple changelogs land as a coordinated batch:

- Share a single timestamp resolved at the batch's draft
  moment, OR
- Use distinct timestamps with explicit batch cross-references
  ("Batch 1 of four, paired with ...")

Mixing distinct timestamps within a batch without explicit
cross-refs makes the batch hard to identify later.

### Filename + content descriptor pairing

When a changelog references another changelog by filename,
include a content descriptor too:

> See the K04 consolidation decision
> (`changelogs/<filename>`).

The content descriptor survives renames; the filename is just
a current locator.

---

## Failure modes and recovery

### Chat operating against fictional workspace

**Symptom:** A chat produces outputs referencing paths or files
that don't exist in the actual repo.

**Cause:** Chat's mental model diverged from repo state because
project knowledge sync wasn't current.

**Recovery:**
1. Update project knowledge sync (or upload current repo files
   to chat).
2. Surface the divergence; have the chat reconcile against
   actual state.
3. Treat the previous chat output as scratch; produce new
   output against verified state.

### Timestamp drift in chat-produced filenames

**Symptom:** Changelog filenames don't match the actual landing
dates. References between changelogs go stale after manual
renames.

**Cause:** Chats infer timestamps from their context rather than
querying the system clock.

**Recovery:**
1. Add `date -u` discipline to project instructions (already
   in the template above).
2. Sessions that produce dated artifacts capture timestamps at
   execution time.
3. Periodically run a cross-reference sweep: grep for filename
   references; verify each points at an existing file.

### Committed changelog with factually incorrect content

**Symptom:** A changelog in the repo describes work that wasn't
done, references paths that don't exist, or claims behavior
that doesn't work.

**Recovery (with the user's surfacing):**
1. Verify the inaccuracy via project knowledge / direct
   inspection.
2. Decide: replace in place, delete and replace, annotate, or
   leave with adjacent corrective record.
3. Choice depends on severity and whether an adjacent record
   already corrects it.

### Verification gate fails on files outside owned-files

**Symptom:** A session's verification gate finds violations in
files not declared as owned.

**Recovery options:**
1. **Expand scope** (recommended for solo sessions on
   long-lived branches): Edit the out-of-scope files, document
   the expansion in commit message under SCOPE EXPANSION.
2. **Pause and re-scope:** Stop the session; re-issue the
   prompt with expanded owned-files.
3. **Skip K-specific verification:** Commit the in-scope work;
   leave a follow-up task.

Future sessions should predict the verification gate's reach
and either include the full set or permit expansion in the
prompt.

---

## What to customize per project

The template above has these adjustable knobs:

- **Spec file path/name** — `spec/myproject-spec.md` or
  `SPEC.md` or wherever your spec lives
- **Source tree structure** — language-specific (Cargo workspace,
  Maven multi-module, etc.)
- **Slash command name** — `/project:session` is the Bock
  pattern; rename for your CC setup
- **Test namespace variable** — `$BOCK_TEST_NAMESPACE` should
  be `$<PROJECT>_TEST_NAMESPACE` or just `$TEST_NAMESPACE`
- **Verification gate commands** — what `cargo build` /
  `cargo clippy` / `cargo test` become for your stack
- **Documentation tool** — mdBook, Hugo, MkDocs, etc.
- **Project-specific rules** — domain-specific conventions
  (e.g., for a language project: tokenization rules; for a
  webapp: API versioning rules; for a library: backward
  compatibility rules)

Strip rules you don't need; add ones specific to your domain.

---

## Anti-patterns to avoid

- **Trusting chat-context dates.** Always `date -u`. Always.
- **Letting chat outputs drift from repo.** Chat outputs are
  scratch. Period.
- **Routing design decisions through implementation chats.**
  Never. Always note as OPEN; surface to design.
- **Dual-source spec content.** If you have a canonical spec
  AND section excerpts, the excerpts drift. Pick one source.
- **Filename references without content descriptors.** Renames
  break refs; descriptors survive.
- **Verification gates with reach exceeding owned-files.**
  Either include the full reachable set or permit scope
  expansion.
- **Mega-changelogs covering multiple decisions.** Per-batch
  changelogs are independently revertable; mega-changelogs
  bury individual decisions.
- **Marketing claims unbacked by spec.** Every marketing claim
  should cite a spec section. Unverified claims get routed to
  design first.
- **Skipping verification by example execution.** Normative
  review misses feature-level gaps in illustrative code.

---

## Starter session for a new project

To bootstrap a new project with this playbook:

1. Create the repository skeleton.
2. Write the first version of the spec.
3. Set up Claude Code with the `/project:session` pattern.
4. Set up project instructions (using the template above) in
   three chat instances: Design, Implementation, Marketing.
5. Run a small first session to validate the workflow before
   investing in larger work.
6. Iterate the conventions to fit your project's needs.

The playbook is a starting point. The conventions that matter
most are repo-as-truth, design-only-in-design-chat, and
verification-gated commits. Everything else can adapt.
