# ADR-0019: Supervise BWG restoration through a protected browser campaign

## Status

Accepted

## Context

Hardware closure must compose the Gate repository's production WebUSB adapter,
the application TinyUSB vendor function, Work Lease signing, the sole
Production Mining Session, and an Ultra 205 without turning a test harness into
a production administration API. WebUSB permission and reacquisition require a
direct browser user gesture. Challenge credentials, possession proofs, Device
Identity, and raw control frames cannot enter logs or committed evidence.

Native hardware work must also preserve the existing detector, package,
protected evidence, earliest-failure, recovery, and redaction boundaries. The
provided USB connector and barrel power are the only physical interfaces in
scope. Direct UART, pins, probes, erasure, arbitrary writes, and fault injection
remain prohibited.

## Decision

The firmware repository owns two commands:
`bwg-worker-restoration-preflight` and
`bwg-worker-restoration-campaign`. Preflight is effect-free. It consumes an
already captured protected detector result, exact clean firmware and Gate
commits, a clean six-artifact package, protected authority/pool/recovery inputs,
an absent projection, and a fresh private attempt root. It emits one
digest-bound mode-`0600` plan and performs no device, network, browser, signing,
or mining action.

The campaign command revalidates every digest and launches a loopback-only,
unguessable-path browser supervisor. Its page imports the exact built production
WebUSB adapter from the admitted Gate commit. Direct button gestures remain the
only permission and reacquisition seams. The supervisor constructs lease terms
from protected inputs, invokes the service-local Work Lease signer through
exclusive temporary mode-`0600` files, returns authorization only to live
browser memory, and deletes those temporary files after each operation.

The page records only closed event names, status categories, and restoration
reasons. Completion, Pause, Cancel, expiry, USB-only disconnect, both-power
reboot, and monotonic-uncertainty scenarios each require a baseline response
and strict browser cleanup. Physical checkpoints have no human-response
timeout; the lease and every automated transfer remain bounded. A failed
reacquisition remains retryable. Any other failure attempts controller cleanup,
preserves the earliest category, writes only private evidence, and publishes no
projection.

Pool admission requires the exact protected three-sample readiness report, its
closed bounded fields, the SHA-256 of the exact protected credential bytes,
and one stable set of loopback/RFC1918 IPv4 resolutions. Campaign startup
re-resolves that set, rejects public or changed addresses, and supplies the
admitted private IP literal to the browser. Before the browser receives a URL,
the campaign also captures at least two monotonic runtime boot attestations over
the admitted USB CDC interface. Those samples must bind the running source commit, pinned
reference commit, and application ELF SHA-256 to the admitted package. The
subsequent browser admission requires exactly one permitted vendor-function
device and its signed Ultra 205 capability. The same Device Identity possession
signature must assert the exact source commit observed over CDC, composing
runtime package identity with current-enumeration possession without relying on
USB serial authority.

An authorization-negative scenario proves expired possession context,
cross-context authorization rejection, and durable renewal replay before it can
publish success. Physical scenarios offer Reacquire from the page-owned WebUSB
checkpoint; the adapter's disconnect subscriber remains the later confirmation
that restoration handling completed.

The preflight reuses the existing package admission boundary in dry-run mode
before writing its plan. It creates one attempt-scoped authorization for the existing
settings-preserving restore tool. If browser cleanup fails, the campaign invokes
that exact recovery bundle, plan, Wi-Fi input, detector port, and protected
recovery root. Semantic failure remains the primary outcome and cleanup/recovery
is recorded separately. A failed recovery, retained signing intermediate, or
projection-finalization error keeps the campaign non-successful. Closing the
browser with an outstanding lease is itself cleanup failure: the native owner
invokes the admitted restore-installed recovery rather than treating the
beacon as proof that controller cleanup occurred.

A successful scenario stages one protected redacted projection containing exact
commit/package/bundle/event digests, terminal reason, and boolean privacy,
baseline, and cleanup facts. The campaign publishes the complete eight-scenario
set only after every scenario has one unique successful result under the same
source, Gate, package, and restore identities. A failed batch publication rolls
back every newly linked projection and remains non-successful. This keeps the
source checkout clean for exact-package admission through the first seven
scenarios. Projections never contain raw paths, ports, credentials,
challenge or lease identifiers, fingerprints, JWKs, proofs, authorizations,
pool identity, device identity, or control/evidence bytes.

The same-origin harness reads only the adapter's permitted challenge-scoped
fingerprint record after successful possession, sends it to the loopback owner,
and stores it in the protected attempt. Batch publication requires the same
fingerprint digest, Authority identity, Gate bundle/trust, pool credential and
readiness identities, package, and restore bundle across all scenarios. Public
evidence retains only `sameDeviceAcrossScenarios: true`; a separate closed
projection validator runs before atomic publication.

An ordinary reboot is not accepted as monotonic-reset evidence. Because the
current effect authorization prohibits fault injection and ad hoc device
writes, `monotonic_uncertainty` remains blocked before device effects until a
separate bounded stimulus seam is designed and explicitly authorized.

The durable authorization replay scenario is likewise blocked before effects.
An old renewal after reboot is also stale by possession context, while a fresh
Start advances the in-memory sequence first; a generic rejection cannot prove
which guard fired. Closure requires an explicitly authorized metadata-only
diagnostic that attributes durable replay without exposing authorization bytes
or adding an arbitrary signing/control surface.

## Consequences

- The hardware campaign exercises the real browser adapter rather than a
  firmware-only host client.
- No remote or production administrative API is introduced; the HTTP server is
  loopback-only, per-attempt, token-scoped, and terminates with the campaign.
- Detector admission and exact-package installation remain the existing
  firmware-owned workflows. Current-enumeration Device Identity possession is
  re-proved by the browser for every scenario.
- A successful software preflight does not claim hardware success. Only a
  completed protected campaign plus independent redaction review can support
  Ticket 07 evidence.
