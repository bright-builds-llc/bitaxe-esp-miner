---
status: completed
created: 2026-07-04
quick_id: 260704-eql
title: Add JSON Pool Credentials Support
---

# Plan

## Scope

- Make JSON the preferred local pool credential format for Phase 21 live mining verification.
- Keep `BITAXE_POOL_*` environment variables as the wrapper's internal runtime interface and `.env` as a documented fallback.
- Preserve the existing safety model: ignored real credentials, redacted logs/evidence, explicit or freshly derived `DEVICE_URL` only, and no raw pool values in committed artifacts.

## Tasks

- [x] Add trackable JSON credential sample and ignore real JSON credential files.
- [x] Add a dependency-free Node helper that validates JSON and emits shell assignments without printing values on errors.
- [x] Teach `scripts/phase21-live-mining-evidence.sh` to accept `--pool-credentials`, load JSON before live-prerequisite checks, and redact JSON/env pool fields.
- [x] Update wrapper tests for missing credentials, JSON success, malformed/missing JSON failure, redaction, and env fallback.
- [x] Update repo guidance, Ultra 205 release docs, Bazel script data, and quick-task tracking.
- [x] Run the requested wrapper, diff, and Rust pre-commit verification before commit.
