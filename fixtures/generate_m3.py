"""Generate sanitized M3 trace v1 fixtures, without external services."""
from pathlib import Path
import struct
ROOT = Path(__file__).parent

def write(name, records):
    with (ROOT / name).open('wb') as out:
        out.write(b'TAVOTRC\x01')
        for delay, text in records:
            data = text.encode()
            out.write(struct.pack('<II', delay, len(data)))
            out.write(data)

def osc(text):
    return '\x1b]' + text + '\x07'

records = [(0, osc('133;A') + 'dev@host:~$ ' + osc('133;B'))]
for command, output, code in [
    ('curl https://example.test/data.json', '\x1b[32m{"ok":true,"items":[1,2,3]}\x1b[0m\r\n', 0),
    ('cat Cargo.toml', '[package]\r\nname = "tavo"\r\nversion = "0.1.0"\r\n', 0),
    ('git diff --color=always', 'diff --git a/config b/config\r\n--- a/config\r\n+++ b/config\r\n@@ -1 +1 @@\r\n\x1b[31m-old\x1b[0m\r\n\x1b[32m+new\x1b[0m\r\n', 0),
    ('nvim config.toml', '\x1b[?1049h\x1b[Hprivate TUI screen', 0),
    ('false', 'failed\r\n', 1),
]:
    records.append((100, command + '\r\n' + osc('133;C') + osc('7501;state=working;kind=command;app=shell')))
    records.append((100, output))
    if command.startswith('nvim'):
        records.append((1200, '\x1b[?1049l'))
    records.append((100, osc('7501;state=done;kind=command;app=shell;title=Ready') + osc(f'133;D;{code}') + osc('133;A') + '$ ' + osc('133;B')))
write('osc133_adaptive_session.trace', records)
write('heuristic_fallback.trace', [
    (0, 'user@host:~$ echo hello\r\nhello\r\nuser@host:~$ '), (100, ''),
    (100, 'user@host:~$ cat data.json\r\n{"fallback":true}\r\nuser@host:~$ '), (100, ''),
    (100, '\x1b[?1047hfullscreen'), (1000, '\x1b[?1047l'), (100, ''),
])
