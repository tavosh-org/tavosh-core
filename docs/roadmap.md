# Tavosh Core roadmap

| Milestone | Core scope | Status |
| --- | --- | --- |
| M0 | Shared smoke-grid contract and initial workspace | Host Rust checks pass |
| M1 | Versioned terminal traces and reproducible VT100, btop and nvim fixtures | Core trace reader and writer verified. The overall M1 exit gate includes the app's native builds and device measurements |
| M2 | Versioned SSH session configuration, connection state and host-key schema | Core schema and serialization tests pass |
| M3 | OSC 133 command boundaries, structured output and shared coding-agent schemas | Planned |
| M4 | Optional authenticated replay and agent IPC | Planned |

Standard SSH is designed to work independently of the optional daemon.
