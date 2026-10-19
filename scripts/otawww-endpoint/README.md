# Station endpoint handoff

`just otawww-endpoint preflight|serve|finish` serves the pinned Gate on
`127.0.0.1:48765` in endpoint-handoff mode. After an admitted Connect it reads
the Worker's station endpoint once, stores it privately as
`endpoint.private.json`, closes the Worker and leaves the page. Each use still
needs an active task contract.

## Attempts

One server allows up to five attempts. Each failed Connect or endpoint read
is recorded and is followed by a reload:

- The page writes `attempt-NNN.json` with value-free categories only: the step,
  the Gate's status, failure, serial category and admission stage, the
  error's closed categories, the granted and attached Worker port counts and
  the foreground flags. The server prints one `endpoint_attempt_failed=` line.
- The page closes the Worker and reloads into a fresh Gate document. Each
  activation gets a fresh challenge and each attempt a fresh nonce, as with a
  fresh handoff. Click Connect again when the page asks.
- After the fifth failure the page records the closed Worker if it can and
  leaves. `finish` then fails with `endpoint_not_recorded` or
  `endpoint_page_not_closed`, and `result.json` keeps the attempt count.

The page also prevents two known failures:

- **Port not attached:** while a granted Worker port is not exactly one
  attached port (USB re-enumeration after an install or reset), the page holds
  the click. Otherwise the Gate opens Chrome's port chooser, which a click
  cannot answer, and admission fails at stage `permission` as
  `operation_failed`. Click again once the device has re-enumerated.
- **Endpoint read during Connect:** the endpoint read waits until the Gate
  publishes its ready state. Connect marks the Gate ready before its own status
  read settles, and a second request in that window loses the session.
