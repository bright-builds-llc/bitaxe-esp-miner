use crate::PlatformCategory;

/// Returns the closed host-platform category used by session admission.
#[must_use]
pub const fn current_platform() -> PlatformCategory {
    if cfg!(target_os = "macos") {
        PlatformCategory::Macos
    } else if cfg!(target_os = "linux") {
        PlatformCategory::Linux
    } else if cfg!(target_os = "windows") {
        PlatformCategory::Windows
    } else {
        PlatformCategory::Other
    }
}
