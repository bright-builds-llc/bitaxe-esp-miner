#[cfg(test)]
mod tests {
    const API_IDENTITY_SOURCE: &str =
        include_str!("../../../crates/bitaxe-api/src/operator_snapshot.rs");
    const BOOT_EVIDENCE_SOURCE: &str =
        include_str!("../../../firmware/bitaxe/src/boot_evidence.rs");
    const RUNTIME_SNAPSHOT_SOURCE: &str =
        include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");

    #[test]
    fn phase34_operator_snapshot_runtime_source_guard() {
        // Arrange
        let capture = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "pub fn collect_api_snapshot",
            "/// Returns the current command-visible mining state.",
        );
        let reservation = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn reserve_operator_snapshot_identity",
            "fn retain_completed_operator_snapshot",
        );
        let retention = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn retain_completed_operator_snapshot",
            "fn mutate_command_visible_state_with_result",
        );

        // Act / Assert
        assert_eq!(BOOT_EVIDENCE_SOURCE.matches("esp_random()").count(), 4);
        assert_eq!(
            BOOT_EVIDENCE_SOURCE.matches("static BOOT_SESSION:").count(),
            1
        );
        assert!(BOOT_EVIDENCE_SOURCE.contains("operator_snapshot_boot_session"));
        assert!(BOOT_EVIDENCE_SOURCE.contains("BootSessionId::from_words(boot_session().0)"));

        assert_eq!(
            RUNTIME_SNAPSHOT_SOURCE
                .matches("static OPERATOR_SNAPSHOT_SEQUENCE:")
                .count(),
            1
        );
        assert_eq!(
            RUNTIME_SNAPSHOT_SOURCE.matches(".next_identity(").count(),
            1
        );
        assert_eq!(
            RUNTIME_SNAPSHOT_SOURCE
                .matches("snapshot.operator_snapshot_identity = operator_snapshot_identity")
                .count(),
            1
        );
        assert!(RUNTIME_SNAPSHOT_SOURCE.contains(
            "static OPERATOR_SNAPSHOT_SEQUENCE: OnceLock<Mutex<OperatorSnapshotSequence>>"
        ));
        assert!(reservation.contains("operator_snapshot_boot_session()"));
        assert!(reservation.contains("cannot exhaust within one boot"));

        let reservation_index = capture
            .find("reserve_operator_snapshot_identity")
            .expect("capture must reserve one identity");
        let fixture_index = capture
            .find("ApiSnapshot::safe_ultra_205")
            .expect("capture must collect the base snapshot");
        let retention_index = capture
            .find("retain_completed_operator_snapshot")
            .expect("completed capture must retain its marker");
        assert!(reservation_index < fixture_index);
        assert!(fixture_index < retention_index);
        assert!(retention.contains("identity.retained_marker()"));
        assert!(retention.contains("append_runtime_log_line(&marker)"));
        assert!(
            API_IDENTITY_SOURCE.contains("operator_snapshot session={} revision={} redacted=true")
        );
        assert!(API_IDENTITY_SOURCE
            .contains("shared_sequence_assigns_unique_revisions_to_concurrent_callers"));

        for forbidden in [
            "esp_random",
            "SystemTime",
            "firmware_commit",
            "app_elf_sha256",
            "mac_addr",
            "fixture_only",
        ] {
            assert!(
                !reservation.contains(forbidden),
                "reservation contains forbidden fallback {forbidden}"
            );
        }
    }

    fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
        let start_index = source.find(start).expect("start marker should exist");
        let tail = &source[start_index..];
        let end_index = tail.find(end).expect("end marker should exist");
        &tail[..end_index]
    }
}
