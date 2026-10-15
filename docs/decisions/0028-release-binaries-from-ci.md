# 0028: Release binaries are built by GitHub Actions on a version tag

Date: 2026-10-15T15:05Z
Status: accepted
Proposed by: human (Andrew), so heeded installs where there is no Rust toolchain
Ratified by: claude-opus-5-5

## Context

Since 0024 heeded is one Rust binary, installed with `cargo
install`. The work machine, and any CI runner that will run
the Layer 3 checks, may have no Rust toolchain. Andrew proposed
building binaries in GitHub Actions and publishing them as
releases, at least for manual download, and asked for macOS too.
The repo had no CI at all, and 0024's claim that heeded runs the
same on Windows and Linux had only been tested on Windows.

## Assessment

Right, and cheap. Releases also give Layer 3 a binary to fetch
on a runner. Running the check on every push tests the
cross-platform claim for the first time. `heeded` had no way to
say which build it is, which a downloaded binary needs, and which
is the start of noticing a stale binary on the `PATH`.

## Decision

- `.github/workflows/check.yml`: `npm run check` on Windows,
  Linux and macOS, on every push to master and every pull
  request.
- `.github/workflows/release.yml`, on a pushed `v*` tag: refuse
  unless the tag equals `Cargo.toml`'s version and the changelog
  has that version's section; open a draft release with that
  section as notes; build Windows x64 (C runtime linked in, so no
  Visual C++ redistributable), Linux x64 on musl (static), macOS
  arm64 and x64; upload each; add `SHA256SUMS`; publish.
- Only `actions/checkout` is used, pinned to a commit. Everything
  else is `gh`, `rustup` and `cargo`, already on the runners.
- `heeded version` and `heeded --version` print `heeded X.Y.Z`.

## Consequences

- A release is a tag push, so it needs Andrew's yes like any
  push; the guard asks. The version bump stays its own commit and
  the tag goes on it.
- The binaries are unsigned. Windows SmartScreen and macOS
  Gatekeeper may warn; a managed work machine may refuse an
  unsigned download. Signing costs money and setup, and waits for
  evidence that it is needed.
- Neither workflow has run yet. The first push will show whether
  the test suite passes on Linux and macOS; it has never run
  there.
- The plugin still does not check that the binary's version
  matches its own. `--version` makes that possible.
