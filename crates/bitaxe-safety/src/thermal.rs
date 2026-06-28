//! Thermal, fan, and PID safety module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/thermal/thermal.c` for sensor abstraction and sentinel values.
//! - `reference/esp-miner/main/thermal/PID.c` for controller constants and output limits.
//! - `reference/esp-miner/main/tasks/fan_controller_task.c` for fan modes and visible fan faults.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for overheat stop and cool behavior.
//!
//! Plan 06-04 owns thermal decisions, fan policy, and overheat state transitions.
//! This boundary intentionally contains no firmware side effects.

pub const MODULE_NAME: &str = "thermal";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/thermal/thermal.c",
    "reference/esp-miner/main/thermal/PID.c",
    "reference/esp-miner/main/tasks/fan_controller_task.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];
