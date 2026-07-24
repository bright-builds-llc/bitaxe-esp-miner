use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

use thiserror::Error;

const DETECTOR_PROGRAM: &str = "just";
const DETECTOR_ARGUMENT: &str = "detect-ultra205";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase36HardwareGateStatus {
    DetectorAdmittedCredentialValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Phase36HardwareGateError {
    #[error("phase36_broker_wrong_board")]
    WrongBoard,
    #[error("phase36_broker_detector_failed")]
    DetectorFailed,
    #[error("phase36_broker_wifi_credentials_invalid")]
    WifiCredentialsInvalid,
}

trait Phase36HardwareBoundary {
    fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError>;
    fn validate_wifi_credentials(
        &mut self,
        wifi_credentials: &Path,
    ) -> Result<(), Phase36HardwareGateError>;
}

struct ProcessHardwareBoundary;

impl Phase36HardwareBoundary for ProcessHardwareBoundary {
    fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError> {
        let output = Command::new(DETECTOR_PROGRAM)
            .arg(DETECTOR_ARGUMENT)
            .stdin(Stdio::null())
            .output()
            .map_err(|_| Phase36HardwareGateError::DetectorFailed)?;
        if !output.status.success() {
            return Err(Phase36HardwareGateError::DetectorFailed);
        }
        Ok(())
    }

    fn validate_wifi_credentials(
        &mut self,
        wifi_credentials: &Path,
    ) -> Result<(), Phase36HardwareGateError> {
        let metadata = fs::symlink_metadata(wifi_credentials)
            .map_err(|_| Phase36HardwareGateError::WifiCredentialsInvalid)?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.permissions().mode() & 0o777 != 0o600
        {
            return Err(Phase36HardwareGateError::WifiCredentialsInvalid);
        }
        Ok(())
    }
}

pub fn run_phase36_hardware_pre_capture_gate(
    board: u16,
    wifi_credentials: &Path,
) -> Result<Phase36HardwareGateStatus, Phase36HardwareGateError> {
    let mut boundary = ProcessHardwareBoundary;
    run_phase36_hardware_pre_capture_gate_with(&mut boundary, board, wifi_credentials)
}

fn run_phase36_hardware_pre_capture_gate_with(
    boundary: &mut impl Phase36HardwareBoundary,
    board: u16,
    wifi_credentials: &Path,
) -> Result<Phase36HardwareGateStatus, Phase36HardwareGateError> {
    if board != 205 {
        return Err(Phase36HardwareGateError::WrongBoard);
    }

    boundary.run_detector()?;
    boundary.validate_wifi_credentials(wifi_credentials)?;

    Ok(Phase36HardwareGateStatus::DetectorAdmittedCredentialValidated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeHardwareBoundary {
        detector_calls: usize,
        credential_calls: usize,
        detector_fails: bool,
        credential_fails: bool,
    }

    impl Phase36HardwareBoundary for FakeHardwareBoundary {
        fn run_detector(&mut self) -> Result<(), Phase36HardwareGateError> {
            self.detector_calls += 1;
            if self.detector_fails {
                return Err(Phase36HardwareGateError::DetectorFailed);
            }
            Ok(())
        }

        fn validate_wifi_credentials(
            &mut self,
            _wifi_credentials: &Path,
        ) -> Result<(), Phase36HardwareGateError> {
            self.credential_calls += 1;
            if self.credential_fails {
                return Err(Phase36HardwareGateError::WifiCredentialsInvalid);
            }
            Ok(())
        }
    }

    #[test]
    fn detector_failure_stops_before_credential_access() {
        // Arrange
        let mut boundary = FakeHardwareBoundary {
            detector_fails: true,
            ..FakeHardwareBoundary::default()
        };

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 205, Path::new("opaque"));

        // Assert
        assert_eq!(result, Err(Phase36HardwareGateError::DetectorFailed));
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(boundary.credential_calls, 0);
    }

    #[test]
    fn detector_success_precedes_single_credential_validation() {
        // Arrange
        let mut boundary = FakeHardwareBoundary::default();

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 205, Path::new("opaque"));

        // Assert
        assert_eq!(
            result,
            Ok(Phase36HardwareGateStatus::DetectorAdmittedCredentialValidated)
        );
        assert_eq!(boundary.detector_calls, 1);
        assert_eq!(boundary.credential_calls, 1);
    }

    #[test]
    fn wrong_board_stops_before_detector_or_credential_access() {
        // Arrange
        let mut boundary = FakeHardwareBoundary::default();

        // Act
        let result =
            run_phase36_hardware_pre_capture_gate_with(&mut boundary, 601, Path::new("opaque"));

        // Assert
        assert_eq!(result, Err(Phase36HardwareGateError::WrongBoard));
        assert_eq!(boundary.detector_calls, 0);
        assert_eq!(boundary.credential_calls, 0);
    }
}
