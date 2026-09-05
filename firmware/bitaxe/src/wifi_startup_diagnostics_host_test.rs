//! Exercises typed classification and first-failure retention without Wi-Fi access.
extern crate self as esp_idf_svc;
#[path = "wifi_adapter/startup_diagnostics.rs"]
mod startup_diagnostics;

pub mod sys {
    pub const ESP_ERR_NO_MEM: i32 = 257;
    pub const ESP_ERR_INVALID_STATE: i32 = 259;
    pub const ESP_ERR_TIMEOUT: i32 = 263;
    #[derive(Debug)]
    pub struct EspError(pub i32);
    impl EspError {
        pub fn code(&self) -> i32 {
            self.0
        }
    }
    impl std::fmt::Display for EspError {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            panic!("network classifier must never format an error")
        }
    }
    impl std::error::Error for EspError {}
}
fn info_retained(line: &str) {
    assert!(bitaxe_core::usb_diagnostics::is_worker_diagnostic_retained_line(line));
}

#[test]
fn typed_driver_failure_is_public_without_formatting_private_error_text() {
    // Arrange
    let result: Result<(), sys::EspError> = Err(sys::EspError(sys::ESP_ERR_NO_MEM));
    // Act
    let failed = startup_diagnostics::observe(startup_diagnostics::Phase::Driver, result);
    // Assert
    assert!(failed.is_err());
    assert_eq!(
        startup_diagnostics::maybe_marker().as_deref(),
        Some("wifi_startup_failure schema=v1 phase=driver error=no_memory redacted=true")
    );
}
