# Parser design

M0 validates the smoke grid's size and two-value cell alphabet. For shell
integration, parse OSC 133 A (prompt), B (command entry), C (execution)
and D (completion with optional exit status). Retain escape parser state
across arbitrary read boundaries and support BEL and ST terminators. Close a
card on completion and retain unknown exit status as an explicit value.

When markers are absent, match configurable shell prompts and use a
50–100 ms quiescence window as a heuristic. Reuse the same state machine
under a controlled clock in fixture tests. The adapter must surface
heuristic card boundaries. Detect alternate-screen transitions independently
of card segmentation. Only sniff JSON, TOML and unified diffs on completed,
size-bounded output. Keep the raw display record available on parse failure.
