#!/usr/bin/env python3
"""Regenerate the deterministic, synthetic M1 terminal traces."""
from pathlib import Path
from struct import pack

ESC = "\x1b["
ROOT = Path(__file__).resolve().parent


def write(name, records):
    with (ROOT / f"{name}.trace").open("wb") as output:
        output.write(b"TAVOTRC\x01")
        for delay, data in records:
            payload = data.encode("utf-8")
            output.write(pack("<II", delay, len(payload)))
            output.write(payload)


def at(row, col, data):
    return f"{ESC}{row};{col}H{data}"


write("vt100_conformance", [
    (0, ESC + "2J" + at(1, 1, "VT100 ┌─┐ 界")),
    (20, at(2, 1, ESC + "31;44;1;3;4mA" + ESC + "7mB" + ESC + "0m")),
    (20, at(3, 1, ESC + "38;5;196;48;2;10;20;30mC" + ESC + "0m")),
    (20, ESC + "5;7r" + at(5, 1, "old\r\nkeep\r\nlast\r\nnew") + ESC + "r"),
    (20, at(4, 3, "─┼─") + at(4, 5, "") + ESC + "2 q"),
])

border = "┌" + "─" * 34 + "┬" + "─" * 36 + "┐"
panel = at(1, 1, border)
for row in range(2, 18):
    panel += at(row, 1, "│") + at(row, 36, "│") + at(row, 73, "│")
panel += at(18, 1, "└" + "─" * 34 + "┴" + "─" * 36 + "┘")
panel += at(2, 3, "CPU / memory") + at(2, 38, "Processes")
stress = [(0, ESC + "2J" + ESC + "?25l" + panel)]
for tick in range(600):
    row = 3 + tick % 15
    blocks = 1 + (tick * 7) % 27
    bar = f"{ESC}38;2;{tick % 256};180;220m" + "█" * blocks
    bar += ESC + "0m" + " " * (27 - blocks)
    process = f"{1000 + row:5d}  worker-{row:02d}   {tick % 100:2d}%"
    stress.append((4, at(row, 3, bar) + at(row, 38, process)))
stress.append((0, ESC + "?25h"))
write("btop_stress", stress)

editor = [
    (0, ESC + "2J" + at(1, 1, "shell $ nvim src/main.rs")),
    (100, ESC + "?1049h" + ESC + "?1002h" + ESC + "?1006h" + ESC + "?2004h"),
    (16, at(1, 1, ESC + "35mfn" + ESC + "0m main() {") +
     at(2, 1, '    println!("hello");') + at(3, 1, "}") +
     "".join(at(row, 36, "│") for row in range(1, 15)) +
     at(1, 38, ESC + "36mREADME.md" + ESC + "0m") +
     at(15, 1, ESC + "7m NORMAL  src/main.rs " + ESC + "0m")),
    (120, at(2, 5, "") + ESC + "6 q"),
    (500, ESC + "?2004l" + ESC + "?1006l" + ESC + "?1002l" + ESC + "?1049l"),
]
write("nvim_session", editor)
