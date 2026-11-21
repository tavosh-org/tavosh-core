# Terminal fixtures

`osc133-alternate.vt` is a synthetic byte stream with shell markers, an
alternate-screen entry/exit and exit status 0. Replay it bytewise and with
arbitrary chunk boundaries. When recording sessions, scrub secrets, hostnames
and personal paths before adding a capture. Preserve escape bytes.
