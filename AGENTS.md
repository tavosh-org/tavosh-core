# Public Tavosh core

Tavosh (Tavo) is a GPU-accelerated, mobile-first SSH terminal and remote AI
agent cockpit. Preserve the power of a real terminal and support touch-first
workflows. Personal daily utility for developers with remote machines and
long-lived CLI agents is the v0 bar.

## Repository boundary

This public, platform-independent Rust workspace is MIT OR Apache-2.0.
Dependencies flow from the proprietary app to core. Keep mobile UI, client
SSH policy, signing and credentials in the app.

- `tavosh-proto` owns shared serde schemas, framing and state types. Document
  versioning and compatibility before introducing wire changes.
- `tavosh-parser` owns OSC 133 slicing, quiescence fallback and structured data
  detection. Preserve byte order and keep raw PTY bytes on the native side.
- `tavosh-daemon` owns optional authenticated replay and agent IPC. Target one
  static Rust binary per supported host architecture and reuse core schemas
  and parsers. Standard SSH and tmux must work without the daemon.
- `fixtures/` holds sanitized terminal captures and synthetic regression traces.

v0 excludes Mosh SSP, arbitrary agent TUI scraping and exhaustive SSH or agent
coverage. The public core supplies the contracts for the app's command feed,
TUI viewport, structured output, agent cockpit, roaming and host companion.

## Engineering and documentation

Pin Rust in `mise.toml`, justify dependency additions, and check in Cargo.lock
updates. Run `cargo fmt --all -- --check` before committing. Use automated VT100
conformance cases, recorded trace replay, and controlled-clock tests for
arbitrary read boundaries, malformed frames and quiescence. Preserve the
smoke grid as the M0 contract until versioned replacements exist.

Update `docs/` and `README.md` whenever architecture, dependencies, or milestone
status changes. Update fixtures when terminal or protocol behavior changes.

## Git Workflow

- Keep `main` clean and buildable. Use short-lived descriptive branches
  (`feat/<name>`, `fix/<name>`, `chore/<name>`, `docs/<name>`) for non-trivial
  work. Direct commits to `main` are allowed for docs-only changes and
  early-milestone cleanup that leaves the repository buildable.
- Use Conventional Commits in the form `type(scope): imperative summary`.
  Make each commit atomic and single-purpose. Keep unrelated formatting
  changes in a separate commit from logic changes.
- Verify the destination before pushing. Each checkout uses its own GitHub
  remote, `tavosh-org/tavosh-core` or `tavosh-org/tavosh-app`, respectively.
  Never publish private app commits to the public core remote.
- Before pushing, run `cargo check --locked --workspace` and
  `cargo test --locked --workspace` in that checkout and pass all relevant
  checks. Never force-push to `main`. An agent must obtain the user's explicit
  approval for the repository and branch before running `git push`.
