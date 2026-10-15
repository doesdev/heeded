# 0025: Git stays a child process; gix is not worth it here

Date: 2026-10-15T05:45Z
Status: accepted
Proposed by: human (Andrew), passing on a task written elsewhere
Ratified by: claude-opus-5-5

## Context

A task handed in by Andrew proposed cutting the hooks' git
latency by replacing the `git` CLI with in-process gitoxide
(`gix`), measuring first. He said it need not be chased but
was worth measuring. It assumed the hooks run on prompt
completion; they run before and after every tool call, so the
per-call cost matters more than it assumed.

Measured on this machine on 2026-10-15, `GIT_OPTIONAL_LOCKS=0`,
median of 21 (p95 within a few ms) in this repo, gazelle-audio
(703 files), wcmd-main (1,288) and a synthetic repo of 30,000:

- Starting git at all (`git --version`): 19 to 23 ms.
- `rev-parse` and `config`: 21 to 24 ms, so all spawn.
- `status --porcelain=v2 --branch -uall`, the hook's one
  expensive call: 29 to 35 ms in real repos, 47 ms at 30,000
  files. Its own work is 7 to 25 ms over the spawn.

So the cost is starting git, not git's work.

A throwaway gix 0.89 prototype in scratch (open, HEAD, status
with untracked files, the `status`, `sha1` and `parallel`
features) found the same changed files as git in all four
repos. As one whole process, against the two git calls it
would replace (49 to 63 ms):

- Opened normally, reading system and global config: 86 to
  169 ms. Opening alone took 35 to 41 ms, and status was
  slower than git's, likely because Git for Windows has
  `core.fscache` on and gix has nothing like it.
- Opened without running `git` to find the system config: no
  better, 90 to 130 ms.
- Opened isolated, with no system or global config: 16 to 31
  ms in real repos, 64 ms at 30,000 files. Fast, but it drops
  the global config, whose settings (line endings, a global
  ignore file) can change what counts as a change.

## Assessment

The task's measure-first order was right, and the measurement
says no. The only fast gix setup ignores config that decides
what status reports, so it could disagree with git silently,
which the task itself ruled out. It would also double the
binary and add 130 crates. The gain, 20 to 30 ms a stamped
call on normal repos, does not pay for that.

## Decision

- heeded keeps running `git` as a child process.
- Every git call heeded makes sets `GIT_OPTIONAL_LOCKS=0`, so
  a status refresh never writes the index. Found while doing
  the inventory: without it, heeded's status could take
  `index.lock` while the agent's own git command needed it.
- The cheap wins already taken stand: one status call reads
  HEAD too, no git before ordinary commands, no git at all in
  a repo that never opted in.

## Consequences

- A stamped repo pays about two git starts after each tool
  call, roughly 45 ms of the hook's 68.
- Worth revisiting if gix gains something like `fscache`, or
  if a repo's own work starts to dominate the spawn (at
  30,000 files status work was 25 ms over it).
- Not built, and only worth proposing with evidence: caching
  HEAD and the index's modified time to skip status when
  nothing changed, and a long-lived process. The first would
  save one git start on calls that changed nothing; the second
  trades every git start for a daemon to keep alive.
