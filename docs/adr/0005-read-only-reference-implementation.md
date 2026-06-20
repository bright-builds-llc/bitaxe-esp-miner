# Keep the Reference Implementation Read-Only

The upstream ESP-Miner submodule will live at `reference/esp-miner`, stay pinned to an explicit upstream commit, and be treated as read-only evidence. Project verification should fail when local modifications exist inside the submodule; only explicit submodule pointer updates are allowed, and those updates should be documented as reference refreshes so parity work can distinguish baseline changes from Rust firmware changes.
