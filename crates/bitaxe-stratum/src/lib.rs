pub mod error;
pub mod jsonrpc;
pub mod v1;

pub use error::StratumV1Error;

/// Phase 4 Stratum status contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StratumRuntimeStatus {
    /// Stratum v1 protocol parsing and serialization are active in the pure core.
    ActiveV1Core,
}

#[cfg(test)]
mod tests {
    use super::StratumRuntimeStatus;
    use crate::error::StratumV1Error;
    use crate::jsonrpc::StratumRequestId;

    #[test]
    fn stratum_v1_contract_runtime_status_is_active() {
        // Arrange
        let status = StratumRuntimeStatus::ActiveV1Core;

        // Act
        let observed = status;

        // Assert
        assert_eq!(observed, StratumRuntimeStatus::ActiveV1Core);
    }

    #[test]
    fn stratum_v1_contract_unknown_method_error_names_stratum_v1() {
        // Arrange
        let error = StratumV1Error::UnknownMethod {
            method: "mining.unknown".to_owned(),
        };

        // Act
        let rendered = error.to_string();

        // Assert
        assert!(rendered.contains("unknown Stratum v1 method"));
    }

    #[test]
    fn stratum_v1_contract_request_id_exposes_raw_value() {
        // Arrange
        let request_id = StratumRequestId::new(7);

        // Act
        let raw = request_id.raw();

        // Assert
        assert_eq!(raw, 7);
    }
}
