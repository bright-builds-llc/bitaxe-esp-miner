# UI-004 worklog

## 2026-08-04T19:00:00Z | selection and plan checkpoint

- Source commit: `d93e63455e48ab512009a56b0847a4f1996625a4`.
- Actions: Ran the deterministic selector, classified earlier candidates by
  evidence and hardware availability, inspected the current static fallback,
  firmware route/static ownership, API contracts, pinned reference workflows,
  frontend standard, and Playwright prerequisites.
- Verification: The branch was clean and synchronized, the selector reported
  no open plan, the reference pin matched, and `npx` is available for the
  required real-browser verification.
- Evidence: This immutable plan and the active `TASKS.md` block.
- Outcome: `UI-004` is the first actionable row. No device, credential, external
  service, or sensitive runtime value was used.
- Blocker or next safe action: Commit the plan checkpoint, implement the scoped
  production assets and regressions, then run focused verification.

## 2026-08-04T19:20:00Z | implementation and browser checkpoint

- Source base: `60d3ef5a`.
- Actions: Replaced the recovery-only landing page with the independent operator
  shell, pure UI core, same-origin API adapter, and DOM workflow adapter. Added
  known direct-route fallback, responsive dark/light styling, write-only
  network and pool secrets, scoped settings, retained/live logs, confirmed
  commands, and confirmed firmware-only upload with OTAWWW visibly unavailable.
- Verification: Nine focused Rust static-route tests, the full `bitaxe-api`
  target, JavaScript syntax checks, and the automation static-contract suite
  pass. Bright Builds reports zero findings.
- Browser evidence: A headed Playwright CLI session loaded production assets
  from a synthetic same-origin fixture, directly entered `/network`, submitted a
  write-only password without exposing it in the resulting accessibility
  snapshot, changed theme, filtered and paused/resumed live logs, navigated pool
  and settings pages, observed the disabled no-file firmware upload, opened the
  responsive mobile menu, and finished with zero console errors or warnings.
  Temporary browser screenshots and snapshots were moved outside the repository
  after visual inspection; no origin or synthetic runtime identifier is being
  committed.
- Outcome: The scoped UI-004 implementation and browser behavior pass focused
  verification without device hardware or destructive effects.
- Blocker or next safe action: Run the exact full repository gate, simplify and
  review the diff, then bind the implementation result and typed transition.
