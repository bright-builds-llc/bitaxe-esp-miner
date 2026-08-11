#!/usr/bin/env bash
set -euo pipefail

normal_elf=$1
probe_elf=$2
marker=ota_boot_validation=rollback_probe_pending

if grep -a -q "$marker" "$normal_elf"; then
  echo "normal firmware contains rollback-probe marker" >&2
  exit 1
fi
if ! grep -a -q "$marker" "$probe_elf"; then
  echo "rollback-probe firmware is missing its pending-validation marker" >&2
  exit 1
fi
if cmp -s "$normal_elf" "$probe_elf"; then
  echo "normal and rollback-probe firmware must differ" >&2
  exit 1
fi
