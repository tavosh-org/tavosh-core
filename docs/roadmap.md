# Tavosh Core roadmap

| Milestone | Core scope | Status |
| --- | --- | --- |
| M0 | Shared smoke-grid contract and initial workspace | Host Rust checks pass |
| M1 | Versioned terminal traces and reproducible VT100, btop and nvim fixtures | Core trace reader and writer verified. The overall M1 exit gate includes the app's native builds and device measurements |
| M2 | Versioned SSH session configuration, connection state and host-key schema | Core schema and serialization tests pass |
| M3 | OSC 133 / OSC 7501, heuristic boundaries, bounded JSON/TOML/diff output, viewport and incremental feed schemas | Implemented; host checks and reproducible replay pass. App/device validation is tracked in the app repository |
| M4 | Optional authenticated replay and agent IPC | Planned |

Standard SSH is designed to work independently of the optional daemon.

M3 verification on 2026-11-23 passes 20 core tests, formatting, locked workspace
check and Clippy with warnings denied. Controlled-clock replay covers both M3
traces at arbitrary boundaries. Optional Unix PTY integration also exercised
all three installed shells (bash, zsh and fish), real JSON output and failure
status. Physical mobile rendering, both Android native ABIs and the private UI
are tracked separately in the app repository.
