# Public core setup

Use the pinned Rust toolchain in `mise.toml`. From this repository:

```sh
mise trust ./mise.toml
mise install
mise exec -- cargo fmt --all -- --check
mise exec -- cargo check --locked --workspace
mise exec -- cargo test --locked --workspace
```

This checkout has a Rust-only toolchain. Rustup and Cargo state default to
`target/toolchains/` under the checkout.

Run all checks from the core checkout. Use sanitized fixtures for automated
VT100 and OSC trace replay as conformance coverage is added. The app tracks
its own Android and EAS setup in its separate checkout.
