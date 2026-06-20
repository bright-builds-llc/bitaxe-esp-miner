# Target AxeOS API and Asset Compatibility First

The Rust firmware project includes AxeOS HTTP API, WebSocket, OTA, recovery, and static asset packaging compatibility, but it does not initially rewrite the Angular AxeOS web UI. The existing UI remains a reference/client compatibility target so early Rust firmware work stays focused on device-user parity instead of combining a firmware rewrite with a frontend rewrite.
