---
status: completed
created: 2026-07-04
quick_id: 260704-dxg
title: Add Fresh Monitor Log DEVICE_URL Repo Rule
---

# Summary

Added a repo-local rule allowing agents to derive `DEVICE_URL` from fresh repo-owned monitor output for local testing.

## Changes

- Added a narrow `AGENTS.md` exception under Autonomous Ultra 205 Hardware Verification.
- The rule requires a same-session successful `just detect-ultra205`, corresponding repo-owned monitor output, exactly one origin-only HTTP(S) candidate, and local runtime-only use.
- The rule still forbids mDNS, ARP, router, scan, stale-log, unrelated-evidence, ambiguous, redacted, or malformed target inference.
- Raw target, endpoint, IP, MAC, Wi-Fi, pool, worker, password, token, and NVS secret values remain forbidden in committed evidence.

## Verification

- `git diff --check`
- `rg "DEVICE_URL|monitor logs|detect-ultra205" AGENTS.md`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Residual Risk

- The rule trusts only fresh same-session monitor output. If the firmware prints multiple, redacted, or malformed URL candidates, agents must stop and record the target as pending instead of guessing.
