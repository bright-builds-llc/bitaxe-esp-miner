#!/usr/bin/env python3
"""Receive bytes from a 115200 8N1 UART without transmitting or toggling lines."""

from __future__ import annotations

import os
import select
import signal
import sys
import termios


READ_WAIT_SECONDS = 0.25
READ_SIZE = 4096
running = True


def stop_reader(_signal_number: int, _frame: object) -> None:
    global running
    running = False


def configure_receive_only_uart(file_descriptor: int) -> None:
    attributes = termios.tcgetattr(file_descriptor)
    attributes[0] = 0
    attributes[1] = 0
    attributes[2] = termios.CLOCAL | termios.CREAD | termios.CS8
    attributes[3] = 0
    attributes[4] = termios.B115200
    attributes[5] = termios.B115200
    attributes[6][termios.VMIN] = 0
    attributes[6][termios.VTIME] = 0
    termios.tcsetattr(file_descriptor, termios.TCSANOW, attributes)


def receive(port: str) -> int:
    flags = os.O_RDONLY | os.O_NOCTTY | os.O_NONBLOCK
    file_descriptor = os.open(port, flags)
    try:
        configure_receive_only_uart(file_descriptor)
        while running:
            readable, _, _ = select.select(
                [file_descriptor], [], [], READ_WAIT_SECONDS
            )
            if not readable:
                continue
            try:
                payload = os.read(file_descriptor, READ_SIZE)
            except BlockingIOError:
                continue
            if not payload:
                continue
            sys.stdout.buffer.write(payload)
            sys.stdout.buffer.flush()
    finally:
        os.close(file_descriptor)
    return 0


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: phase13-uart-native-reader.py PORT", file=sys.stderr)
        return 2

    signal.signal(signal.SIGINT, stop_reader)
    signal.signal(signal.SIGTERM, stop_reader)
    return receive(sys.argv[1])


if __name__ == "__main__":
    raise SystemExit(main())
