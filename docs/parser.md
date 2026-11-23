# M3 adaptive stream parser

`StreamParser` is platform-independent, safe Rust. A backend supplies ordered
bytes through `push` and monotonic session-relative milliseconds through `tick`.
The app invokes it alongside libvterm on the same dedicated Rust worker. Neither
PTY bytes nor parsing, syntax validation or sniffing crosses into JavaScript.
`submitted` optionally supplies a locally composed command for heuristic mode.

## Boundaries and controls

OSC 133 accepts `A` (prompt), `B` (input), `C` (execution), and `D` or
`D;<signed exit code>` (completion). BEL and ST terminate OSC frames. Partial
escape, CSI, OSC and UTF-8 input state survives arbitrary read boundaries.
Unknown and malformed markers are ignored. Oversized controls are discarded
through their terminator. DCS/APC/PM/SOS contents never become feed text.
The B-to-C input line applies backspaces and cursor/erase operations before
capturing a command, handling readline and ZLE redraws. Raw-terminal editing
with unusual terminal column widths can still need the manual viewport toggle.

Before the first valid OSC 133 marker, the parser operates heuristically.
Prompt suffixes default to `$ `, `# `, `% ` and `❯ `; a bounded shell-like
prefix recognizes `user@host:path$ `. Callers can configure up to 16 suffixes.
A 75 ms idle period completes a card; configuration clamps this to 50–100 ms.
Prompt and known command echo lines are removed before sniffing. A valid OSC
133 marker immediately and permanently promotes that attachment to shell-hook
mode. Idle gaps no longer complete commands. Reattachment creates a fresh
parser. Heuristic boundaries can split a slowly streaming command and cannot
infer its exit code; unknown status remains `null`, never a fabricated success.

Shell-specific bash, zsh and fish templates are exported by the crate. They
preserve normal prompts and prompt callbacks, and install once per shell.
Bash declines integration when a DEBUG trap already exists. The app selects
and installs the source out of band through SSH, keeping it out of shell
history. Existing tmux panes and shells that strip markers use heuristics;
the parser does not require tmux configuration or a daemon.

OSC 7501 uses semicolon-separated `key=value` fields. Required fields are
`state=working|done|blocked|error`, `kind` and `app`. Optional fields are
`message` and `title`; `=` inside a value is preserved. Semicolons delimit
fields and cannot occur inside values in v1. Duplicate keys, invalid states,
invalid UTF-8 and controls over 4096 bytes are rejected. Unknown fields are
ignored for forward compatibility. Latest status is retained independently
of command cards, ready for future agent adapters.

## Classification and limits

Completion runs deterministic native validators in order: JSON (`serde_json`),
unified text diff, TOML (`toml`), then plain text. ANSI-aware preprocessing
removes escape strings and styles and projects carriage-return overwrites.
A terminal's transient erased prompt line does not corrupt an otherwise valid
JSON/TOML payload. JSON and TOML normalize to the same tree with RFC 6901 paths,
labels, types, scalar values, original child counts and truncation flags.
JSON paths escape `~` and `/`; numeric scalar spellings stay native strings.
Diffs retain paths, hunks, typed lines and added/removed counts. Hunk ranges
must balance exactly. Binary, combined, malformed or oversized diffs fall back
to their plain-text preview.

| Resource | Bound |
| --- | --- |
| OSC/CSI control or captured input line | 4096 bytes / input characters |
| Retained classifier input | 1 MiB per active command; larger output skips AST parsing |
| Text preview | 16 KiB of UTF-8 presentation text per card |
| Tree | 512 nodes, 24 levels, 256-byte labels/values, 2048-byte paths |
| Structured representation | 256 KiB serialized; larger trees/diffs fall back to text |
| Diff | 64 files, 512 retained lines, 4096 bytes per line |
| Feed | At most 31 completed cards plus one active card; 2 MiB accounted completed-record budget |
| Lifecycle queue | 64 events and a 2 MiB accounted record budget; consumers drain regularly |

Accounting includes field and container overhead estimates; allocator capacity
adds finite overhead. Independent limits apply to the parser input, previews,
records and pending bridge document. Full input byte and line counts continue
after the preview cap, with `truncated: true`. Counts describe text bytes after
control-sequence removal, before carriage-return projection. Terminal emulation
continues consuming every original byte even when feed retention is capped.
`Started`, coalescible `OutputDelta`, and `Completed` events share core schemas.
Old lifecycle events/records are evicted at their bounds; snapshots are the
retained source of truth.

## Viewport

DEC private modes 1049, 1047 and 47 select `TuiCanvas` on `h` and `Feed` on `l`,
including combined parameters and fragmented sequences. While alternate screen
is active, screen repaint bytes do not enter card output. Exit produces a
compact `TuiSummary`; command completion supplies its exit code when available.
The app keeps both rendering surfaces mounted and has a manual Feed/Raw override
that changes presentation without changing the parser's physical buffer state.

## Verification

`cargo test --workspace` covers split reads, BEL/ST, malformed and bounded
controls, controlled idle timing, promotion, status validation, colored JSON,
TOML and diffs, whitespace in diff context, large payloads, bounded retention,
and both complete M3 traces. Unix host integration tests use a small Python
PTY driver to exercise installed bash, zsh and fish, including command headers,
non-zero exit status and real JSON output. Missing Python/shell executables are
reported as skips; deterministic Rust tests do not depend on them.
