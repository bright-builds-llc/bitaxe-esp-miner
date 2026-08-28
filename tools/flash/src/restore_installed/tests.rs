use super::*;

#[test]
fn restore_ranges_exclude_nvs_and_coredump_storage() {
    // Arrange
    let nvs = 0x009000..0x00f000;
    let coredump_start = 0xf12000;

    // Act / Assert
    assert_eq!(RESTORE_RANGES.len(), 8);
    for (_, address, size) in RESTORE_RANGES {
        let end = address + size;
        assert!(address >= nvs.end || end <= nvs.start);
        assert!(end <= coredump_start);
    }
}

#[test]
fn diagnostic_restore_authority_is_exact_and_does_not_admit_arbitrary_history() {
    // Arrange / Act
    let admitted =
        authorized_remediation_plan("diagnostic_restore", 4).expect("diagnostic authority");

    // Assert
    assert_eq!(
        admitted,
        (NOISE_DIAGNOSTIC_PLAN_RELATIVE, NOISE_DIAGNOSTIC_PLAN_SHA256)
    );
    assert!(authorized_remediation_plan("diagnostic_restore", 3).is_err());
    assert!(authorized_remediation_plan("historical_restore", 1).is_err());
}

#[test]
fn tcp_payload_recovery_authority_is_current_and_narrow() {
    // Arrange / Act
    let admitted = authorized_remediation_plan("tcp_payload_recovery", 2)
        .expect("current TCP payload recovery authority");

    // Assert
    assert_eq!(
        admitted,
        (
            "docs/parity/work-plans/20260828T185251Z-STR-005/PLAN.md",
            "14bd8aef5d78f38881a3da1a99a6808f7f6e8c93bb1d1a02d7972fcaaeb1d843",
        )
    );
    assert!(authorized_remediation_plan("tcp_payload_recovery", 1).is_err());
}

#[test]
fn tcp_payload_recovery_root_selects_only_the_current_plan() {
    // Arrange / Act
    let contract = restore_invocation_contract(Utf8Path::new(TCP_PAYLOAD_RECOVERY_ROOT), false);

    // Assert
    assert_eq!(contract.0, Utf8Path::new(TCP_PAYLOAD_RECOVERY_ROOT));
    assert_eq!(contract.1, TCP_PAYLOAD_PLAN_RELATIVE);
}

#[test]
fn tcp_payload_recovery_action_matches_only_the_recovery_root() {
    // Arrange / Act / Assert
    assert!(authorization_action_allowed(
        false,
        TCP_PAYLOAD_RECOVERY_ROOT,
        "tcp_payload_recovery"
    ));
    assert!(!authorization_action_allowed(
        false,
        EFFECT_ROOT,
        "tcp_payload_recovery"
    ));
}

#[test]
fn tcp_payload_diagnostic_restore_is_current_and_narrow() {
    // Arrange / Act
    let admitted = authorized_remediation_plan("tcp_payload_diagnostic_restore", 6)
        .expect("current diagnostic restore authority");
    let contract =
        restore_invocation_contract(Utf8Path::new(TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT), false);

    // Assert
    assert_eq!(
        admitted,
        (TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256)
    );
    assert_eq!(
        contract.0,
        Utf8Path::new(TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT)
    );
    assert_eq!(contract.1, TCP_PAYLOAD_PLAN_RELATIVE);
    assert!(authorization_action_allowed(
        false,
        TCP_PAYLOAD_DIAGNOSTIC_RESTORE_ROOT,
        "tcp_payload_diagnostic_restore"
    ));
    assert!(authorized_remediation_plan("tcp_payload_diagnostic_restore", 5).is_err());
}
