# Worker port reuse amendment

## Status and precedence

Owner-authorized on 2026-10-02 under `task-worker-port-reuse`. This amendment
supersedes one sentence of the
[V2 serial permission amendment](str005-v2-serial-permission-amendment.md): its
prohibition on "a getPorts fallback" and on automatically selecting a permitted
port. Every other requirement there still applies, including the trusted click,
visible and focused document and active user activation before Connect. The
earlier amendment's text and every historical result stay unchanged.

## Decision

Gate commit `8b835c2c09148ec946396cfb4660222a92853c81` implements
[ADR-0101](https://github.com/bright-builds-llc/bitaxe-turnstile-system/blob/8b835c2c09148ec946396cfb4660222a92853c81/docs/adr/0101-reuse-a-single-granted-worker-port.md).
On Connect, the Gate reuses exactly one already granted, attached port that
matches the Worker vendor and product IDs, without showing Chrome's port chooser.
No grant, several grants or an unavailable listing fall back to `requestPort()`
with the same filter.

Selection is never authority. This matches
[native USB ownership](native-usb-ownership.md), where USB identifiers are
discovery hints, and ADR-0021/0022, where authority comes from a fresh Hello and
possession. The following still apply to every Connect:
- the native Connect gesture;
- foreground and activation checks;
- the Web Lock;
- the exact device-filter check;
- a fresh Hello with exact Device Identity possession;
- every failure category.

## Operation

With one granted Ultra 205, a qualification run needs no person at the chooser.
The agent or owner still clicks **Connect Worker** with a real input event while
the page is visible, and the agent verifies visibility read-only before each
click. Granting the port the first time, or choosing between several granted
Workers, still uses the chooser. Revoking the site's serial permission in Chrome
restores the chooser for every Connect.

## Verification

Gate tests cover reuse of a single grant without a chooser, the chooser fallback
for ambiguous, detached, foreign or unlisted grants, a cancelled chooser, and a
reused grant passing fresh Hello, possession and the maximum probe. A read-only
browser check confirmed that Chrome kept this origin's grant for the Ultra 205
(`303a:1001`) across attempt-003's five reflashes. No device effect was part of
this amendment.
