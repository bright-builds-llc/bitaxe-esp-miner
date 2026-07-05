use crate::v1::messages::PoolDifficulty;
use crate::v1::production_work::PoolSessionGeneration;
use crate::v1::state::{
    HashrateInputs, MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus,
};
use crate::v1::submit_response::SubmitClassification;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeTelemetrySequence(u64);

impl RuntimeTelemetrySequence {
    #[must_use]
    pub const fn new(sequence: u64) -> Self {
        Self(sequence)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeProjectionSampleSource {
    RuntimeEvent,
    SafeStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeProjectionSampleMarker {
    pub sequence: RuntimeTelemetrySequence,
    pub timestamp_ms: u64,
    pub source: RuntimeProjectionSampleSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeTelemetryEvent {
    LifecycleChanged {
        sequence: RuntimeTelemetrySequence,
        lifecycle: PoolLifecycleStatus,
    },
    PoolDifficultyObserved {
        sequence: RuntimeTelemetrySequence,
        difficulty: PoolDifficulty,
    },
    HashrateObserved {
        sequence: RuntimeTelemetrySequence,
        inputs: HashrateInputs,
    },
    FallbackChanged {
        sequence: RuntimeTelemetrySequence,
        active: bool,
    },
    WorkSubmissionReady {
        sequence: RuntimeTelemetrySequence,
    },
    Blocked {
        sequence: RuntimeTelemetrySequence,
        reason: &'static str,
    },
    BoundedSampleReady {
        sequence: RuntimeTelemetrySequence,
        timestamp_ms: u64,
        source: RuntimeProjectionSampleSource,
    },
    SubmitClassified {
        sequence: RuntimeTelemetrySequence,
        generation: PoolSessionGeneration,
        classification: SubmitClassification,
    },
    SafeStopped {
        sequence: RuntimeTelemetrySequence,
        reason: &'static str,
    },
}

impl RuntimeTelemetryEvent {
    #[must_use]
    pub const fn sequence(&self) -> RuntimeTelemetrySequence {
        match self {
            Self::LifecycleChanged { sequence, .. }
            | Self::PoolDifficultyObserved { sequence, .. }
            | Self::HashrateObserved { sequence, .. }
            | Self::FallbackChanged { sequence, .. }
            | Self::WorkSubmissionReady { sequence }
            | Self::Blocked { sequence, .. }
            | Self::BoundedSampleReady { sequence, .. }
            | Self::SubmitClassified { sequence, .. }
            | Self::SafeStopped { sequence, .. } => *sequence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionShareOutcome {
    Accepted,
    Rejected,
    NoCounterChange,
    IgnoredStaleSequence,
    IgnoredStaleGeneration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTelemetryProjection {
    last_sequence: RuntimeTelemetrySequence,
    current_generation: PoolSessionGeneration,
    maybe_pending_sample_marker: Option<RuntimeProjectionSampleMarker>,
    state: MiningRuntimeState,
}

impl RuntimeTelemetryProjection {
    #[must_use]
    pub fn new(generation: PoolSessionGeneration) -> Self {
        Self {
            last_sequence: RuntimeTelemetrySequence::new(0),
            current_generation: generation,
            maybe_pending_sample_marker: None,
            state: MiningRuntimeState::default(),
        }
    }

    #[must_use]
    pub const fn state(&self) -> &MiningRuntimeState {
        &self.state
    }

    #[must_use]
    pub const fn current_generation(&self) -> PoolSessionGeneration {
        self.current_generation
    }

    pub fn fold(&mut self, event: RuntimeTelemetryEvent) -> ProjectionShareOutcome {
        let sequence = event.sequence();
        if sequence <= self.last_sequence {
            return ProjectionShareOutcome::IgnoredStaleSequence;
        }
        self.last_sequence = sequence;

        match event {
            RuntimeTelemetryEvent::LifecycleChanged { lifecycle, .. } => {
                self.state.set_lifecycle(lifecycle);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::PoolDifficultyObserved { difficulty, .. } => {
                self.state.set_pool_difficulty(difficulty);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::HashrateObserved { inputs, .. } => {
                self.state.record_hashrate_inputs(inputs);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::FallbackChanged { active, .. } => {
                self.state.set_fallback_active(active);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::WorkSubmissionReady { .. } => {
                self.state.allow_work_submission();
                self.state.set_mining_activity(MiningActivityStatus::Active);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::Blocked { reason, .. } => {
                self.state.block_work_submission(reason);
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::BoundedSampleReady {
                sequence,
                timestamp_ms,
                source,
            } => {
                self.maybe_pending_sample_marker = Some(RuntimeProjectionSampleMarker {
                    sequence,
                    timestamp_ms,
                    source,
                });
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::SubmitClassified { generation, .. } => {
                if generation != self.current_generation {
                    return ProjectionShareOutcome::IgnoredStaleGeneration;
                }
                ProjectionShareOutcome::NoCounterChange
            }
            RuntimeTelemetryEvent::SafeStopped { reason, .. } => {
                self.current_generation = self.current_generation.next();
                self.state.set_lifecycle(PoolLifecycleStatus::Disconnected);
                self.state.block_work_submission(reason);
                ProjectionShareOutcome::NoCounterChange
            }
        }
    }

    pub fn drain_pending_sample_marker(&mut self) -> Option<RuntimeProjectionSampleMarker> {
        self.maybe_pending_sample_marker.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::production_work::PoolSessionGeneration;
    use crate::v1::state::{
        HashrateInputs, MiningActivityStatus, PoolLifecycleStatus, ShareDifficulty,
        WorkSubmissionGate,
    };
    use crate::v1::submit_response::{RedactedSubmitRejectReason, SubmitClassification};

    #[test]
    fn projection_lifecycle_event_sets_active_without_counter_changes() {
        // Arrange
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::LifecycleChanged {
            sequence: RuntimeTelemetrySequence::new(1),
            lifecycle: PoolLifecycleStatus::Active,
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::NoCounterChange);
        assert_eq!(projection.state().lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 0);
    }

    #[test]
    fn projection_hashrate_event_updates_inputs_without_counter_changes() {
        // Arrange
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());
        let inputs = HashrateInputs {
            hashes_done: 2_048,
            elapsed_ms: 4_096,
            rolling_hashrate_hs: 512.0,
        };

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::HashrateObserved {
            sequence: RuntimeTelemetrySequence::new(2),
            inputs,
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::NoCounterChange);
        assert_eq!(projection.state().hashrate_inputs, inputs);
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 0);
    }

    #[test]
    fn projection_blocked_event_sets_safe_blocked_status_and_exact_reason() {
        // Arrange
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::Blocked {
            sequence: RuntimeTelemetrySequence::new(3),
            reason: "phase25_safe_stop",
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::NoCounterChange);
        assert_eq!(
            projection.state().work_submission,
            WorkSubmissionGate::Blocked
        );
        assert_eq!(
            projection.state().mining_activity,
            MiningActivityStatus::SafeBlocked
        );
        assert_eq!(
            projection.state().maybe_blocked_reason,
            Some("phase25_safe_stop")
        );
    }

    #[test]
    fn projection_sample_marker_drains_once_per_runtime_boundary() {
        // Arrange
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::BoundedSampleReady {
            sequence: RuntimeTelemetrySequence::new(4),
            timestamp_ms: 12_345,
            source: RuntimeProjectionSampleSource::RuntimeEvent,
        });
        let maybe_marker = projection.drain_pending_sample_marker();
        let maybe_second_marker = projection.drain_pending_sample_marker();

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::NoCounterChange);
        assert_eq!(
            maybe_marker,
            Some(RuntimeProjectionSampleMarker {
                sequence: RuntimeTelemetrySequence::new(4),
                timestamp_ms: 12_345,
                source: RuntimeProjectionSampleSource::RuntimeEvent,
            })
        );
        assert_eq!(maybe_second_marker, None);
    }

    #[test]
    fn projection_repeated_request_reads_do_not_create_statistics_samples() {
        // Arrange
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());

        // Act
        let first_read = projection.drain_pending_sample_marker();
        let second_read = projection.drain_pending_sample_marker();

        // Assert
        assert_eq!(first_read, None);
        assert_eq!(second_read, None);
    }

    #[test]
    fn projection_debug_output_uses_redaction_safe_labels_only() {
        // Arrange
        let event = RuntimeTelemetryEvent::Blocked {
            sequence: RuntimeTelemetrySequence::new(5),
            reason: "phase25_safe_stop",
        };
        let projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());
        let redaction_denylist = [
            "stratum+tcp://",
            "poolURL",
            "poolUser",
            "poolPassword",
            "target=",
            "extranonce=",
            "share_payload",
            "socket_error",
            "device_url",
            "wifi",
            "NVS",
            "raw_bm1366_frame",
        ];

        // Act
        let rendered_event = format!("{event:?}");
        let rendered_projection = format!("{projection:?}");

        // Assert
        assert!(rendered_event.contains("phase25_safe_stop"));
        for denied in redaction_denylist {
            assert!(!rendered_event.contains(denied));
            assert!(!rendered_projection.contains(denied));
        }
    }

    #[test]
    fn projection_advances_accepted_counter_only_for_current_generation() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(current_generation);

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(6),
            generation: current_generation,
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: Some(ShareDifficulty::new(128.0)),
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::Accepted);
        assert_eq!(projection.state().counters.accepted, 1);
        assert_eq!(projection.state().counters.rejected, 0);
        assert_eq!(
            projection.state().counters.maybe_best_difficulty,
            Some(ShareDifficulty::new(128.0))
        );
    }

    #[test]
    fn projection_advances_rejected_counter_with_redacted_reason() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(current_generation);

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(7),
            generation: current_generation,
            classification: SubmitClassification::Rejected {
                reason: RedactedSubmitRejectReason::PoolRejectedShare,
            },
            maybe_share_difficulty: None,
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::Rejected);
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 1);
        assert_eq!(
            projection.state().counters.rejected_reasons,
            vec!["pool_rejected_share".to_owned()]
        );
    }

    #[test]
    fn projection_does_not_advance_counters_for_stale_generation() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let stale_generation = current_generation.next();
        let mut projection = RuntimeTelemetryProjection::new(current_generation);

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(8),
            generation: stale_generation,
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: Some(ShareDifficulty::new(64.0)),
        });

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::IgnoredStaleGeneration);
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 0);
    }

    #[test]
    fn projection_does_not_advance_counters_for_non_share_classifications() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let classifications = [
            SubmitClassification::NoObservedShare,
            SubmitClassification::Timeout,
            SubmitClassification::Reconnect,
            SubmitClassification::Malformed,
            SubmitClassification::Blocked {
                reason: "stale_generation",
            },
            SubmitClassification::Blocked {
                reason: "submit_intent_missing",
            },
            SubmitClassification::Blocked {
                reason: "blocked_safe_prerequisite",
            },
            SubmitClassification::Stopped,
        ];
        let mut projection = RuntimeTelemetryProjection::new(current_generation);

        // Act
        for (offset, classification) in classifications.into_iter().enumerate() {
            let _outcome = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
                sequence: RuntimeTelemetrySequence::new(9 + offset as u64),
                generation: current_generation,
                classification,
                maybe_share_difficulty: Some(ShareDifficulty::new(256.0)),
            });
        }

        // Assert
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 0);
    }

    #[test]
    fn projection_safe_stop_prevents_stale_active_mining() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(current_generation);
        let _ready = projection.fold(RuntimeTelemetryEvent::WorkSubmissionReady {
            sequence: RuntimeTelemetrySequence::new(17),
        });

        // Act
        let safe_stop = projection.fold(RuntimeTelemetryEvent::SafeStopped {
            sequence: RuntimeTelemetrySequence::new(18),
            reason: "phase25_safe_stop",
        });
        let lower_sequence = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(17),
            generation: current_generation,
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: Some(ShareDifficulty::new(1.0)),
        });
        let stale_generation = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(19),
            generation: current_generation,
            classification: SubmitClassification::Rejected {
                reason: RedactedSubmitRejectReason::Unknown,
            },
            maybe_share_difficulty: None,
        });

        // Assert
        assert_eq!(safe_stop, ProjectionShareOutcome::NoCounterChange);
        assert_eq!(lower_sequence, ProjectionShareOutcome::IgnoredStaleSequence);
        assert_eq!(
            stale_generation,
            ProjectionShareOutcome::IgnoredStaleGeneration
        );
        assert_eq!(projection.state().lifecycle, PoolLifecycleStatus::Disconnected);
        assert_eq!(
            projection.state().mining_activity,
            MiningActivityStatus::SafeBlocked
        );
        assert_eq!(
            projection.state().work_submission,
            WorkSubmissionGate::Blocked
        );
        assert_eq!(projection.state().counters.accepted, 0);
        assert_eq!(projection.state().counters.rejected, 0);
    }

    #[test]
    fn projection_submit_classification_does_not_emit_request_time_sample_marker() {
        // Arrange
        let current_generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(current_generation);

        // Act
        let outcome = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(20),
            generation: current_generation,
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: None,
        });
        let maybe_marker = projection.drain_pending_sample_marker();

        // Assert
        assert_eq!(outcome, ProjectionShareOutcome::Accepted);
        assert_eq!(projection.state().counters.accepted, 1);
        assert_eq!(
            projection.state().counters.maybe_best_difficulty,
            Some(ShareDifficulty::new(0.0))
        );
        assert_eq!(maybe_marker, None);
    }
}
