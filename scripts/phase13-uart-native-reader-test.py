#!/usr/bin/env python3
"""Host regression tests for the receive-only UART reader."""

from __future__ import annotations

import os
from pathlib import Path
import pty
import select
import subprocess
import sys
import termios
import time


SCRIPT = Path(__file__).with_name("phase13-uart-native-reader.py")


def fail(message: str) -> None:
    raise AssertionError(message)


def wait_for_framing(slave_fd: int) -> None:
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        attributes = termios.tcgetattr(slave_fd)
        expected = termios.CLOCAL | termios.CREAD | termios.CS8
        if attributes[2] & expected == expected:
            return
        time.sleep(0.02)
    fail("reader did not configure 115200 8N1 local receive mode")


def read_exact(stream_fd: int, expected_length: int) -> bytes:
    received = bytearray()
    deadline = time.monotonic() + 3
    while len(received) < expected_length and time.monotonic() < deadline:
        readable, _, _ = select.select([stream_fd], [], [], 0.1)
        if readable:
            received.extend(os.read(stream_fd, expected_length - len(received)))
    return bytes(received)


def test_pty_delivery_and_framing() -> None:
    # Arrange
    master_fd, slave_fd = pty.openpty()
    slave_name = os.ttyname(slave_fd)
    process = subprocess.Popen(
        [sys.executable, str(SCRIPT), slave_name],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if process.stdout is None:
        fail("reader stdout pipe is unavailable")

    try:
        wait_for_framing(slave_fd)
        attributes = termios.tcgetattr(slave_fd)
        payload = b"runtime_heartbeat session=0123456789abcdef0123456789abcdef\n"

        # Act
        os.write(master_fd, payload[:17])
        os.write(master_fd, payload[17:])
        received = read_exact(process.stdout.fileno(), len(payload))

        # Assert
        if received != payload:
            fail(f"fragmented PTY payload mismatch: {received!r}")
        if attributes[4] != termios.B115200 or attributes[5] != termios.B115200:
            fail("UART speed is not 115200")
        if attributes[2] & termios.PARENB:
            fail("UART parity is enabled")
        if hasattr(termios, "CSTOPB") and attributes[2] & termios.CSTOPB:
            fail("UART uses two stop bits")
    finally:
        process.terminate()
        process.wait(timeout=3)
        os.close(master_fd)
        os.close(slave_fd)


def test_reader_source_has_no_transmit_or_modem_control() -> None:
    # Arrange
    source = SCRIPT.read_text(encoding="utf-8")
    prohibited = ("os.write(", "tcflow(", "TIOCM", "TIOCMB", "TIOCSBRK")

    # Act
    present = [token for token in prohibited if token in source]

    # Assert
    if present:
        fail(f"reader contains prohibited transmit/modem operations: {present}")


def main() -> int:
    test_pty_delivery_and_framing()
    test_reader_source_has_no_transmit_or_modem_control()
    print("phase13_uart_native_reader_test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
