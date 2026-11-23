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
types, and the optional host companion. The mobile client uses the shared
connection schema to report verification, authentication and roaming state.
Standard SSH sessions work independently of the companion.

**Development status.** M3 implements the native adaptive stream parser:
OSC 133 boundaries, heuristic fallback, OSC 7501 status, JSON/TOML trees,
validated unified diffs and Feed/TUI transitions. Bounded records and incremental
feed schemas are shared with the app. [Parser rules](docs/parser.md) and
[reproducible M3 fixtures](fixtures/README.md#m3-adaptive-traces) document the
contract. The optional daemon remains independent of standard SSH.

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
| [`fixtures/`](fixtures) | Sanitized samples and deterministic terminal replay fixtures |

## Documentation

- [Architecture](docs/architecture.md) and [roadmap](docs/roadmap.md) cover the public boundary and milestone status.
- [Protocol design](docs/protocol.md) covers versioning, message framing, and replay offsets.
- [Parser design](docs/parser.md) covers OSC 133 boundaries, timing fallback, and output detection.
- [Daemon design](docs/daemon.md) covers session replay and agent IPC.
- [Development setup](docs/setup.md) and [repository guidelines](AGENTS.md) cover local checks and contribution workflow.

## License

Tavosh Core is available under either the [MIT License](LICENSE) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.
