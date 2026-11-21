```text
                              ##
          #########        ####
      #######################
   #######    ###########
  ####         ####
  ##           ####
               ####
               ####      ###
              ####     #####
              ####    ###
             ####     ###
        #   ####       ######
       ##############   ####
        ###########
```

# Tavosh Core

Tavosh (Tavo) is a GPU-accelerated, mobile-first SSH terminal and remote AI
agent cockpit for developers using remote machines and long-lived CLI agents.
This public Rust repository provides the shared contracts for its real
terminal, touch-first command feed and optional stateful roaming. Personal
daily utility is the v0 bar. The core is dual-licensed under [MIT](LICENSE)
OR [Apache-2.0](LICENSE-APACHE).

- `crates/tavosh-proto` defines shared state and framing contracts.
- `crates/tavosh-parser` owns stream slicing and structured output detection.
- `crates/tavosh-daemon` reserves the host companion entry point.
- `fixtures` contains sanitized terminal bytes for parser replay.

Run `mise trust`, `mise install` and
`mise exec -- cargo test --locked --workspace`. See [setup](docs/setup.md)
and the [protocol](docs/protocol.md), [parser](docs/parser.md) and
[daemon](docs/daemon.md) designs.

**M0 status.** Host checks pass. The parser validates the shared smoke grid,
and the daemon prints its version. The wire protocol and terminal parsers
are designs for later milestones.
