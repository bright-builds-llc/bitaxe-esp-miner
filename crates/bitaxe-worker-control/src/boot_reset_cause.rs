//! Closed reset category the boot review reports for the current boot.

/// Why this boot started, as reported by `worker-boot-review-v1`.
///
/// The labels match the firmware boot-identity reset categories exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootResetCause {
    PowerOn,
    SoftwareCpu,
    Watchdog,
    Panic,
    Brownout,
    /// Any reset reason not admitted above, or one the adapter cannot observe.
    Other,
}

impl BootResetCause {
    /// Stable wire label for `resetCause`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PowerOn => "power_on",
            Self::SoftwareCpu => "software_cpu",
            Self::Watchdog => "watchdog",
            Self::Panic => "panic",
            Self::Brownout => "brownout",
            Self::Other => "other",
        }
    }
}
