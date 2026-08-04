# Parity work log

## 2026-08-04T19:29:18Z | selection and root-cause checkpoint

- Source commit: `6d42e35271d973ba1425521c152b799d24575519`.
- Actions: Resumed the open `API-010` lineage, preserved its immutable plan,
  inspected only committed flash, USB-session, firmware boot-evidence,
  classifier, and orchestration source, and derived the production multi-epoch
  transcript shape without reading the private attempt trace.
- Verification: The exact-package flash-monitor path performs a factory write
  and a credential-seed write before opening its final receive-only reader.
  Firmware emits a fresh boot identity for each reset and replays current
  identity/origin evidence, while queued intermediate bytes can precede the
  final epoch in the reader's transcript. The existing strict whole-trace
  classifier therefore returns `baseline_multiple_sessions` by design.
- Evidence: Source-derived root cause only; no private trace, credential,
  device, network, or hardware evidence was accessed or reproduced.
- Outcome: A closed terminal-epoch rule is specified in the immutable follow-up
  plan. The generic whole-trace classifier remains strict and unchanged.
- Blocker or next safe action: Run planning verification, commit and push this
  plan, then implement the synthetic classifier/orchestration regression with
  no hardware interaction.
