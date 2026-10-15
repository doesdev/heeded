# 0024: heeded is one Rust binary

Date: 2026-10-15T05:01Z
Status: accepted
Proposed by: human (Andrew), on the model's measurements
Ratified by: claude-opus-5-5

## Context

0002 made the hooks Node scripts so they would run the same on
Windows and Linux without bash. Every tool call now runs the
hook twice, before and after. Measured on this machine on
2026-10-15, median of 15:

- `node -e 0` starts in 39 ms; an empty ES module reading
  stdin in 47 ms.
- The hook in a repo that never opted in: 46 ms, after 0023
  cut it down to Node's own start.
- The hook in a stamped repo: 171 ms, of which about 70 ms is
  three git calls.
- A Rust release binary that reads stdin and checks for the
  marker: 8 ms.

Andrew asked whether Node is the right tool, given that every
millisecond here is paid on every tool call, and said he would
as soon use Rust, which his other agent-driven projects use.

## Assessment

Ratified. The language of a tool he will run is his call, and
the numbers support it: the floor drops from about 46 ms to
about 8 ms a call, paid twice a call, in every repo on the
machine. 0002's reason, one behaviour on every platform with
no bash, holds better for a single static binary than for
Node, which also brought fnm and per-version `npm link` into
it.

What I am keeping from the Node version:

- The behaviour, exactly. The Node test suite was run against
  the Rust binary before the Node code was removed, so the
  port is judged by the same tests, not by reading.
- The shapes on disk: marker, notes, journal lines, settings,
  session state. A stamped repo needs nothing done to it.
- Git as a child process. An in-process git library might cut
  more, but it is a large dependency for an unmeasured gain.

## Decision

- One binary, `heeded`, is the CLI and the hook (`heeded hook`
  reads the hook's input on stdin). Both harnesses' manifests
  and the skills call it by name, from the `PATH`.
- It is installed with `cargo install --path .`. Built output
  is never committed.
- Dependencies: `serde` and `serde_json` for JSON, `regex` for
  the command and secret patterns. Hashing, time formatting
  and argument parsing are written here.
- Tests are `cargo test`. This repo's `check` stays
  `npm run check`, a one-line script that runs `cargo fmt`,
  `cargo clippy` and `cargo test`, so the name does not
  change (0003).

## Consequences

- Measured after the port on this machine, median of 15 per
  hook call: 11 to 12 ms in a repo that never opted in (Node:
  46); 11 ms before an ordinary command in a stamped repo, now
  with no git at all; 36 ms before a push; 68 ms after a tool
  call in a stamped repo (Node: 171), which is now two git
  calls instead of three. The tree fingerprint reads HEAD from
  `git status --porcelain=v2 --branch`.
- The Node test suite passed against the binary (75 tests)
  before it was retired; `cargo test` then carries 81.
- Supersedes 0002 and 0005.
- A machine needs the binary on its `PATH`, built for it. For
  now that means a Rust toolchain on each machine, the work
  machine included. Prebuilt binaries per platform come with
  any public release.
- The plugin and the binary can drift: the plugin's hook
  manifest calls whatever `heeded` is on the `PATH`. A version
  mismatch is not detected yet.
- Tree fingerprints change hash function, so a session that
  spans the switch sees one spurious change.
- The voice directive and his voice skill are about
  JavaScript; the Rust here follows Rust's own idiom.
