//! The firmware crate-root items the production mapping reaches, as the host test fixes them.

/// The host test drives the Ultra 205's production mapping.
pub(crate) mod board_identity {
    pub(crate) const ASIC_FAMILY: bitaxe_asic::asic::AsicFamily =
        bitaxe_asic::asic::AsicFamily::Bm1366;
}

pub(crate) fn semantic_version() -> &'static str {
    "synthetic-test"
}

pub(crate) mod runtime_uptime {
    pub(crate) fn millis() -> u64 {
        1_000
    }
}
