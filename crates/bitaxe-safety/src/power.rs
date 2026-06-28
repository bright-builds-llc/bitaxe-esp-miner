//! Power, voltage, and current safety module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/power/DS4432U.c` for Ultra 205 regulator behavior.
//! - `reference/esp-miner/main/power/INA260.c` for current, voltage, and power telemetry.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for stop, cool, and restart policy.
//!
//! Plan 06-03 owns power decisions, telemetry classification, and effect planning.
//! This boundary intentionally contains no firmware side effects.

pub const MODULE_NAME: &str = "power";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/power/DS4432U.c",
    "reference/esp-miner/main/power/INA260.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];
