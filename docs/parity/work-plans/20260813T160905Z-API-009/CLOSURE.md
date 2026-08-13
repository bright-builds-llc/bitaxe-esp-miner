# Parity work closure

- Parity row: `API-009`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `092660fe0eac80ac983b6f139c6968dca46f0e387dc7c39e44bb5adc2ae8d83e`
- Active task: `task-parity-api009-command-effect-evidence-audit`

## Closure reason

Attempt-011's missing pause confirmation was caused by a lost-update race. The
HTTP pause/resume command wrote requested operator intent into the same
`MiningRuntimeState` value that the production owner replaced wholesale on
each session publication. A stale publication could restore `Run` before
authoritative readiness consumed a newer pause. The later
`safety_prerequisites_stale` deadline state was secondary.

Requested operator intent now has one separate typed boot-lifetime owner.
Boot preference and command effects update that owner, production readiness
reads it, and production session publication cannot replace it. The existing
session-derived mining projection remains the API and telemetry source, so the
fix changes ownership rather than weakening pause, safety, or evidence rules.

Behavioral tests prove a pause request survives an interleaved stale session
publication and the latest command wins. Source-ownership tests require the
distinct owner, command write, readiness read, and publication exclusion.
Focused intent, campaign-status, production-session, flash-campaign, and real
automation tests pass. The complete Cargo, Bright Builds, Bazel, parity,
privacy, reference, and real ESP firmware-build gates also pass.

API-009 remains `implemented`: this software-only plan produced no new
hardware evidence and claims no pause, IDENTIFY, dismissal, or restart quorum.

## Next safe action

A clean selector may create a separate immutable attempt-012 contract because
the exact attempt-011 blocker now has a regression-backed root-cause fix. That
contract must independently restore the exact-package, one-detector,
one-attempt, checkpoint, recovery, privacy, and promotion boundaries. This
closure does not itself authorize hardware.

## Non-claims

This closure does not verify or promote API-009, publish hardware evidence,
access credentials or protected attempt contents, interact with USB/device/
network interfaces, or authorize direct UART or pins/pads/GPIO.
