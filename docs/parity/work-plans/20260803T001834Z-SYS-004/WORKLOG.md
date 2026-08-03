# Parity work log

## 2026-08-03T00:18:34Z | static-version root-cause investigation

- Source commit: `6c377f4d7439d4cedb1162899f93647577c1a478`
- Actions: Selected the first deterministic candidate, traced upstream version
  initialization and the Rust package/runtime path, and defined one bounded
  exact-package hardware contract.
- Verification: Clean synchronized `main`; pinned reference
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`; `next-item` returned no open plan
  and `SYS-004` first at `implemented`. Upstream `SYSTEM_init_versions` reads
  `/version.txt`; Rust package input contains no `version.txt`, while
  `platform_identity.rs` returns the generic `assets/release.json` name.
- Evidence: `reference/esp-miner/main/system.c`,
  `reference/esp-miner/main/http_server/system_api_json.c`,
  `firmware/bitaxe/src/platform_identity.rs`,
  `firmware/bitaxe/static/www/assets/release.json`, and
  `scripts/package-firmware.sh`.
- Outcome: The remaining static-semantics gap is an implementation defect, not
  merely missing evidence. The targeted fix must make the installed SPIFFS
  payload own the reported version before consuming the one hardware attempt.
- Blocker or next safe action: Run all plan pre-commit gates, commit and push
  this immutable plan/task contract, then implement the software fix without
  exercising hardware.
