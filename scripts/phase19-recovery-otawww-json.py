#!/usr/bin/env python3
"""JSON transformations for the Phase 19 recovery/OTAWWW shell."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


def print_manifest_field(target_path: Path, field: str) -> None:
    try:
        value: Any = json.loads(target_path.read_text(encoding="utf-8"))
    except Exception:
        print("unavailable")
        return

    for part in field.split("."):
        if not isinstance(value, dict) or part not in value:
            print("unavailable")
            return
        value = value[part]
    print("unavailable" if value is None else value)


def collect_device_urls(value: Any, urls: list[str]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if str(key) == "device_url" and isinstance(child, str):
                urls.append(child)
            else:
                collect_device_urls(child, urls)
    elif isinstance(value, list):
        for child in value:
            collect_device_urls(child, urls)


def extract_flash(target_path: Path) -> int:
    try:
        data = json.loads(target_path.read_text(encoding="utf-8"))
    except Exception as error:
        print(f"error=flash evidence JSON is unreadable: {error}")
        return 1

    command_kind = str(data.get("command_kind", ""))
    command = str(data.get("command", ""))
    if "flash-monitor" not in command_kind and "flash-monitor" not in command:
        print("error=flash evidence is not a flash-monitor command")
        return 1
    if str(data.get("board", "")) != "205":
        print("error=flash board is not 205")
        return 1
    if data.get("trusted_output") is not True:
        print("error=flash trusted_output is not true")
        return 1
    if str(data.get("redaction_mode", "")).lower() in {
        "raw",
        "raw-target",
        "unredacted",
    }:
        print("error=flash evidence redaction_mode cannot be raw target")
        return 1

    urls: list[str] = []
    collect_device_urls(data, urls)
    monitor_value = (
        data.get("monitor_log_path")
        or data.get("flash_monitor_log_path")
        or data.get("flash_monitor_log")
        or data.get("log_path")
    )
    if monitor_value:
        monitor_path = Path(str(monitor_value))
        if not monitor_path.is_file() and not monitor_path.is_absolute():
            maybe_relative = target_path.parent / monitor_path
            if maybe_relative.is_file():
                monitor_path = maybe_relative
        if monitor_path.is_file():
            urls.extend(
                match.decode("ascii", errors="ignore").split("=", 1)[1]
                for match in re.findall(
                    rb'device_url=https?://[^\s"<>]+',
                    monitor_path.read_bytes(),
                )
            )

    unique_urls = sorted(set(urls))
    if len(unique_urls) != 1:
        print("error=flash evidence must contain exactly one device_url marker")
        return 1

    selected_port = data.get("selected_port") or data.get("port") or ""
    print(f"device_url={unique_urls[0]}")
    print(f"selected_port={selected_port}")
    return 0


def write_target_lock(arguments: list[str]) -> None:
    (
        target_path,
        target_status,
        source,
        redacted,
        selected_port,
        source_commit,
        reference_commit,
        manifest,
        flash_json,
    ) = arguments
    payload = {
        "target_status": target_status,
        "device_url_source": source,
        "device_url_redacted": redacted,
        "selected_port": selected_port,
        "network_scan": "disabled",
        "source_commit": source_commit,
        "reference_commit": reference_commit,
        "manifest": manifest,
        "flash_evidence_json": flash_json,
    }
    Path(target_path).write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main(arguments: list[str]) -> int:
    if not arguments:
        return 2
    command, *values = arguments
    if command == "manifest-field" and len(values) == 2:
        print_manifest_field(Path(values[0]), values[1])
        return 0
    if command == "extract-flash" and len(values) == 1:
        return extract_flash(Path(values[0]))
    if command == "write-target-lock" and len(values) == 9:
        write_target_lock(values)
        return 0
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
