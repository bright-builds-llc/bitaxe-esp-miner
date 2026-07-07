use bitaxe_asic::bm1366::{
    adapter_gate::{
        default_fail_closed_status_log, work_result_diagnostic_dispatched_log,
        work_result_diagnostic_invalid_log, work_result_diagnostic_parsed_log,
        work_result_diagnostic_started_log, work_result_diagnostic_timeout_log,
    },
    observation::AsicInitStatus,
    production::{ProductionAsicBlocker, ProductionAsicStatus},
};

pub const DEFAULT_FAIL_CLOSED_STATUS_LOG: &str = "asic_status=preflight_missing reason=hardware_evidence_ack_missing initialized=false mining=disabled work_submission=disabled";

pub fn publish_default_fail_closed_status() {
    debug_assert_eq!(
        default_fail_closed_status_log(),
        DEFAULT_FAIL_CLOSED_STATUS_LOG
    );
    log::info!("{}", default_fail_closed_status_log());
}

pub fn publish_mining_loop_blocked_status(reason: &'static str) {
    log::info!("mining_loop_status=blocked reason={reason} work_submission=disabled");
}

pub fn publish_production_asic_status(status: ProductionAsicStatus) {
    log::info!("{}", production_asic_status_log_line(status));
}

pub fn publish_production_asic_blocked_status(reason: ProductionAsicBlocker) {
    log::warn!("{}", production_asic_blocked_status_log_line(reason));
}

fn production_asic_status_log_line(status: ProductionAsicStatus) -> String {
    match status {
        ProductionAsicStatus::InitializedForProduction => {
            "asic_production_status=initialized".to_owned()
        }
        ProductionAsicStatus::WorkDispatched => "asic_production_status=work_dispatched".to_owned(),
        ProductionAsicStatus::ResultCorrelated => {
            "asic_production_status=result_correlated".to_owned()
        }
        ProductionAsicStatus::FailClosed { reason } => {
            production_asic_blocked_status_log_line(reason)
        }
    }
}

fn production_asic_blocked_status_log_line(reason: ProductionAsicBlocker) -> String {
    format!(
        "asic_production_status=fail_closed reason={} mining=disabled work_submission=disabled",
        reason.as_str()
    )
}

pub fn publish_work_result_diagnostic_started_status() {
    log::info!(
        "{} work_submission=disabled",
        work_result_diagnostic_started_log()
    );
}

pub fn publish_work_result_dispatched_status(job_id: u8, bytes: usize) {
    log::info!(
        "{} job_id=0x{job_id:02x} bytes={bytes} mining=disabled",
        work_result_diagnostic_dispatched_log()
    );
}

pub fn publish_work_result_parsed_status(job_id: u8) {
    log::info!(
        "{} job_id=0x{job_id:02x} mining=disabled work_submission=disabled",
        work_result_diagnostic_parsed_log()
    );
}

pub fn publish_work_result_timeout_status() {
    log::warn!(
        "{} mining=disabled work_submission=disabled",
        work_result_diagnostic_timeout_log()
    );
}

pub fn publish_work_result_bootstrap_initialized_status() {
    log::info!(
        "bm1366_diagnostic_result=bounded_no_result bootstrap=initialized_no_mining mining=disabled work_submission=disabled"
    );
}

pub fn publish_work_result_invalid_status(error: impl std::fmt::Display) {
    log::warn!(
        "{} mining=disabled work_submission=disabled error={error}",
        work_result_diagnostic_invalid_log()
    );
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
