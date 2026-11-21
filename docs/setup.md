# Public core setup

Use the pinned Rust toolchain in `mise.toml`. From this repository:

```sh
mise trust ./mise.toml
mise install
mise exec -- cargo fmt --all -- --check
mise exec -- cargo check --locked --workspace
mise exec -- cargo test --locked --workspace
```

Inside the hub, trust the parent config as well. The hub installs the
Android toolchain for the private client. The core workspace builds on a
host without Android, Node, Java or Expo when cloned independently.
