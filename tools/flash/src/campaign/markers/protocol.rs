use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProtocolGateMarker {
    Ready,
    TransactionUnavailable,
    PartitionOwnerUnavailable,
    NamespaceUnavailable,
    PrimarySelectorInvalid,
    PrimarySelectorUnsupported,
    FallbackSelectorInvalid,
    FallbackSelectorUnsupported,
}

impl ProtocolGateMarker {
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
