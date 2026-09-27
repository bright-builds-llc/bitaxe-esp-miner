#!/usr/bin/env python3
"""Run one marked software client on a real PTY, then close the terminal."""
import os
import pty
import select
import subprocess
import sys
import time

master, slave = pty.openpty()
child = subprocess.Popen(sys.argv[1:], stdin=slave, stdout=slave, stderr=slave)
os.close(slave)
output = bytearray()
try:
    deadline = time.monotonic() + 10
    while child.poll() is None:
        if time.monotonic() >= deadline:
            raise TimeoutError("synthetic PTY client deadline")
        if select.select([master], [], [], 0.05)[0]:
            try:
                output.extend(os.read(master, 16384))
            except OSError:
                break
    code = child.wait(timeout=3)
finally:
    if child.poll() is None:
        child.kill()
        child.wait(timeout=3)
    os.close(master)
sys.stdout.buffer.write(output)
raise SystemExit(code)
