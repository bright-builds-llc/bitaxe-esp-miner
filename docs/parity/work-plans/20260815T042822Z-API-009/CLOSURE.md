# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `9f0b46f459be5385961aa489525c42361f328effe7338dd18f3a8cd473d66cbb`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

Pushed source `697688f0cda007ed43da380af06e32fa512a4fa2` fixes the
attempt-026 preservation boundary. The first active notification still proves
genuine command-effect provenance. After the one pause request joins both HTTP
paused state and serial hardware safe stop, the host now requires and captures
the current positive cumulative count immediately before issuing the sole
dismissal request. The first cleared readback must preserve that paused count
before IDENTIFY readiness can be armed.

The live-shaped regression starts with notification count seven, observes
count eight after pause convergence, and then clears the notification while
preserving eight. It failed against the prior source because the host compared
the cleared readback with seven, and passes after the repair. A separate test
proves a zero paused count fails before any dismissal request.

All required software gates pass and the implementation checkpoint is pushed.
This software-only plan prohibited hardware access and API-009 promotion, so
no live device evidence exists for the repaired boundary and the row correctly
remains `implemented`.

## Next safe action

Create and push a fresh immutable attempt-027 plan bound to source
`697688f0cda007ed43da380af06e32fa512a4fa2`. After its exact command, package,
detector, privacy, recovery, retry, and stop gates pass, run at most one new
detector-gated Ultra 205 campaign. Promote API-009 only if the complete sealed
command/restart/device-user quorum passes; otherwise preserve its earliest
typed failure, safe stop, cleanup, evidence withholding, and no automatic
retry.

## Non-claims

This closure does not claim live notification dismissal, block-count
preservation, IDENTIFY display behavior, active resume, terminal HTTP truth,
accepted share behavior, canonical restart, same-device recovery, public
parity evidence, or API-009 verification. It exposes no origin, hostname,
port, USB/network identity, credential, worker, address, password, token,
sensor value, or raw trace.
