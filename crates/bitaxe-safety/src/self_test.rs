//! Self-test lifecycle module boundary.
//!
//! Upstream breadcrumbs:
//! - `reference/esp-miner/main/self_test/self_test.c` for factory and manual self-test lifecycle behavior.
//! - `reference/esp-miner/main/tasks/power_management_task.c` for safe blocked interaction during thermal or power faults.
//!
//! Plan 06-05 owns self-test states, effects, and result reporting.
//! This boundary intentionally contains no firmware side effects.

pub const MODULE_NAME: &str = "self_test";

pub const REFERENCE_BREADCRUMBS: &[&str] = &[
    "reference/esp-miner/main/self_test/self_test.c",
    "reference/esp-miner/main/tasks/power_management_task.c",
];
