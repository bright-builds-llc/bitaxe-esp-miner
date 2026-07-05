#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::production_work::PoolSessionGeneration;
    use crate::v1::state::{
        HashrateInputs, MiningActivityStatus, PoolLifecycleStatus, WorkSubmissionGate,
    };

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
}
