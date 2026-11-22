# Tavosh Core architecture

The public Rust workspace owns versioned protocol framing, stream parsing,
synthetic terminal fixtures and the optional host-side companion. The app
depends on the core and owns SSH transport, libvterm and the mobile UI.

M1 adds a deterministic [terminal trace format](protocol.md#terminal-replay-trace-v1).
Its relative delays and raw payloads drive the app's native terminal worker
in timed replay or as fast as the worker can consume them. The
[fixture generator](../fixtures/generate_m1.py) produces VT100, btop and nvim
regression streams. Rust tests cover fragmented reads, malformed records and
the end of the stream. The app tests viewport behavior by replaying the same
fixtures through libvterm.

The app's display frame ABI and Skia batching strategy live in its architecture
guide. The core trace format carries terminal bytes and timing. On mobile,
those bytes stay in the native worker.

See [parser design](parser.md), [daemon design](daemon.md) and the
[roadmap](roadmap.md) for the remaining public components.

## M2 connection schema

`tavosh-proto::connection::v1` defines SSH session configuration, connection
states, disconnect reasons, and host-key fingerprints and decisions. Credential
fields reference app-managed entries. The schema carries no private key bytes.
The app selects the backend, owns the trust store, and drives its SSH transport
and tmux attachment. The future daemon can reuse the shared schema where
appropriate. See the [schema contract](protocol.md#ssh-connection-schemas).
