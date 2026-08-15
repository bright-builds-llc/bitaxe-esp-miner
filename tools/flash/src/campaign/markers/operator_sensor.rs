use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum OperatorSensorStageMarker {
    None,
    Power,
    AsicTemperature,
    Tachometer,
    CoreVoltage,
    Display,
    Actuation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum OperatorSensorOutcomeMarker {
    None,
    Ready,
    Recovered,
    DriverFailed,
    BudgetExhausted,
    SampleInvalid,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::campaign) enum OperatorSensorDurationBucketMarker {
    None,
    #[serde(rename = "under_100_ms")]
    Under100Ms,
    #[serde(rename = "under_250_ms")]
    Under250Ms,
    #[serde(rename = "under_500_ms")]
    Under500Ms,
    #[serde(rename = "under_1000_ms")]
    Under1000Ms,
    #[serde(rename = "at_least_1000_ms")]
    AtLeast1000Ms,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::campaign) struct OperatorSensorDiagnosticMarker {
    pub(in crate::campaign) available: bool,
    pub(in crate::campaign) boot_session: u64,
    pub(in crate::campaign) revision: u64,
    pub(in crate::campaign) stage: OperatorSensorStageMarker,
    pub(in crate::campaign) outcome: OperatorSensorOutcomeMarker,
    pub(in crate::campaign) duration_bucket: OperatorSensorDurationBucketMarker,
}

impl OperatorSensorDiagnosticMarker {
    pub(super) const fn is_valid(self) -> bool {
        if self.available {
            return self.boot_session != 0
                && self.revision != 0
                && !matches!(self.stage, OperatorSensorStageMarker::None)
                && !matches!(self.outcome, OperatorSensorOutcomeMarker::None)
                && !matches!(
                    self.duration_bucket,
                    OperatorSensorDurationBucketMarker::None
                );
        }
        self.boot_session == 0
            && self.revision == 0
            && matches!(self.stage, OperatorSensorStageMarker::None)
            && matches!(self.outcome, OperatorSensorOutcomeMarker::None)
            && matches!(
                self.duration_bucket,
                OperatorSensorDurationBucketMarker::None
            )
    }
}
