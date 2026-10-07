# Target AxeOS API and Asset Compatibility First

Status: Superseded for the web UI by [ADR-0034](0034-web-ui-variants-and-build-flag.md). API, WebSocket, OTA, recovery and static asset compatibility remain in force; the project now ships its own web UI variants instead of leaving the UI out of scope.

The Rust firmware project includes AxeOS HTTP API, WebSocket, OTA, recovery, and static asset packaging compatibility, but it does not initially rewrite the Angular AxeOS web UI. The existing UI remains a reference/client compatibility target so early Rust firmware work stays focused on device-user parity instead of combining a firmware rewrite with a frontend rewrite.
