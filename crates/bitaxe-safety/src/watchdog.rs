//! Watchdog supervision module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/tasks/power_management_task.c` for bounded power-management loop behavior.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for bounded fan-control loop behavior.
//! - `reference/esp-miner/main/self_test/self_test.c` for self-test work that must remain supervised.
//!
//! Plan 06-05 owns watchdog-friendly step progression and responsiveness contracts.
//! This boundary intentionally contains no firmware side effects.

pub const MODULE_NAME: &str = "watchdog";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/tasks/power_management_task.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/self_test/self_test.c",
];
