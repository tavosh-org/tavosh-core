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
