#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::messages::PoolDifficulty;
    use bitaxe_stratum::v1::production_work::PoolSessionGeneration;
    use bitaxe_stratum::v1::state::{
        HashrateInputs, MiningRuntimeState, PoolLifecycleStatus, ShareDifficulty,
    };
    use bitaxe_stratum::v1::submit_response::{
        RedactedSubmitRejectReason, SubmitClassification,
    };
    use bitaxe_stratum::v1::telemetry_projection::{
        RuntimeProjectionSampleMarker, RuntimeProjectionSampleSource, RuntimeTelemetryEvent,
        RuntimeTelemetryProjection, RuntimeTelemetrySequence,
    };
    use serde_json::Value;

    use super::{project_api_views, ProjectedApiViews};
    use crate::{
        ApiSnapshot, LiveTelemetryPlanner, SystemInfoWire, WebSocketRouteKind, WebSocketState,
    };

    #[test]
    fn projection_system_info_preserves_axeos_fields() {
        // Arrange
        let projection = active_projection_with_share_counters();
        let base = ApiSnapshot::safe_ultra_205();

        // Act
        let views = project_api_views(base, &projection, None, 10_000, 12.5);
        let wire = SystemInfoWire::from_snapshot(&views.snapshot);
        let public_json = views.telemetry_payload;

        // Assert
        assert_eq!(wire.hash_rate, 2_500.0);
        assert_eq!(wire.shares_accepted, 1);
        assert_eq!(wire.shares_rejected, 1);
        assert_eq!(wire.best_diff, 128.0);
        assert_eq!(wire.pool_difficulty, 4_096.0);
        assert_eq!(wire.pool_connection_info, "active");
        assert_eq!(wire.is_using_fallback_stratum, 1);
        assert!(!wire.mining_paused);
        assert_eq!(public_json.get("hashRate"), Some(&Value::from(2_500.0)));
        assert_eq!(public_json.get("sharesAccepted"), Some(&Value::from(1)));
    }

    #[test]
    fn projection_statistics_empty_without_bounded_sample() {
        // Arrange
        let projection = active_projection_with_share_counters();

        // Act
        let views = project_api_views(
            ApiSnapshot::safe_ultra_205(),
            &projection,
            None,
            20_000,
            5.0,
        );

        // Assert
        assert!(views.statistics_samples.is_empty());
    }

    #[test]
    fn projection_statistics_ignores_repeated_request_time_reads_without_sample_marker() {
        // Arrange
        let projection = active_projection_with_share_counters();

        // Act
        let first = project_api_views(
            ApiSnapshot::safe_ultra_205(),
            &projection,
            None,
            20_000,
            5.0,
        );
        let second = project_api_views(
            ApiSnapshot::safe_ultra_205(),
            &projection,
            None,
            20_500,
            6.0,
        );
        let marker = RuntimeProjectionSampleMarker {
            sequence: RuntimeTelemetrySequence::new(90),
            timestamp_ms: 21_000,
            source: RuntimeProjectionSampleSource::RuntimeEvent,
        };
        let bounded = project_api_views(
            ApiSnapshot::safe_ultra_205(),
            &projection,
            Some(marker),
            21_500,
            7.0,
        );

        // Assert
        assert!(first.statistics_samples.is_empty());
        assert!(second.statistics_samples.is_empty());
        assert_eq!(bounded.statistics_samples.len(), 1);
        assert_eq!(bounded.statistics_samples[0].timestamp, marker.timestamp_ms);
        assert_eq!(bounded.statistics_samples[0].response_time, 7.0);
    }

    #[test]
    fn projection_scoreboard_empty_without_parsed_share_outcome() {
        // Arrange
        let states = [
            blocked_projection("blocked_safe_prerequisite"),
            blocked_projection("fake_pool_only"),
            stale_generation_projection(),
            stopped_projection(),
            RuntimeTelemetryProjection::new(PoolSessionGeneration::initial()),
        ];

        // Act
        let projected_scoreboards = states
            .iter()
            .map(|projection| {
                project_api_views(
                    ApiSnapshot::safe_ultra_205(),
                    projection,
                    None,
                    30_000,
                    0.0,
                )
                .scoreboard_entries
            })
            .collect::<Vec<_>>();

        // Assert
        for entries in projected_scoreboards {
            assert!(entries.is_empty());
        }
    }

    #[test]
    fn projection_live_telemetry_safe_stop_not_active() {
        // Arrange
        let active_projection = active_projection_with_share_counters();
        let stopped_projection = stopped_projection();
        let active_views =
            project_api_views(ApiSnapshot::safe_ultra_205(), &active_projection, None, 1, 0.0);
        let stopped_views =
            project_api_views(ApiSnapshot::safe_ultra_205(), &stopped_projection, None, 2, 0.0);
        let mut websocket = WebSocketState::default();
        let _registered = websocket.register_client(1, WebSocketRouteKind::LiveTelemetry);
        let _connect = websocket.live_connect_frame(active_views.telemetry_payload);

        // Act
        let cadence = websocket
            .live_cadence_frame(stopped_views.telemetry_payload)
            .expect("safe stop should produce a cadence diff");
        let rendered = cadence.to_string();

        // Assert
        assert_eq!(cadence["event"], "update");
        assert_eq!(cadence["data"]["miningPaused"], Value::Bool(true));
        assert_eq!(cadence["data"]["poolConnectionInfo"], Value::String("disconnected".into()));
        assert!(!rendered.contains(":\"active\""));
    }

    #[test]
    fn projection_redaction_denylist_fields_stay_out_of_public_json() {
        // Arrange
        let projection = active_projection_with_share_counters();
        let denied_fields = [
            "sourceLabel",
            "evidenceTier",
            "redactionStatus",
            "poolURL",
            "poolUser",
            "poolPassword",
            "device_url",
            "raw_bm1366_frame",
        ];

        // Act
        let ProjectedApiViews {
            telemetry_payload, ..
        } = project_api_views(ApiSnapshot::safe_ultra_205(), &projection, None, 1, 0.0);

        // Assert
        for field in denied_fields {
            assert!(telemetry_payload.get(field).is_none());
        }
    }

    fn active_projection_with_share_counters() -> RuntimeTelemetryProjection {
        let generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(generation);
        let _hashrate = projection.fold(RuntimeTelemetryEvent::HashrateObserved {
            sequence: RuntimeTelemetrySequence::new(1),
            inputs: HashrateInputs {
                hashes_done: 2_500_000_000_000,
                elapsed_ms: 1_000,
                rolling_hashrate_hs: 2_500_000_000_000.0,
            },
        });
        let _difficulty = projection.fold(RuntimeTelemetryEvent::PoolDifficultyObserved {
            sequence: RuntimeTelemetrySequence::new(2),
            difficulty: PoolDifficulty {
                difficulty: 4_096.0,
            },
        });
        let _fallback = projection.fold(RuntimeTelemetryEvent::FallbackChanged {
            sequence: RuntimeTelemetrySequence::new(3),
            active: true,
        });
        let _lifecycle = projection.fold(RuntimeTelemetryEvent::LifecycleChanged {
            sequence: RuntimeTelemetrySequence::new(4),
            lifecycle: PoolLifecycleStatus::Active,
        });
        let _ready = projection.fold(RuntimeTelemetryEvent::WorkSubmissionReady {
            sequence: RuntimeTelemetrySequence::new(5),
        });
        let _accepted = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(6),
            generation,
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: Some(ShareDifficulty::new(128.0)),
        });
        let _rejected = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(7),
            generation,
            classification: SubmitClassification::Rejected {
                reason: RedactedSubmitRejectReason::PoolRejectedShare,
            },
            maybe_share_difficulty: None,
        });
        projection
    }

    fn blocked_projection(reason: &'static str) -> RuntimeTelemetryProjection {
        let mut projection = RuntimeTelemetryProjection::new(PoolSessionGeneration::initial());
        let _blocked = projection.fold(RuntimeTelemetryEvent::Blocked {
            sequence: RuntimeTelemetrySequence::new(1),
            reason,
        });
        projection
    }

    fn stale_generation_projection() -> RuntimeTelemetryProjection {
        let generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(generation);
        let _stale = projection.fold(RuntimeTelemetryEvent::SubmitClassified {
            sequence: RuntimeTelemetrySequence::new(1),
            generation: generation.next(),
            classification: SubmitClassification::Accepted,
            maybe_share_difficulty: Some(ShareDifficulty::new(64.0)),
        });
        projection
    }

    fn stopped_projection() -> RuntimeTelemetryProjection {
        let generation = PoolSessionGeneration::initial();
        let mut projection = RuntimeTelemetryProjection::new(generation);
        let _ready = projection.fold(RuntimeTelemetryEvent::WorkSubmissionReady {
            sequence: RuntimeTelemetrySequence::new(1),
        });
        let _stopped = projection.fold(RuntimeTelemetryEvent::SafeStopped {
            sequence: RuntimeTelemetrySequence::new(2),
            reason: "phase25_safe_stop",
        });
        projection
    }
}
