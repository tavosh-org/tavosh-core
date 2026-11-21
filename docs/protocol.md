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
