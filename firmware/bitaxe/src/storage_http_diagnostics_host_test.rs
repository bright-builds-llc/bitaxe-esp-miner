//! Production diagnostic classification with platform error types mocked, and no hardware access.
extern crate self as esp_idf_svc;
#[path = "storage_http_diagnostics.rs"]
mod storage_http_diagnostics;

pub mod sys {
    #[allow(non_camel_case_types)]
    pub type esp_err_t = i32;
    pub const ESP_FAIL: i32 = -1;
    pub const ESP_ERR_NO_MEM: i32 = 0x101;
    pub const ESP_ERR_INVALID_ARG: i32 = 0x102;
    pub const ESP_ERR_INVALID_STATE: i32 = 0x103;
    pub const ESP_ERR_NOT_FOUND: i32 = 0x105;
    pub const ESP_ERR_TIMEOUT: i32 = 0x107;
    pub const ESP_ERR_HTTPD_ALLOC_MEM: i32 = 0xb007;
    pub const ESP_ERR_HTTPD_TASK: i32 = 0xb008;
    pub const ESP_ERR_HTTPD_HANDLERS_FULL: i32 = 0xb001;
    pub const ESP_ERR_HTTPD_HANDLER_EXISTS: i32 = 0xb002;
    #[derive(Debug, Copy, Clone)]
    pub struct EspError(i32);
    impl EspError {
        pub fn from(code: i32) -> Option<Self> {
            (code != 0).then_some(Self(code))
        }
        pub fn code(&self) -> i32 {
            self.0
        }
    }
    impl std::fmt::Display for EspError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("callee error text must never be formatted")
        }
    }
    impl std::error::Error for EspError {}
}
pub mod io {
    #[derive(Debug)]
    pub struct EspIOError(pub crate::sys::EspError);
    impl std::fmt::Display for EspIOError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("wrapped callee error text must never be formatted")
        }
    }
    impl std::error::Error for EspIOError {}
}
fn info_retained(line: &str) {
    assert!(bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(line));
    assert!(!line.contains("private"));
}

#[test]
fn wrapped_http_error_is_retained_without_formatting_and_outcomes_are_separate() {
    // Arrange
    let error = io::EspIOError(
        sys::EspError::from(sys::ESP_ERR_HTTPD_ALLOC_MEM).expect("nonzero fixture code"),
    );
    // Act
    let result: anyhow::Result<()> =
        storage_http_diagnostics::observe(storage_http_diagnostics::Phase::HttpServer, Err(error));
    storage_http_diagnostics::record_esp(
        storage_http_diagnostics::Phase::HttpRoutes,
        sys::ESP_ERR_HTTPD_HANDLERS_FULL,
    );
    storage_http_diagnostics::record_io(
        storage_http_diagnostics::Phase::HttpTelemetryWorker,
        &std::io::Error::new(std::io::ErrorKind::PermissionDenied, "private-path"),
    );
    storage_http_diagnostics::filesystem_outcome(true);
    assert_eq!(storage_http_diagnostics::maybe_status_marker(), None);
    storage_http_diagnostics::http_outcome(false);
    // Assert
    assert!(result.is_err());
    assert_eq!(
        storage_http_diagnostics::maybe_failure_marker().as_deref(),
        Some("storage_http_failure schema=v1 phase=http_server error=no_memory redacted=true")
    );
    assert_eq!(
        storage_http_diagnostics::maybe_status_marker().as_deref(),
        Some("storage_http_status schema=v1 spiffs_available=true http_ready=false redacted=true")
    );
}
