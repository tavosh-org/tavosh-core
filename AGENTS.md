# Public Tavosh core

Tavosh is a touch-first SSH terminal and CLI coding-agent cockpit. This
repository owns the public, platform-independent Rust layer under MIT OR
Apache-2.0. Keep mobile UI, client SSH policy, signing, and credentials in
the proprietary app repository. Dependencies flow from app to core.

- `tavosh-proto` owns schemas, framing and state types. Document versioning and
  compatibility before adding serde wire formats.
- `tavosh-parser` owns deterministic stream slicing, quiescence fallback and
  structured content detection. Inject time in tests. Cover arbitrary read
  boundaries, input-size limits and malformed frames with sanitized fixtures.
  Preserve byte order and keep raw PTY bytes on the native side.
- `tavosh-daemon` owns optional host replay. Keep its binary static where
  supported. Specify authentication, bounded replay and backpressure.
  Standard Mosh SSP is outside v0.

M0 defines the smoke grid and the crate boundaries. Update the tracked protocol,
parser and daemon specs and byte fixtures when their contracts change. Pin
the Rust toolchain in `mise.toml`, justify additions to dependencies, and
commit `Cargo.lock` changes. Run `cargo fmt --all -- --check` before committing.

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
