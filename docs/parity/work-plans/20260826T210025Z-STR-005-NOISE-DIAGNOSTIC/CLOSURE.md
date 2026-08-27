# STR-005 Noise diagnostic closure

- Plan: `20260826T210025Z-STR-005-NOISE-DIAGNOSTIC`
- Task: `task-str005-noise-handshake-diagnostic`
- Parity row: `STR-005`
- Final parity status: `implemented`
- Terminal decision: `stop_repeated_boundary`

## Outcome

The no-mining diagnostic owner, typed Noise completion classifier,
handshake-only fixture, protected host workflow, independent projection
validator, and exact recovery-006 restoration path are implemented and fully
software-verified.

Three progress-backed Ultra 205 diagnostics were run from clean pushed exact
packages. Each run used only Wi-Fi, TCP, Noise preparation, USB flashing, and
the approved exact restore path. No run initialized or worked the ASIC, changed
fan or core voltage, opened a mining channel, received a job, searched a nonce,
submitted a share, or started a campaign.

## Authoritative boundary

Diagnostic 003 supplied the decisive closed signature:

- firmware reached `tcp_connected`, `act_one_created`, and `act_one_sent`;
- firmware stopped at `act_two_read`;
- the fixture admitted the exact current device peer;
- zero unexpected peers were observed;
- the fixture received zero act-one bytes; and
- the fixture read ended by `timeout`.

This boundary occurs before the responder parses act one, creates a Noise
responder, creates act two, or exercises certificate, signature, decrypt, or
Noise completion logic. It therefore does not support another repository-side
Noise fix or an unchanged hardware ordinal. A different network transport path
or environment would require a separate formal plan.

## Safety and restoration

Every diagnostic completed the approved recovery-006 snapshot write and
separate Wi-Fi seed, then restored the exact attempt-004 settings and theme.
The final state proves the original source/app/reference/partition identity,
`mineonboot=false`, inactive mining, zero hashrate and shares, USB cleanup, and
zero owned processes. Fresh post-run Ultra 205 detection passed.

## Evidence and disposition

The independently validated final failed projection is:

`docs/parity/evidence/str005-noise-diagnostic/noise-diagnostic-projection-003.json`

Diagnostic 002 is preserved as an earlier independently validated failed
projection. Private roots remain ignored and protected. No `RESULT.md`,
hardware-regression evidence, STR-005 promotion, campaign retry, or task archive
is created. The task remains active but blocked at the repeated pre-Noise local
transport boundary.
