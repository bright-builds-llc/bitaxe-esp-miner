#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProtocolSelectorObservation {
    Missing,
    V1,
    V2,
    Unsupported,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfiguredStratumProtocol {
    V1,
    V2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConfiguredProtocolPlan {
    pub(crate) primary: ConfiguredStratumProtocol,
    pub(crate) fallback: ConfiguredStratumProtocol,
    pub(crate) prefer_fallback: bool,
}

impl ConfiguredProtocolPlan {
    pub(crate) const fn from_selectors(
        primary: ProtocolSelectorObservation,
        fallback: ProtocolSelectorObservation,
        prefer_fallback: bool,
    ) -> Result<Self, ProductionProtocolGateDecision> {
        let primary = match primary {
            ProtocolSelectorObservation::Missing | ProtocolSelectorObservation::V1 => {
                ConfiguredStratumProtocol::V1
            }
            ProtocolSelectorObservation::V2 => ConfiguredStratumProtocol::V2,
            ProtocolSelectorObservation::Invalid => {
                return Err(ProductionProtocolGateDecision::PrimarySelectorInvalid)
            }
            ProtocolSelectorObservation::Unsupported => {
                return Err(ProductionProtocolGateDecision::PrimarySelectorUnsupported)
            }
        };
        let fallback = match fallback {
            ProtocolSelectorObservation::Missing | ProtocolSelectorObservation::V1 => {
                ConfiguredStratumProtocol::V1
            }
            ProtocolSelectorObservation::V2 => ConfiguredStratumProtocol::V2,
            ProtocolSelectorObservation::Invalid => {
                return Err(ProductionProtocolGateDecision::FallbackSelectorInvalid)
            }
            ProtocolSelectorObservation::Unsupported => {
                return Err(ProductionProtocolGateDecision::FallbackSelectorUnsupported)
            }
        };
        Ok(Self {
            primary,
            fallback,
            prefer_fallback,
        })
    }

    #[must_use]
    pub(crate) const fn initial(self) -> ConfiguredStratumProtocol {
        if self.prefer_fallback {
            self.fallback
        } else {
            self.primary
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductionProtocolGateDecision {
    Ready,
    TransactionUnavailable,
    PartitionOwnerUnavailable,
    NamespaceUnavailable,
    PrimarySelectorInvalid,
    PrimarySelectorUnsupported,
    FallbackSelectorInvalid,
    FallbackSelectorUnsupported,
}

impl ProductionProtocolGateDecision {
    pub(crate) const fn from_selectors(
        primary: ProtocolSelectorObservation,
        fallback: ProtocolSelectorObservation,
    ) -> Self {
        match primary {
            ProtocolSelectorObservation::Invalid => Self::PrimarySelectorInvalid,
            ProtocolSelectorObservation::V2 | ProtocolSelectorObservation::Unsupported => {
                Self::PrimarySelectorUnsupported
            }
            ProtocolSelectorObservation::Missing | ProtocolSelectorObservation::V1 => {
                match fallback {
                    ProtocolSelectorObservation::Invalid => Self::FallbackSelectorInvalid,
                    ProtocolSelectorObservation::V2 | ProtocolSelectorObservation::Unsupported => {
                        Self::FallbackSelectorUnsupported
                    }
                    ProtocolSelectorObservation::Missing | ProtocolSelectorObservation::V1 => {
                        Self::Ready
                    }
                }
            }
        }
    }

    pub(crate) const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::TransactionUnavailable => "transaction_unavailable",
            Self::PartitionOwnerUnavailable => "partition_owner_unavailable",
            Self::NamespaceUnavailable => "namespace_unavailable",
            Self::PrimarySelectorInvalid => "primary_selector_invalid",
            Self::PrimarySelectorUnsupported => "primary_selector_unsupported",
            Self::FallbackSelectorInvalid => "fallback_selector_invalid",
            Self::FallbackSelectorUnsupported => "fallback_selector_unsupported",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_selectors_use_the_v1_default() {
        // Arrange
        let primary = ProtocolSelectorObservation::Missing;
        let fallback = ProtocolSelectorObservation::Missing;

        // Act
        let decision = ProductionProtocolGateDecision::from_selectors(primary, fallback);

        // Assert
        assert_eq!(decision, ProductionProtocolGateDecision::Ready);
        assert!(decision.is_ready());
    }

    #[test]
    fn explicit_v1_selectors_are_ready() {
        // Arrange
        let primary = ProtocolSelectorObservation::V1;
        let fallback = ProtocolSelectorObservation::V1;

        // Act
        let decision = ProductionProtocolGateDecision::from_selectors(primary, fallback);

        // Assert
        assert_eq!(decision, ProductionProtocolGateDecision::Ready);
    }

    #[test]
    fn primary_failure_precedes_fallback_failure() {
        // Arrange
        let primary = ProtocolSelectorObservation::Invalid;
        let fallback = ProtocolSelectorObservation::Unsupported;

        // Act
        let decision = ProductionProtocolGateDecision::from_selectors(primary, fallback);

        // Assert
        assert_eq!(
            decision,
            ProductionProtocolGateDecision::PrimarySelectorInvalid
        );
    }

    #[test]
    fn every_closed_decision_has_a_value_free_label() {
        // Arrange
        let decisions = [
            ProductionProtocolGateDecision::Ready,
            ProductionProtocolGateDecision::TransactionUnavailable,
            ProductionProtocolGateDecision::PartitionOwnerUnavailable,
            ProductionProtocolGateDecision::NamespaceUnavailable,
            ProductionProtocolGateDecision::PrimarySelectorInvalid,
            ProductionProtocolGateDecision::PrimarySelectorUnsupported,
            ProductionProtocolGateDecision::FallbackSelectorInvalid,
            ProductionProtocolGateDecision::FallbackSelectorUnsupported,
        ];

        // Act / Assert
        for decision in decisions {
            let label = decision.label();
            assert!(!label.contains("SV1"));
            assert!(!label.contains(':'));
            assert!(!label.contains('/'));
        }
    }

    #[test]
    fn explicit_v2_plan_selects_primary_or_fallback_without_opening_secrets() {
        // Arrange
        let primary = ProtocolSelectorObservation::V2;
        let fallback = ProtocolSelectorObservation::V1;

        // Act
        let primary_plan =
            ConfiguredProtocolPlan::from_selectors(primary, fallback, false).expect("primary plan");
        let fallback_plan =
            ConfiguredProtocolPlan::from_selectors(primary, fallback, true).expect("fallback plan");

        // Assert
        assert_eq!(primary_plan.initial(), ConfiguredStratumProtocol::V2);
        assert_eq!(fallback_plan.initial(), ConfiguredStratumProtocol::V1);
    }
}
