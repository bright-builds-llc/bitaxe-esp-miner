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
    let admitted = authorized_remediation_plan("tcp_payload_recovery", 3)
        .expect("current TCP payload recovery authority");

    // Assert
    assert_eq!(
        admitted,
        (
            "docs/parity/work-plans/20260829T032813Z-STR-005-CONNECTION-IDENTITY/PLAN.md",
            "544f57f8c940bc4e5cfeb69539928e153629b55dc12c5d04e404219ca48a5ba5",
        )
    );
    assert!(authorized_remediation_plan("tcp_payload_recovery", 2).is_err());
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
    let admitted = authorized_remediation_plan("tcp_payload_diagnostic_restore", 9)
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
    assert!(authorized_remediation_plan("tcp_payload_diagnostic_restore", 8).is_err());
}

#[test]
fn tcp_payload_restore_preflight_is_admission_only_and_current() {
    // Arrange / Act
    let admitted = authorized_remediation_plan("tcp_payload_restore_preflight", 9)
        .expect("current TCP payload preflight authority");
    let contract = restore_invocation_contract(Utf8Path::new(TCP_PAYLOAD_PREFLIGHT_ROOT), true);

    // Assert
    assert_eq!(
        admitted,
        (TCP_PAYLOAD_PLAN_RELATIVE, TCP_PAYLOAD_PLAN_SHA256)
    );
    assert_eq!(contract.0, Utf8Path::new(TCP_PAYLOAD_PREFLIGHT_ROOT));
    assert_eq!(contract.1, TCP_PAYLOAD_PLAN_RELATIVE);
    assert!(authorization_action_allowed(
        true,
        TCP_PAYLOAD_PREFLIGHT_ROOT,
        "tcp_payload_restore_preflight"
    ));
    assert!(!authorization_action_allowed(
        false,
        TCP_PAYLOAD_PREFLIGHT_ROOT,
        "tcp_payload_restore_preflight"
    ));
}

#[test]
fn bwg_restoration_root_and_authority_are_closed_to_one_attempt_grammar() {
    // Arrange
    let root = Utf8Path::new("scratch/bwg-worker-restoration/bwg007-attempt-001/recovery");

    // Act
    let contract = restore_invocation_contract(root, false);
    let authority = authorized_remediation_plan("bwg_worker_restoration", 1)
        .expect("BWG restoration authority should be explicit");

    // Assert
    assert_eq!(contract, (root, BWG_RESTORATION_PLAN_RELATIVE));
    assert_eq!(
        authority,
        (BWG_RESTORATION_PLAN_RELATIVE, BWG_RESTORATION_PLAN_SHA256)
    );
    assert!(is_bwg_restoration_root(root));
    assert!(!is_bwg_restoration_root(Utf8Path::new(
        "scratch/bwg-worker-restoration/bwg007-attempt-1/recovery"
    )));
    assert!(authorized_remediation_plan("bwg_worker_restoration", 1_000).is_err());
    assert_eq!(
        sha256_bytes(include_str!(
            "../../../../docs/adr/0019-supervise-bwg-restoration-through-a-protected-browser-campaign.md"
        ).as_bytes()),
        BWG_RESTORATION_PLAN_SHA256
    );
}
