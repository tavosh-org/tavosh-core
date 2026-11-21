```text
 #########
    ###
    ###
    ###
    ###
```

# Tavosh Core

The public Rust core defines platform-independent protocol types, deterministic
terminal parsing and an optional host replay daemon. It is dual-licensed under
[MIT](LICENSE) OR [Apache-2.0](LICENSE-APACHE).

- `crates/tavosh-proto` defines shared state and framing contracts.
- `crates/tavosh-parser` owns stream slicing and structured output detection.
- `crates/tavosh-daemon` reserves the host companion entry point.
- `fixtures` contains sanitized terminal bytes for parser replay.

Run `mise trust`, `mise install` and
`mise exec -- cargo test --locked --workspace`. See [setup](docs/setup.md)
and the [protocol](docs/protocol.md), [parser](docs/parser.md) and
[daemon](docs/daemon.md) designs.

**M0 status:** The host workspace passes checks and tests. The crate
scaffolding includes the shared smoke-grid contract.
