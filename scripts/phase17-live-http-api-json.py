#!/usr/bin/env python3
"""JSON and monitor-log transformations for the Phase 17 shell."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


def print_field(target_path: Path, field: str) -> None:
    try:
        value = json.loads(target_path.read_text(encoding="utf-8")).get(field)
    except Exception:
        return

    if isinstance(value, bool):
        print("true" if value else "false")
    elif value is not None:
        print(value)


def print_monitor_urls(target_path: Path) -> None:
    data = target_path.read_bytes()
    urls = sorted(
        {
            match.decode("ascii", errors="ignore").split("=", 1)[1]
            for match in re.findall(rb'device_url=https?://[^\s"<>]+', data)
        }
    )
    print("\n".join(urls))


def write_target_lock(arguments: list[str]) -> None:
    (
        target_path,
        target_status,
        source,
        redacted,
        board,
        selected_port,
        source_commit,
        reference_commit,
        manifest,
        flash_json,
    ) = arguments
    payload: dict[str, object] = {
        "target_status": target_status,
        "device_url_source": source,
        "device_url_redacted": redacted,
        "board": board,
        "source_commit": source_commit,
        "reference_commit": reference_commit,
        "manifest": manifest,
        "flash_evidence_json": flash_json,
        "network_scan": "disabled",
        "created_from_explicit_input": True,
    }
    if selected_port:
        payload["selected_port"] = selected_port
    Path(target_path).write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main(arguments: list[str]) -> int:
    if not arguments:
        return 2

    command, *values = arguments
    if command == "field" and len(values) == 2:
        print_field(Path(values[0]), values[1])
        return 0
    if command == "monitor-urls" and len(values) == 1:
        print_monitor_urls(Path(values[0]))
        return 0
    if command == "write-target-lock" and len(values) == 10:
        write_target_lock(values)
        return 0
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
