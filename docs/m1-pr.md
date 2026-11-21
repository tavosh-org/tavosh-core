# M1 terminal trace format and fixtures

**Base** `main` · **Head** `feat/m1-gpu-tui-engine`

## Summary

Define a versioned binary replay format with little-endian record delays and
payload lengths. Add bounded Rust reader and writer helpers with tests for
fragmented reads, unsupported versions, truncation, and oversized records.

Ship reproducible synthetic traces for VT100 conformance, btop-style bursts,
and an nvim-style session with alternate-screen, mouse, and paste-mode
transitions. The companion app PR consumes the same fixtures for host
conformance tests and its mobile replay harness.

## Review

- Check format compatibility and record validation. [Protocol](protocol.md)
- Check the fixture coverage and regenerate the streams with
  `python3 fixtures/generate_m1.py`. [Fixtures](../fixtures/README.md)

## Verification

- [x] Rust workspace check, tests, formatting, and Clippy with warnings denied.
- [x] Fixture regeneration produced identical committed files.
