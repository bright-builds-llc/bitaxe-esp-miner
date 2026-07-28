use bitaxe_safety::{
    evidence::SafetyCriticalEvidence, power::PowerEvidenceToken, status::SafetyStatus,
    thermal::ThermalEvidenceToken,
};

use crate::bm1366::{
    command::{Bm1366AdapterAction, Bm1366Command, DEFAULT_BAUD},
    observation::AsicInitStatus,
};

use super::{
    Bm1366InitPlan, Bm1366InitStage, Bm1366Preflight, BoardPreflightEvidence,
    ChipDetectPlanOptions, ConfigPreflightEvidence, FailClosedAction, PowerPreflightEvidence,
    SafetyPreflightEvidence, ThermalPreflightEvidence,
};

#[test]
fn init_plan_upstream_aligned_skips_reset_and_adds_version_mask_prelude() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );

    // Act
    let decision = Bm1366InitPlan::chip_detect_with_options(
        preflight,
        ChipDetectPlanOptions::upstream_aligned_after_safety_bring_up(),
    );

    // Assert
    assert!(!decision
        .actions()
        .contains(&Bm1366AdapterAction::reset_pulse()));
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::WAIT_TX_DONE));
    assert_eq!(
        decision
            .actions()
            .iter()
            .filter(|action| matches!(action, Bm1366AdapterAction::WriteFrame(_)))
            .count(),
        4,
        "expected 3 version-mask frames plus 1 read-chip-id frame"
    );
}

#[test]
fn init_plan_chip_detect_only_emits_reset_default_baud_and_validating_chip_id_read_actions() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );
    let read_chip_id = Bm1366Command::ReadChipId
        .frame_bytes()
        .expect("read chip-id frame should encode");

    // Act
    let decision = Bm1366InitPlan::chip_detect_only(preflight);

    // Assert
    assert_eq!(decision.status(), AsicInitStatus::ChipDetectOnly);
    assert!(decision.stages().contains(&Bm1366InitStage::Reset));
    assert!(decision
        .stages()
        .contains(&Bm1366InitStage::UartDefaultBaud));
    assert!(decision.stages().contains(&Bm1366InitStage::ChipDetect));
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::reset_pulse()));
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::UseDefaultBaud { baud: DEFAULT_BAUD }));
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::WriteFrame(read_chip_id)));
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::read_chip_id_response(1)));
    assert!(!decision.actions().iter().any(|action| matches!(
        action,
        Bm1366AdapterAction::PublishStatus(AsicInitStatus::ChipDetectedNoMining { .. })
    )));
}

#[test]
fn init_plan_missing_board_scope_fails_closed_with_hold_reset_low() {
    // Arrange
    let preflight =
        Bm1366Preflight::new().with_config(ConfigPreflightEvidence::ultra_205_defaults());

    // Act
    let decision = Bm1366InitPlan::chip_detect_only(preflight);

    // Assert
    assert_eq!(
        decision.status(),
        AsicInitStatus::PreflightMissing {
            reason: "board_preflight_evidence_missing"
        }
    );
    assert_eq!(
        decision.maybe_fail_closed_action(),
        Some(FailClosedAction::HoldResetLow)
    );
    assert!(decision
        .actions()
        .contains(&Bm1366AdapterAction::HoldResetLow));
}

#[test]
fn init_plan_full_init_without_power_thermal_or_safety_fails_before_effectful_stages() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );

    // Act
    let decision = Bm1366InitPlan::full_init(preflight);

    // Assert
    assert_eq!(
        decision.status(),
        AsicInitStatus::PreflightMissing {
            reason: "power_thermal_evidence_missing"
        }
    );
    assert_eq!(decision.stages(), &[Bm1366InitStage::Preflight]);
    assert!(!decision.stages().contains(&Bm1366InitStage::RegisterInit));
    assert!(!decision
        .stages()
        .contains(&Bm1366InitStage::FrequencyNonceSetup));
    assert!(!decision.stages().contains(&Bm1366InitStage::MaxBaud));
    assert!(!decision
        .stages()
        .contains(&Bm1366InitStage::InitializedNoMining));
    assert_eq!(
        decision.maybe_fail_closed_action(),
        Some(FailClosedAction::HoldResetLow)
    );
}

#[test]
fn init_plan_missing_safety_evidence_uses_distinct_fail_closed_reason() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    )
    .with_power(power_preflight())
    .with_thermal(thermal_preflight());

    // Act
    let decision = Bm1366InitPlan::full_init(preflight);

    // Assert
    assert_eq!(
        decision.status(),
        AsicInitStatus::PreflightMissing {
            reason: "safety_preflight_evidence_missing"
        }
    );
    assert_eq!(
        decision.maybe_fail_closed_action(),
        Some(FailClosedAction::HoldResetLow)
    );
}

#[test]
fn init_plan_faulted_safety_status_uses_distinct_fail_closed_reason() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    )
    .with_power(power_preflight())
    .with_thermal(thermal_preflight())
    .with_safety(SafetyPreflightEvidence::from_safety_status(
        SafetyCriticalEvidence::implemented_not_verified("unit"),
        SafetyStatus::PowerFault {
            reason: "power_fault",
        },
    ));

    // Act
    let decision = Bm1366InitPlan::full_init(preflight);

    // Assert
    assert_eq!(
        decision.status(),
        AsicInitStatus::PreflightMissing {
            reason: "safety_preflight_evidence_missing"
        }
    );
}

#[test]
fn init_plan_chip_detect_only_does_not_emit_job_frames() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    );

    // Act
    let decision = Bm1366InitPlan::chip_detect_only(preflight);

    // Assert
    assert!(decision.actions().iter().all(|action| {
        let Bm1366AdapterAction::WriteFrame(frame) = action else {
            return true;
        };

        frame.as_ref().len() < 20
    }));
}

#[test]
fn init_plan_full_init_with_all_preflight_evidence_reaches_initialized_no_mining() {
    // Arrange
    let preflight = Bm1366Preflight::chip_detect(
        BoardPreflightEvidence::active_ultra_205(),
        ConfigPreflightEvidence::ultra_205_defaults(),
    )
    .with_power(power_preflight())
    .with_thermal(thermal_preflight())
    .with_safety(safety_preflight());

    // Act
    let decision = Bm1366InitPlan::full_init(preflight);

    // Assert
    assert_eq!(decision.status(), AsicInitStatus::InitializedNoMining);
    assert!(decision
        .stages()
        .contains(&Bm1366InitStage::InitializedNoMining));
}

fn power_preflight() -> PowerPreflightEvidence {
    PowerPreflightEvidence::from_power_token(PowerEvidenceToken {
        bus_voltage_volts: 5.0,
        current_amps: 2.5,
        power_watts: 12.5,
    })
}

fn thermal_preflight() -> ThermalPreflightEvidence {
    ThermalPreflightEvidence::from_thermal_token(ThermalEvidenceToken {
        chip_temp_celsius: 55.0,
        evidence: SafetyCriticalEvidence::implemented_not_verified("unit"),
    })
}

fn safety_preflight() -> SafetyPreflightEvidence {
    SafetyPreflightEvidence::from_safety_status(
        SafetyCriticalEvidence::implemented_not_verified("unit"),
        SafetyStatus::Normal,
    )
}
