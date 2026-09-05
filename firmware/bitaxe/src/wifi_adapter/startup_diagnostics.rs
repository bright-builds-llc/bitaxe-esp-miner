//! Keeps the first startup failure without retaining arbitrary error or network text.
#[path = "startup_diagnostics/model.rs"]
mod model;
use bitaxe_core::usb_diagnostics::NetworkStartupError;
pub(super) use bitaxe_core::usb_diagnostics::NetworkStartupPhase as Phase;
use esp_idf_svc::sys::{EspError, ESP_ERR_INVALID_STATE, ESP_ERR_NO_MEM, ESP_ERR_TIMEOUT};

static FAILURE: model::FirstFailure = model::FirstFailure::new();

pub(super) fn observe<T, E: Into<anyhow::Error>>(
    phase: Phase,
    result: Result<T, E>,
) -> anyhow::Result<T> {
    result.map_err(|error| {
        let error = error.into();
        let category = classify(&error);
        if FAILURE.record(phase, category) {
            if let Some(line) = FAILURE.marker() {
                crate::info_retained(&line);
            }
        }
        error
    })
}

/// Returns a replay of the first failure; never traverses error chains or network state.
pub(super) fn maybe_marker() -> Option<String> {
    FAILURE.marker()
}

fn classify(error: &anyhow::Error) -> NetworkStartupError {
    if let Some(error) = error.downcast_ref::<EspError>() {
        return match error.code() {
            ESP_ERR_NO_MEM => NetworkStartupError::NoMemory,
            ESP_ERR_INVALID_STATE => NetworkStartupError::InvalidState,
            ESP_ERR_TIMEOUT => NetworkStartupError::Timeout,
            _ => NetworkStartupError::DriverError,
        };
    }
    if let Some(error) = error.downcast_ref::<std::io::Error>() {
        return match error.kind() {
            std::io::ErrorKind::OutOfMemory => NetworkStartupError::NoMemory,
            std::io::ErrorKind::TimedOut => NetworkStartupError::Timeout,
            _ => NetworkStartupError::IoError,
        };
    }
    NetworkStartupError::OwnerError
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn io_classification_does_not_use_private_error_text() {
        // Arrange
        let error = anyhow::Error::from(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "private-network-fixture",
        ));
        // Act / Assert
        assert_eq!(classify(&error), NetworkStartupError::Timeout);
    }
    #[test]
    fn opaque_errors_have_one_closed_category() {
        // Arrange
        let error = anyhow::anyhow!("private-network-fixture");
        // Act / Assert
        assert_eq!(classify(&error), NetworkStartupError::OwnerError);
    }
}
