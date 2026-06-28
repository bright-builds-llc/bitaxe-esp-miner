//! Safety fault-policy module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/tasks/power_management_task.c` for overheat and power safe-stop policy.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for fan set failure and visible fault behavior.
//! - `reference/esp-miner/main/thermal/thermal.c` for unavailable or invalid temperature observations.
//!
//! Plan 06-04 owns fault classification and fail-closed policy composition.
//! This boundary intentionally contains no firmware side effects.

pub const MODULE_NAME: &str = "fault";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/tasks/power_management_task.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/thermal/thermal.c",
];
