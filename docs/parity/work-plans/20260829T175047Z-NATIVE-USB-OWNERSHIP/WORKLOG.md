# Native USB ownership worklog

## 2026-08-29T17:50:47Z | Immutable plan

- Source base: `a0337f013b2b1db1956843d56ccdd2d003f493d7`.
- Immutable plan SHA-256:
  `9568c4eb98386f9a31d5b96079bec35df7d7a635a299ab416413cb7f5c78a807`.
- Action: created the single-PHY ownership, buttonless handoff, one-time manual
  bootstrap, durability, and repository-guardrail contract.
- Hardware/network effects: none.
- Next safe action: verify, commit, and push this plan separately, then implement
  the pre-agreed software seams without hardware effects.

## 2026-08-29T18:20:00Z | Guardrail and profile core

- Added ADR-0020, native USB ownership guidance, the always-loaded AGENTS
  pointer, profile-aware device-session types, and regression tests proving a
  Worker runtime flash selects handoff while monitoring never does.
- Hardware effects: none. Firmware handoff and host adapters remain pending.
