"""Small PTY driver for optional host integration tests; stdout is captured bytes."""
import fcntl
import os
from pathlib import Path
import pty
import select
import shutil
import subprocess
import sys
import tempfile
import termios
import time

shell, hook = sys.argv[1:]
if not shutil.which(shell):
    sys.exit(77)
with tempfile.TemporaryDirectory(prefix='tavosh-hook-') as directory:
    root = Path(directory)
    env = dict(PATH='/usr/bin:/bin', HOME=directory, TERM='xterm-256color', LANG='C.UTF-8',
               XDG_CONFIG_HOME=directory+'/config', XDG_DATA_HOME=directory+'/data',
               XDG_CACHE_HOME=directory+'/cache')
    source = Path(hook).read_text()
    if shell == 'bash':
        rc = root / 'bashrc'
        rc.write_text("PS1='$ '\n" + source)
        args = ['bash', '--noprofile', '--rcfile', str(rc), '-i']
    elif shell == 'zsh':
        (root / '.zshrc').write_text("PROMPT='$ '\n" + source)
        env['ZDOTDIR'] = directory
        args = ['zsh', '-i']
    else:
        args = ['fish', '--no-config', '-i', '-C', source]
    master, slave = pty.openpty()
    def terminal():
        os.setsid()
        fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
    child = subprocess.Popen(args, stdin=slave, stdout=slave, stderr=slave, env=env, preexec_fn=terminal)
    os.close(slave)
    captured = bytearray()
    def prompt():
        response = bytearray()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                data = os.read(master, 65536)
                captured.extend(data)
                response.extend(data)
                if b"\x1b[0c" in data:
                    os.write(master, b"\x1b[?1;2c")
                if b'\x1b]133;B\x07' in response:
                    return
        raise RuntimeError(f'{shell}: prompt marker timeout {response!r}')
    try:
        prompt()
        for command in [b"printf 'hello\\n'\n", b'false\n', b"printf '{\"ok\":true}\\n'\n"]:
            os.write(master, command)
            prompt()
        os.write(master, b'exit\n')
        child.wait(timeout=5)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        os.close(master)
    sys.stdout.buffer.write(captured)
