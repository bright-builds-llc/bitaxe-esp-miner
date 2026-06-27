//! ASIC dispatch boundary for the V1 Ultra 205 BM1366 path.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/asic.c`
//! - `reference/esp-miner/main/device_config.h`
//! - parity checklist rows `ASIC-003`, `ASIC-004`, and `ASIC-008`

use bitaxe_config::catalog::{BoardCatalogEntry, VerificationScope};

const ACTIVE_BOARD_VERSION: &str = "205";
const ACTIVE_FAMILY: &str = "Ultra";
const ACTIVE_ASIC_MODEL: &str = "BM1366";
const ACTIVE_ASIC_COUNT: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsicDispatch {
    ActiveBm1366,
    Deferred(DeferredAsic),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredAsic {
    model: DeferredAsicModel,
    reason: DeferredAsicReason,
    scope: VerificationScope,
}

impl DeferredAsic {
    #[must_use]
    pub const fn model(self) -> DeferredAsicModel {
        self.model
    }

    #[must_use]
    pub const fn reason(self) -> DeferredAsicReason {
        self.reason
    }

    #[must_use]
    pub const fn scope(self) -> VerificationScope {
        self.scope
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredAsicModel {
    Bm1370,
    Bm1368,
    Bm1397,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredAsicReason {
    NotV1ActiveTarget,
    MissingHardwareEvidence,
}

#[must_use]
pub fn dispatch_catalog_entry(entry: BoardCatalogEntry) -> AsicDispatch {
    dispatch_from_parts(
        entry.board_version(),
        entry.family(),
        entry.asic().model(),
        entry.asic_count(),
        entry.verification_scope(),
    )
}

#[must_use]
pub fn dispatch_from_parts(
    board_version: &str,
    family: &str,
    asic_model: &str,
    asic_count: u8,
    scope: VerificationScope,
) -> AsicDispatch {
    if is_active_ultra_205_bm1366(board_version, family, asic_model, asic_count, scope) {
        return AsicDispatch::ActiveBm1366;
    }

    AsicDispatch::Deferred(DeferredAsic {
        model: DeferredAsicModel::from_model(asic_model),
        reason: deferred_reason(scope),
        scope,
    })
}

fn is_active_ultra_205_bm1366(
    board_version: &str,
    family: &str,
    asic_model: &str,
    asic_count: u8,
    scope: VerificationScope,
) -> bool {
    board_version == ACTIVE_BOARD_VERSION
        && family == ACTIVE_FAMILY
        && asic_model == ACTIVE_ASIC_MODEL
        && asic_count == ACTIVE_ASIC_COUNT
        && scope == VerificationScope::ActiveUltra205
}

impl DeferredAsicModel {
    #[must_use]
    pub const fn from_model(model: &str) -> Self {
        match model.as_bytes() {
            b"BM1370" => Self::Bm1370,
            b"BM1368" => Self::Bm1368,
            b"BM1397" => Self::Bm1397,
            _ => Self::Other,
        }
    }
}

const fn deferred_reason(scope: VerificationScope) -> DeferredAsicReason {
    match scope {
        VerificationScope::ActiveUltra205 => DeferredAsicReason::NotV1ActiveTarget,
        VerificationScope::NotHardwareVerified => DeferredAsicReason::MissingHardwareEvidence,
    }
}
