# Public Tavosh core

This workspace is dual-licensed under MIT OR Apache-2.0. Read the parent
workspace rules when working in the hub. Keep the public crates
platform-independent and free of app UI and credentials.

- `tavosh-proto` owns shared schemas, framing and state types. Preserve explicit
  versioning and lay out compatibility rules before adding a serde wire format.
- `tavosh-parser` owns deterministic stream slicing, quiescence fallback and
  structured content detection. Inject time in tests. Cover arbitrary read
  boundaries, hostile input sizes and malformed framing with sanitized fixtures.
- `tavosh-daemon` owns optional host replay. Keep its deployable binary static
  where supported. Treat authentication, backpressure and replay overflows as
  protocol concerns. Keep mobile reconnect policy in the private repo.

M0 defines only the smoke-test grid and crate boundaries. Protocol changes
require updated specs and byte fixtures. Run Rust formatting, workspace checks
and locked tests before committing. Add dependencies only for a documented
responsibility and keep Cargo.lock current.
