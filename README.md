<h1 align="center">
  <img src="docs/assets/tavosh-core-icon.png" width="88" alt=""><br>
  Tavosh Core
</h1>

<p align="center">Shared Rust components for the Tavosh SSH terminal and remote coding-agent cockpit.</p>

<p align="center">
  <a href="#getting-started">Getting started</a> ·
  <a href="#workspace">Workspace</a> ·
  <a href="#documentation">Documentation</a> ·
  <a href="#license">License</a>
</p>

## About

Tavosh Core is the public workspace for terminal stream parsing, shared protocol
types, and the optional host companion. The mobile client uses these components
to interpret shell output and, eventually, to resume sessions across network
changes. Standard SSH sessions work independently of the companion.

**Development status:** The current milestone defines a smoke-test frame shared
with the mobile client. The parser validates that frame, and the daemon reports
its version. OSC 133 parsing, replay, and agent IPC are described in the design
docs and are still in development.

## Getting started

From the repository root, use [mise](docs/setup.md) to install the pinned Rust
toolchain and check the workspace:

```sh
mise trust ./mise.toml
mise install
mise exec -- cargo check --locked --workspace
mise exec -- cargo test --locked --workspace
```

## Workspace

| Path | Responsibility |
| --- | --- |
| [`crates/tavosh-proto`](crates/tavosh-proto) | Shared protocol types and framing |
| [`crates/tavosh-parser`](crates/tavosh-parser) | Terminal output boundaries and structured output detection |
| [`crates/tavosh-daemon`](crates/tavosh-daemon) | Optional Rust host companion for replay and agent IPC |
| [`fixtures/`](fixtures) | Sanitized terminal byte streams for parser regression coverage |

## Documentation

- [Protocol design](docs/protocol.md) covers versioning, message framing, and replay offsets.
- [Parser design](docs/parser.md) covers OSC 133 boundaries, timing fallback, and output detection.
- [Daemon design](docs/daemon.md) covers session replay and agent IPC.
- [Development setup](docs/setup.md) and [repository guidelines](AGENTS.md) cover local checks and contribution workflow.

## License

Tavosh Core is available under either the [MIT License](LICENSE) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
