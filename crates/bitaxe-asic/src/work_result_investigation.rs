//! Host-testable work-result investigation mode parsing.
//!
//! Firmware wraps this with compile-time `option_env!("BITAXE_WORK_RESULT_INVESTIGATION")`.

#[must_use]
pub fn investigation_modes_contain(raw: &str, mode: &str) -> bool {
    if raw.is_empty() {
        return false;
    }

    raw.split(',').any(|part| part.trim() == mode)
}

#[cfg(test)]
mod tests {
    use super::investigation_modes_contain;

    #[test]
    fn investigation_modes_contain_single_mode() {
        assert!(investigation_modes_contain(
            "initialized_no_mining_gate",
            "initialized_no_mining_gate"
        ));
        assert!(!investigation_modes_contain(
            "initialized_no_mining_gate",
            "frequency_ramp"
        ));
    }

    #[test]
    fn investigation_modes_contain_comma_separated_combo() {
        let raw = "frequency_ramp,initialized_no_mining_gate";
        assert!(investigation_modes_contain(raw, "frequency_ramp"));
        assert!(investigation_modes_contain(raw, "initialized_no_mining_gate"));
        assert!(!investigation_modes_contain(raw, "skip_boot_diagnostic_work"));
    }

    #[test]
    fn require_uart_proof_mode_is_distinct_from_require_diagnostic_nonce() {
        let raw = "require_uart_proof_for_production,initialized_no_mining_gate";
        assert!(investigation_modes_contain(raw, "require_uart_proof_for_production"));
        assert!(!investigation_modes_contain(raw, "require_diagnostic_nonce"));
    }

    #[test]
    fn h4_orchestration_modes_parse_in_comma_separated_combo() {
        let raw = "continuous_result_task,job_redispatch_pump,frequency_ramp";
        assert!(investigation_modes_contain(raw, "continuous_result_task"));
        assert!(investigation_modes_contain(raw, "job_redispatch_pump"));
        assert!(investigation_modes_contain(raw, "frequency_ramp"));
        assert!(!investigation_modes_contain(raw, "skip_boot_diagnostic_work"));
    }
}
