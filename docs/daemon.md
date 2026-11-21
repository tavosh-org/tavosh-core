# Daemon design

M0 ships a version-printing stub. The optional host companion will retain
a bounded, ordered byte stream and replay from a validated session offset
over an authenticated channel. Preserve OSC, CSI and arbitrary UTF-8 bytes
through replays. Standard Mosh SSP is outside the v0 design.

Use per-session authorization, cap replay history and concurrent sessions,
and report expired output explicitly. Keep SSH and tmux reattachment usable
without the companion. Validate nonblocking backpressure and network
handoffs before packaging a static binary for supported host architectures.
