use bitaxe_asic::bm1366::{
    adapter_gate::default_fail_closed_status_log, observation::AsicInitStatus,
};

pub const DEFAULT_FAIL_CLOSED_STATUS_LOG: &str = "asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled";

pub fn publish_default_fail_closed_status() {
    debug_assert_eq!(
        default_fail_closed_status_log(),
        DEFAULT_FAIL_CLOSED_STATUS_LOG
    );
    log::info!("{}", default_fail_closed_status_log());
}

pub fn publish_status(status: AsicInitStatus) {
    match status {
        AsicInitStatus::PreflightMissing { reason } => log::info!(
            "asic_status=preflight_missing reason={reason} initialized=false mining=disabled work_submission=disabled"
        ),
        AsicInitStatus::ChipDetectOnly => log::info!(
            "asic_status=chip_detect_only initialized=false mining=disabled work_submission=disabled"
        ),
        AsicInitStatus::ChipDetectedNoMining { chips } => log::info!(
            "asic_status=chip_detected chips={chips} initialized=false mining=disabled work_submission=disabled"
        ),
        AsicInitStatus::InitializedNoMining => log::info!(
            "asic_status=initialized_no_mining initialized=true mining=disabled work_submission=disabled"
        ),
        AsicInitStatus::FailClosed { reason } => log::info!(
            "asic_status=fail_closed reason={reason} initialized=false mining=disabled work_submission=disabled"
        ),
    }
}
