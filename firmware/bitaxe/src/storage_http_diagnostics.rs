//! Shared first-error and completed-outcome reporting; never formats callee errors.
#[path = "storage_http_diagnostics/model.rs"]
mod model;
use bitaxe_core::usb_diagnostics::StorageHttpError as Error;
pub(crate) use bitaxe_core::usb_diagnostics::StorageHttpPhase as Phase;
use esp_idf_svc::{io::EspIOError, sys};
static RESULTS: model::StartupResults = model::StartupResults::new();

pub(crate) fn observe<T, E: Into<anyhow::Error>>(
    phase: Phase,
    result: Result<T, E>,
) -> anyhow::Result<T> {
    result.map_err(|error| {
        let error = error.into();
        record(phase, classify(phase, &error));
        error
    })
}
pub(crate) fn record_esp(phase: Phase, code: sys::esp_err_t) {
    record(phase, classify_esp(phase, code));
}
pub(crate) fn record_io(phase: Phase, error: &std::io::Error) {
    record(phase, classify_io(error));
}
pub(crate) fn filesystem_outcome(available: bool) {
    RESULTS.filesystem(available);
    publish_outcome();
}
pub(crate) fn http_outcome(ready: bool) {
    RESULTS.http(ready);
    publish_outcome();
}
pub(crate) fn maybe_failure_marker() -> Option<String> {
    RESULTS.failure_marker()
}
pub(crate) fn maybe_status_marker() -> Option<String> {
    RESULTS.status_marker()
}
fn publish_outcome() {
    if let Some(line) = RESULTS.status_marker() {
        crate::info_retained(&line);
    }
}
fn record(phase: Phase, error: Error) {
    if RESULTS.failure(phase, error) {
        if let Some(line) = RESULTS.failure_marker() {
            crate::info_retained(&line);
        }
    }
}
fn classify(phase: Phase, error: &anyhow::Error) -> Error {
    if let Some(error) = error.downcast_ref::<EspIOError>() {
        return classify_esp(phase, error.0.code());
    }
    if let Some(error) = error.downcast_ref::<sys::EspError>() {
        return classify_esp(phase, error.code());
    }
    if let Some(error) = error.downcast_ref::<std::io::Error>() {
        return classify_io(error);
    }
    Error::Other
}
fn classify_io(error: &std::io::Error) -> Error {
    match error.kind() {
        std::io::ErrorKind::OutOfMemory => Error::NoMemory,
        std::io::ErrorKind::TimedOut => Error::Timeout,
        _ => Error::IoError,
    }
}
fn classify_esp(phase: Phase, code: sys::esp_err_t) -> Error {
    match code {
        sys::ESP_ERR_NO_MEM | sys::ESP_ERR_HTTPD_ALLOC_MEM => Error::NoMemory,
        sys::ESP_ERR_NOT_FOUND => Error::NotFound,
        sys::ESP_ERR_INVALID_STATE => Error::InvalidState,
        sys::ESP_ERR_INVALID_ARG => Error::InvalidArgument,
        sys::ESP_ERR_TIMEOUT => Error::Timeout,
        sys::ESP_ERR_HTTPD_HANDLER_EXISTS => Error::HandlerExists,
        sys::ESP_ERR_HTTPD_HANDLERS_FULL => Error::HandlersFull,
        sys::ESP_ERR_HTTPD_TASK => Error::HttpTask,
        sys::ESP_FAIL if phase == Phase::SpiffsRegister => Error::MountFailed,
        sys::ESP_FAIL if phase == Phase::HttpServer => Error::SocketSetup,
        _ => Error::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn storage_failure_and_http_transport_failure_have_distinct_typed_causes() {
        // Arrange / Act / Assert
        assert_eq!(
            classify_esp(Phase::SpiffsRegister, sys::ESP_FAIL),
            Error::MountFailed
        );
        assert_eq!(
            classify_esp(Phase::HttpServer, sys::ESP_FAIL),
            Error::SocketSetup
        );
        assert_eq!(
            classify_esp(Phase::SpiffsInfo, sys::ESP_ERR_NOT_FOUND),
            Error::NotFound
        );
        assert_eq!(
            classify_esp(Phase::HttpRoutes, sys::ESP_ERR_HTTPD_HANDLERS_FULL),
            Error::HandlersFull
        );
        assert_eq!(
            classify_esp(Phase::HttpRoutes, sys::ESP_ERR_HTTPD_HANDLER_EXISTS),
            Error::HandlerExists
        );
        assert_eq!(
            classify_esp(Phase::HttpServer, sys::ESP_ERR_HTTPD_TASK),
            Error::HttpTask
        );
    }
    #[test]
    fn nested_http_error_preserves_inner_code_without_using_its_display() {
        // Arrange
        let io = EspIOError(
            sys::EspError::from(sys::ESP_ERR_HTTPD_HANDLERS_FULL).expect("nonzero fixture code"),
        );
        let wrapped = anyhow::Error::new(io).context("private-route-context");
        // Act / Assert
        assert_eq!(classify(Phase::HttpRoutes, &wrapped), Error::HandlersFull);
    }
}
