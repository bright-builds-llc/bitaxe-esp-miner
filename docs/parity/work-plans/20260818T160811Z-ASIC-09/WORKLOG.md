# Parity work log

## 2026-08-19T14:50:00Z | source-bound live chain

- Source commit: `7f8ca3bb9d6e9b7b56d1040b1d6d6eeb2bf2648d`.
- Actions: resumed the open ASIC-09 plan, independently validated the four
  accepted public ASIC projections, ran current diagnostic-admission and
  production-command tests, reviewed the production executor source, packaged
  firmware, and wrote the source-bound summary.
- Verification: projection digests matched the immutable plan; all four Rust
  validators accepted; adapter-gate tests 8 passed; production tests 9 passed;
  production executor contains no diagnostic-work variant; reference is clean;
  `just package` produced the current Ultra 205 image artifacts.
- Evidence:
  `docs/parity/evidence/asic09-mode-separation/summary.md` joining
  `eee750561a7c1dcec1a5698b1e5827d3f1508d43655c3c4aa237097338dcf8d4`,
  `447af65ae9e6cd5cc2199ef639ff8e0fa7f63d4c9708570bd66781c5a162e80c`,
  `e99c054c4d660155d5c2b1ee38d3f17aed5ae7101e7e4a5fd1c6451d1b48b7c7`, and
  `bad828db694ee59c4ef3d77b2e58ef89e0195ef382526b97912d0a71e882ad69`.
- Outcome: accepted same-chain evidence supports ASIC-09 promotion to verified.
- Blocker or next safe action: none for ASIC-09. Commit summary, `WORKLOG.md`,
  and `RESULT.md` as `SOURCE_COMMIT`; then transition only ASIC-09, sync
  progress, archive this task, final-gate, and push.
