//! Shared safety evidence contracts.
//!
//! Phase 6 boundary notes:
//! - D-17: unavailable or unverified safety state remains explicit.
//! - D-18: ASIC and mining gates consume evidence without treating implementation as proof.
//! - D-20: evidence labels align with parity checklist semantics.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SafetyCriticalEvidence {
    Missing,
    ImplementedNotVerified { source: &'static str },
    HardwareSmoke { evidence_id: &'static str },
    HardwareRegression { evidence_id: &'static str },
}

impl SafetyCriticalEvidence {
    #[must_use]
    pub const fn implemented_not_verified(source: &'static str) -> Self {
        Self::ImplementedNotVerified { source }
    }

    #[must_use]
    pub const fn hardware_smoke(evidence_id: &'static str) -> Self {
        Self::HardwareSmoke { evidence_id }
    }

    #[must_use]
    pub const fn hardware_regression(evidence_id: &'static str) -> Self {
        Self::HardwareRegression { evidence_id }
    }

    #[must_use]
    pub const fn is_hardware_verified(self) -> bool {
        matches!(
            self,
            Self::HardwareSmoke { .. } | Self::HardwareRegression { .. }
        )
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Missing => "pending",
            Self::ImplementedNotVerified { .. } => "unit",
            Self::HardwareSmoke { .. } => "hardware-smoke",
            Self::HardwareRegression { .. } => "hardware-regression",
        }
    }
}

#[cfg(test)]
pub(crate) const MODULE_NAME: &str = "evidence";

#[cfg(test)]
mod tests {
    use super::SafetyCriticalEvidence;

    #[test]
    fn implemented_not_verified_is_not_valid_hardware_verification_evidence() {
        // Arrange
        let evidence = SafetyCriticalEvidence::implemented_not_verified("unit");

        // Act
        let hardware_verified = evidence.is_hardware_verified();
        let label = evidence.label();

        // Assert
        assert!(!hardware_verified);
        assert_eq!(label, "unit");
    }

    #[test]
    fn hardware_smoke_is_valid_hardware_verification_evidence() {
        // Arrange
        let evidence =
            SafetyCriticalEvidence::hardware_smoke("phase-06-ultra-205-safety-hardware-smoke");

        // Act
        let hardware_verified = evidence.is_hardware_verified();
        let label = evidence.label();

        // Assert
        assert!(hardware_verified);
        assert_eq!(label, "hardware-smoke");
    }
}
