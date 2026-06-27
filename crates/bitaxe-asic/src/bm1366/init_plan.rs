//! Pure BM1366 staged initialization planning.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/device_config.h`
//! - `reference/esp-miner/main/power/asic_init.c`
//! - `reference/esp-miner/main/power/asic_reset.c`
//! - `reference/esp-miner/components/asic/asic_common.c:count_asic_chips`
//! - parity checklist rows `ASIC-005`, `ASIC-006`, and `ASIC-008`

use bitaxe_config::{
    ultra_205_catalog_entry, ultra_205_defaults, BoardCatalogEntry, Ultra205Defaults,
    VerificationScope,
};

use super::{
    command::{
        Bm1366AdapterAction, Bm1366Command, NonceSpacePlan, VersionMask, DEFAULT_BAUD, MAX_BAUD,
    },
    observation::AsicInitStatus,
};

const BOARD_PREFLIGHT_EVIDENCE_MISSING: &str = "board_preflight_evidence_missing";
const CONFIG_PREFLIGHT_EVIDENCE_MISSING: &str = "config_preflight_evidence_missing";
const POWER_THERMAL_EVIDENCE_MISSING: &str = "power_thermal_evidence_missing";
const SAFETY_PREFLIGHT_EVIDENCE_MISSING: &str = "safety_preflight_evidence_missing";
const INIT_COMMAND_ENCODING_FAILED: &str = "init_command_encoding_failed";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366InitPlan;

impl Bm1366InitPlan {
    pub fn chip_detect_only(preflight: Bm1366Preflight) -> Bm1366InitDecision {
        if let Err(reason) = preflight.validate_board_and_config() {
            return Bm1366InitDecision::preflight_missing(reason, FailClosedAction::HoldResetLow);
        }

        let mut actions = vec![
            Bm1366AdapterAction::PublishStatus(AsicInitStatus::ChipDetectOnly),
            Bm1366AdapterAction::reset_pulse(),
            Bm1366AdapterAction::UseDefaultBaud { baud: DEFAULT_BAUD },
        ];

        let read_chip_id_actions = Bm1366Command::ReadChipId.adapter_actions();
        let Ok(read_chip_id_actions) = read_chip_id_actions else {
            return Bm1366InitDecision::fail_closed(
                INIT_COMMAND_ENCODING_FAILED,
                FailClosedAction::HoldResetLow,
            );
        };

        actions.extend(read_chip_id_actions);
        actions.push(Bm1366AdapterAction::read_chip_id_response(
            preflight.expected_chips(),
        ));

        Bm1366InitDecision {
            stages: vec![
                Bm1366InitStage::Preflight,
                Bm1366InitStage::Reset,
                Bm1366InitStage::UartDefaultBaud,
                Bm1366InitStage::ChipDetect,
            ],
            actions,
            status: AsicInitStatus::ChipDetectOnly,
            maybe_fail_closed_action: None,
        }
    }

    pub fn full_init(preflight: Bm1366Preflight) -> Bm1366InitDecision {
        if let Err(reason) = preflight.validate_board_and_config() {
            return Bm1366InitDecision::preflight_missing(reason, FailClosedAction::HoldResetLow);
        }

        if !preflight.has_power_and_thermal_evidence() {
            return Bm1366InitDecision::preflight_missing(
                POWER_THERMAL_EVIDENCE_MISSING,
                FailClosedAction::HoldResetLow,
            );
        }

        if !preflight.has_safety_evidence() {
            return Bm1366InitDecision::preflight_missing(
                SAFETY_PREFLIGHT_EVIDENCE_MISSING,
                FailClosedAction::HoldResetLow,
            );
        }

        let mut decision = Self::chip_detect_only(preflight);
        if decision.maybe_fail_closed_action.is_some() {
            return decision;
        }

        let register_actions = [
            Bm1366Command::SetVersionMask(VersionMask::new(0x1fffe000)),
            Bm1366Command::SetNonceSpace(NonceSpacePlan {
                hash_counting_number: 0,
            }),
        ];

        for command in register_actions {
            let encoded = command.adapter_actions();
            let Ok(encoded) = encoded else {
                return Bm1366InitDecision::fail_closed(
                    INIT_COMMAND_ENCODING_FAILED,
                    FailClosedAction::HoldResetLow,
                );
            };

            decision.actions.extend(encoded);
        }

        decision.stages.extend([
            Bm1366InitStage::RegisterInit,
            Bm1366InitStage::FrequencyNonceSetup,
            Bm1366InitStage::MaxBaud,
            Bm1366InitStage::InitializedNoMining,
        ]);
        decision
            .actions
            .push(Bm1366AdapterAction::UseMaxBaud { baud: MAX_BAUD });
        decision.actions.push(Bm1366AdapterAction::ClearRx);
        decision.actions.push(Bm1366AdapterAction::PublishStatus(
            AsicInitStatus::InitializedNoMining,
        ));
        decision.status = AsicInitStatus::InitializedNoMining;
        decision
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366InitStage {
    Preflight,
    Reset,
    UartDefaultBaud,
    ChipDetect,
    RegisterInit,
    FrequencyNonceSetup,
    MaxBaud,
    InitializedNoMining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bm1366Preflight {
    maybe_board: Option<BoardPreflightEvidence>,
    maybe_config: Option<ConfigPreflightEvidence>,
    maybe_power: Option<PowerPreflightEvidence>,
    maybe_thermal: Option<ThermalPreflightEvidence>,
    maybe_safety: Option<SafetyPreflightEvidence>,
}

impl Bm1366Preflight {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_board: None,
            maybe_config: None,
            maybe_power: None,
            maybe_thermal: None,
            maybe_safety: None,
        }
    }

    #[must_use]
    pub const fn chip_detect(
        board: BoardPreflightEvidence,
        config: ConfigPreflightEvidence,
    ) -> Self {
        Self::new().with_board(board).with_config(config)
    }

    #[must_use]
    pub const fn with_board(mut self, board: BoardPreflightEvidence) -> Self {
        self.maybe_board = Some(board);
        self
    }

    #[must_use]
    pub const fn with_config(mut self, config: ConfigPreflightEvidence) -> Self {
        self.maybe_config = Some(config);
        self
    }

    #[must_use]
    pub const fn with_power(mut self, power: PowerPreflightEvidence) -> Self {
        self.maybe_power = Some(power);
        self
    }

    #[must_use]
    pub const fn with_thermal(mut self, thermal: ThermalPreflightEvidence) -> Self {
        self.maybe_thermal = Some(thermal);
        self
    }

    #[must_use]
    pub const fn with_safety(mut self, safety: SafetyPreflightEvidence) -> Self {
        self.maybe_safety = Some(safety);
        self
    }

    fn validate_board_and_config(self) -> Result<(), &'static str> {
        let Some(board) = self.maybe_board else {
            return Err(BOARD_PREFLIGHT_EVIDENCE_MISSING);
        };

        if !board.matches_active_ultra_205() {
            return Err(BOARD_PREFLIGHT_EVIDENCE_MISSING);
        }

        let Some(config) = self.maybe_config else {
            return Err(CONFIG_PREFLIGHT_EVIDENCE_MISSING);
        };

        if !config.matches_ultra_205_defaults() {
            return Err(CONFIG_PREFLIGHT_EVIDENCE_MISSING);
        }

        Ok(())
    }

    const fn has_power_and_thermal_evidence(self) -> bool {
        self.maybe_power.is_some() && self.maybe_thermal.is_some()
    }

    const fn has_safety_evidence(self) -> bool {
        self.maybe_safety.is_some()
    }

    fn expected_chips(self) -> u8 {
        self.maybe_board
            .map_or(ultra_205_catalog_entry().asic_count(), |board| {
                board.entry.asic_count()
            })
    }
}

impl Default for Bm1366Preflight {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardPreflightEvidence {
    entry: BoardCatalogEntry,
}

impl BoardPreflightEvidence {
    #[must_use]
    pub const fn active_ultra_205() -> Self {
        Self {
            entry: ultra_205_catalog_entry(),
        }
    }

    #[must_use]
    pub const fn from_catalog_entry(entry: BoardCatalogEntry) -> Self {
        Self { entry }
    }

    fn matches_active_ultra_205(self) -> bool {
        let asic = self.entry.asic();
        let capabilities = self.entry.capabilities();

        self.entry.board_version() == "205"
            && self.entry.family() == "Ultra"
            && asic.model() == "BM1366"
            && self.entry.asic_count() == 1
            && self.entry.verification_scope() == VerificationScope::ActiveUltra205
            && capabilities.ds4432u()
            && capabilities.ina260()
            && capabilities.asic_enable()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigPreflightEvidence {
    defaults: Ultra205Defaults,
}

impl ConfigPreflightEvidence {
    #[must_use]
    pub const fn ultra_205_defaults() -> Self {
        Self {
            defaults: ultra_205_defaults(),
        }
    }

    #[must_use]
    pub const fn from_defaults(defaults: Ultra205Defaults) -> Self {
        Self { defaults }
    }

    fn matches_ultra_205_defaults(self) -> bool {
        self.defaults.board_version() == "205"
            && self.defaults.device_model() == "ultra"
            && self.defaults.asic_model() == "BM1366"
            && self.defaults.asic_frequency_mhz() == 485
            && self.defaults.asic_voltage_mv() == 1200
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerPreflightEvidence;

impl PowerPreflightEvidence {
    #[must_use]
    pub const fn present() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermalPreflightEvidence;

impl ThermalPreflightEvidence {
    #[must_use]
    pub const fn present() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyPreflightEvidence;

impl SafetyPreflightEvidence {
    #[must_use]
    pub const fn present() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailClosedAction {
    HoldResetLow,
    DisableAsicEnable,
    NoHardwareEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1366InitDecision {
    stages: Vec<Bm1366InitStage>,
    actions: Vec<Bm1366AdapterAction>,
    status: AsicInitStatus,
    maybe_fail_closed_action: Option<FailClosedAction>,
}

impl Bm1366InitDecision {
    #[must_use]
    pub fn stages(&self) -> &[Bm1366InitStage] {
        &self.stages
    }

    #[must_use]
    pub fn actions(&self) -> &[Bm1366AdapterAction] {
        &self.actions
    }

    #[must_use]
    pub const fn status(&self) -> AsicInitStatus {
        self.status
    }

    #[must_use]
    pub const fn fail_closed_action(&self) -> Option<FailClosedAction> {
        self.maybe_fail_closed_action
    }

    fn preflight_missing(reason: &'static str, action: FailClosedAction) -> Self {
        let status = AsicInitStatus::PreflightMissing { reason };
        Self {
            stages: vec![Bm1366InitStage::Preflight],
            actions: vec![
                Bm1366AdapterAction::HoldResetLow,
                Bm1366AdapterAction::PublishStatus(status),
            ],
            status,
            maybe_fail_closed_action: Some(action),
        }
    }

    fn fail_closed(reason: &'static str, action: FailClosedAction) -> Self {
        let status = AsicInitStatus::FailClosed { reason };
        Self {
            stages: vec![Bm1366InitStage::Preflight],
            actions: vec![
                Bm1366AdapterAction::HoldResetLow,
                Bm1366AdapterAction::PublishStatus(status),
            ],
            status,
            maybe_fail_closed_action: Some(action),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bm1366::{
        command::{Bm1366AdapterAction, Bm1366Command, DEFAULT_BAUD},
        observation::AsicInitStatus,
    };

    use super::{
        Bm1366InitPlan, Bm1366InitStage, Bm1366Preflight, BoardPreflightEvidence,
        ConfigPreflightEvidence, FailClosedAction, PowerPreflightEvidence, SafetyPreflightEvidence,
        ThermalPreflightEvidence,
    };

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
            decision.fail_closed_action(),
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
            decision.fail_closed_action(),
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
        .with_power(PowerPreflightEvidence::present())
        .with_thermal(ThermalPreflightEvidence::present());

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
            decision.fail_closed_action(),
            Some(FailClosedAction::HoldResetLow)
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
        .with_power(PowerPreflightEvidence::present())
        .with_thermal(ThermalPreflightEvidence::present())
        .with_safety(SafetyPreflightEvidence::present());

        // Act
        let decision = Bm1366InitPlan::full_init(preflight);

        // Assert
        assert_eq!(decision.status(), AsicInitStatus::InitializedNoMining);
        assert!(decision
            .stages()
            .contains(&Bm1366InitStage::InitializedNoMining));
    }
}
