//! Functional scenarios using the real controller, authority and actuation core.
mod actuation;
mod authorization;
mod boundary_scenarios;
mod controller;
pub mod noise_probe;
mod resource_scenarios;
mod scenarios;
pub mod v2;

pub use scenarios::{run_scenario, run_scenario_with_observer, scenario_names};
use serde::{Deserialize, Serialize};

pub const SCENARIO_VERSION: &str = "ultra205-scenarios-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileCheck {
    pub id: String,
    pub status: CheckStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub phase: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEvent {
    pub at_ms: u64,
    pub phase: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub schema: String,
    pub scenario: String,
    pub seed: u64,
    pub checks: Vec<ProfileCheck>,
    pub expected_outcome: String,
    pub actual_outcome: String,
    pub journal: Vec<JournalEvent>,
    pub maybe_earliest_failure: Option<Failure>,
    pub cleanup_failures: Vec<Failure>,
    pub virtual_time_ms: u64,
    pub modeled_memory: bool,
    pub hardware_qualified: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SimulationError {
    #[error("unknown scenario")]
    UnknownScenario,
    #[error("scenario failed: {original}; cleanup failed: {cleanup}")]
    Cleanup {
        original: Box<SimulationError>,
        cleanup: Box<SimulationError>,
    },
    #[error("scenario boundary failed: {0}")]
    Boundary(&'static str),
    #[error("scenario encoding failed")]
    Encoding(#[from] serde_json::Error),
    #[error("functional model failed")]
    Model(#[from] bitaxe_virtual_board::ModelError),
}

/// Fixed diagnostic checkpoints; observer adapters must not allocate or alter decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ScenarioPhase {
    Entry = 1,
    BeforeController = 2,
    BeforePossession = 3,
    AfterPossession = 4,
}
