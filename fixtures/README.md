# Terminal fixtures

`osc133-alternate.vt` is a synthetic byte stream with shell markers, an
alternate-screen entry/exit and exit status 0. Replay it bytewise and with
arbitrary chunk boundaries. When recording sessions, scrub secrets, hostnames
and personal paths before adding a capture. Preserve escape bytes.

## M1 replay fixtures

Run `python3 fixtures/generate_m1.py` from the core checkout to reproduce
these synthetic fixtures. They use the [v1 trace format](../docs/protocol.md#terminal-replay-trace-v1).
Use an 80-column by 24-row grid for reference playback.

| Fixture | Coverage |
| --- | --- |
| `vt100_conformance.trace` | Cursor addressing and shape, scroll margins, SGR colors and styles, UTF-8, wide characters, and box drawing |
| `btop_stress.trace` | 600 timed incremental updates across a two-pane dashboard, using truecolor and block elements |
| `nvim_session.trace` | Alternate screen and restoration, split panes, statusline, cursor movement, mouse modes, and bracketed paste |
