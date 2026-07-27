# Production Mining Session software verification

## Scope

This record describes deterministic software verification added after
ADR-0017. It does not alter authenticated historical checklist evidence,
promote a parity status, or claim real pool, ASIC, or hardware behavior.

## Verified in software

- Ordered readiness admission before lazy pool configuration.
- Stratum V1 framing, configure, subscribe, authorize, difficulty, notify,
  clean-jobs invalidation, version-mask handling, and work dispatch.
- Production nonce correlation and redaction-safe accepted/rejected submit
  classification with request-ID and generation checks.
- Retry budgets, recovery pause, configured fallback preference, automatic
  fallback, non-disruptive primary probes, and primary restoration under
  virtual time.
- Network, settings, safety, operator-intent, and shutdown handling.
- Ordered, idempotent safe stop and immutable final session publication.
- Ordinary ESP firmware compilation with actuation qualification fixed false
  and secret, network, socket-write, and ASIC effects unreachable.

## Explicit non-claims

- No pool credentials were read.
- No real pool connection or TCP/TLS behavior was exercised.
- No work was sent to an ASIC and no nonce was observed from hardware.
- No device was flashed, mined, or actuated.
- Deterministic accepted and rejected shares are not hardware evidence.
- Existing parity checklist statuses and historical evidence remain unchanged.
