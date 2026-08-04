# Parity work log

## 2026-08-04T16:00:00Z | selection and plan

- Source commit: `bb007f5f82c86437974616dea5701bfd2973a095`.
- Actions: Continued deterministic selection, retained the audited boundaries
  on earlier implemented and in-progress candidates, and traced the pinned
  APSTA, SoftAP, DNS, and station-success behavior into the current station-only
  ESP-IDF adapter and HTTP access gate.
- Verification: Confirmed a clean synchronized branch, no open parity plan,
  native ESP-IDF mixed/AP configuration and AP-netif support, HTTP startup after
  Wi-Fi admission, and no existing captive DNS owner.
- Evidence: Pinned reference breadcrumbs, current Wi-Fi/startup/access sources,
  immutable plan, and active task record.
- Outcome: `NET-002` is bounded to software provisioning behavior and will
  remain below verified without a real provisioning client session.
- Blocker or next safe action: Commit the plan and task after mandatory gates,
  then implement the pure DNS core and thin firmware owner.
