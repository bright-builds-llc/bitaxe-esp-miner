# Phase 3: BM1366 ASIC Protocol And Safe Initialization - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-06-26T22:51:59.597Z
**Phase:** 3 - BM1366 ASIC Protocol And Safe Initialization
**Mode:** Yolo
**Areas discussed:** BM1366 protocol surface, UART adapter boundary, safe initialization gate, parity evidence and fixture strategy

---

## BM1366 Protocol Surface

| Option | Description | Selected |
| --- | --- | --- |
| CRC/register/init-only | Lowest live-hardware risk, but leaves work/result scope underbuilt and pushes too much into Phase 4. | |
| Full pure codec plus gated adapter | Covers CRC, packet, register, init-plan, work encoding, result parsing, nonce/domain/error fixtures without live mining. | yes |
| Pure codec plus diagnostic live work smoke | Stronger work/result hardware evidence but higher hardware risk and blurs Phase 4 mining boundary. | |

**User's choice:** Yolo selected "Full pure codec plus gated adapter" as the recommended default.
**Notes:** Phase 3 should implement work/result purely now but keep runtime work submission disabled until Phase 4.

---

## UART Adapter Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Pure semantic command/observation state machine emits UART actions | Strong functional-core fit; fake UART tests can replay transcripts; raw frames stay in `bitaxe-asic`. | yes |
| Stateful generic driver over an ASIC UART trait | Ergonomic firmware calls, but mixes protocol decisions with effect sequencing. | |
| Codec-only crate with firmware-owned sequencing | Small pure crate, but raw protocol details leak into firmware and weaken typed gates. | |
| Use ESP/HAL UART traits directly | Aligns with ESP APIs, but does not fully model baud, buffer, timeout, and fail-closed semantics. | |

**User's choice:** Yolo selected "Pure semantic command/observation state machine emits UART actions" as the recommended default.
**Notes:** Firmware owns UART pins, baud changes, delays, exact reads, buffer clears, reset GPIO, and status logs.

---

## Safe Initialization Gate

| Option | Description | Selected |
| --- | --- | --- |
| Protocol-only gate | Safest when no hardware session exists, but does not satisfy live reset/UART/init evidence. | |
| Chip-detect-first staged gate | Exercises reset, UART, and BM1366 chip ID as the first live hardware smoke. | yes |
| Full staged init state machine | Matches the final Phase 3 goal, but should follow chip-detect smoke and preflight evidence. | yes |
| Direct upstream sequence behind flag | Fastest mirror, but bypasses type gates and risks unsafe effects. | |

**User's choice:** Yolo selected a staged path: chip-detect-first smoke, then full staged init only through explicit gates.
**Notes:** Missing board/config/power/thermal gates must fail closed with no mining, no work submission, and visible status.

---

## Parity Evidence And Fixture Strategy

| Option | Description | Selected |
| --- | --- | --- |
| Boundary-split fixture/evidence matrix | Separates pure protocol evidence from live hardware claims and makes GPL-derived fixture posture explicit. | yes |
| Coarse upstream-function rows with notes | Minimal checklist churn but risks ambiguous statuses and overloaded notes. | |
| Generated reference fixture harness | Useful if byte fixtures become numerous; adds tooling and GPL-derived output concerns. | |
| Hardware-capture-first evidence | Required for live init verification but does not replace unit/golden edge cases. | |

**User's choice:** Yolo selected "Boundary-split fixture/evidence matrix" as the recommended default.
**Notes:** Pure BM1366 fixtures should include source, reference commit, license posture, derivation note, and checklist IDs. Live init evidence must record board, port, command, firmware commit, reference commit, logs, observed result, and fail-closed conclusion.

---

## the agent's Discretion

- Exact Rust module names, type names, fixture schema, and plan granularity.
- Whether to introduce generated fixture extraction later if manual fixture maintenance becomes risky.
- Exact wording for disabled work-submission statuses, provided Phase 4 remains owner of production mining.

## Deferred Ideas

- Production work submission and accepted-share mining evidence belong to Phase 4.
- Broad safety controllers and verified voltage/fan/thermal/power effects belong to Phase 6.
- Generated reference fixture tooling is optional and should be justified by fixture volume.
