# Protocol design

M0 shares in-process smoke-grid dimensions. A 35-byte row-major snapshot
contains seven columns and five rows. ASCII space is empty and `#` is filled.
There is no daemon wire protocol in M0.

For daemon replay, define a versioned handshake and bounded frames with an
explicit type, session identifier, monotonic byte offset and payload length.
Encode structured metadata with serde once fields and compatibility rules
are established. Preserve data frames byte-for-byte; offsets make replay gaps
and duplicate deliveries detectable. A client resumes from its acknowledged
offset. Report expired ranges explicitly.

The companion and app use the same Rust serde schema definitions and parser
crate. Version agent IPC messages alongside replay frames and assign request
IDs so replies and approval decisions correlate across reconnections.
Keep PTY payload bytes opaque in framing and separate from structured IPC.

## Terminal replay trace v1

A trace starts with the eight bytes `TAVOTRC\x01`. Each record contains a
little-endian unsigned 32-bit delay in milliseconds, a little-endian unsigned
32-bit payload length, and that many original terminal bytes. Delays are relative
to the preceding record or to playback start for the first record. Zero delay
and empty payloads are valid. Individual payloads are limited to 1 MiB. EOF is
valid only at a record boundary. Readers reject unsupported magic and versions,
oversized records, and incomplete headers or payloads. This format is for
fixtures and replay and does not change the daemon protocol.
