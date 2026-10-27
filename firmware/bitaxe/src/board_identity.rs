//! The built board compared with the `boardversion` the board stores (ADR-0041).
//!
//! Read once after the settings snapshot loads. The comparison is reported over USB and the
//! API; only a stored value naming another known board withholds hardware control.
use bitaxe_api::acceptance_budget::SoakPrerequisite;
use bitaxe_api::{AttestedBoard, MiningSafetyLimits, PlatformAsic, PlatformBoard};
use bitaxe_asic::asic::AsicFamily;
use bitaxe_config::{BoardCatalogEntry, StoredValueKind};
use bitaxe_core::board_identity::{BoardIdentity, KnownBoard, StoredBoard};
use bitaxe_core::{AsicTarget, BoardTarget};
use std::sync::OnceLock;

/// Everything that names the built board, selected together so the facts cannot drift apart.
/// Package admission requires `MARKER` in the application (`tools/flash/src/board_target.rs`).
#[cfg(not(feature = "board-gamma-601"))]
mod built {
    use super::*;
    pub(super) const BOARD: KnownBoard = KnownBoard::Ultra205;
    pub(super) const MARKER: &str = "firmware_board=205";
    pub(super) const TARGET: BoardTarget = BoardTarget::Ultra205;
    pub(super) const ASIC: AsicTarget = AsicTarget::Bm1366;
    pub(super) const ATTESTED: AttestedBoard = AttestedBoard::Ultra205;
    pub(super) const PLATFORM_BOARD: PlatformBoard = PlatformBoard::Ultra205;
    pub(super) const PLATFORM_ASIC: PlatformAsic = PlatformAsic::Bm1366;
    pub(super) const FAMILY: AsicFamily = AsicFamily::Bm1366;
    pub(super) const DISPATCH_ADMITTED: bool = true;
    pub(super) const LIVE_HARDWARE_CHANGES: bool = true;
    pub(super) const WORKER_MODEL: &str = "bitaxe-ultra";
    pub(super) const WORKER_CAPABILITY: &str = include_str!("../bwg/ultra205-capability.json");
    pub(super) const SOAK_PREREQUISITE: SoakPrerequisite =
        SoakPrerequisite::OriginalCampaignComplete;
    /// The kind SELF-001's evidence and admission tuple carry (`ultra205_full_v1`).
    pub(super) const SELF_TEST_KIND: &str = "ultra205_full_v1";
    /// The DS4432U reports no regulator temperature.
    pub(super) const MAYBE_SELF_TEST_VR_MAX_C: Option<f32> = None;
    pub(super) const fn catalog() -> BoardCatalogEntry {
        bitaxe_config::ultra_205_catalog_entry()
    }
}

#[cfg(feature = "board-gamma-601")]
mod built {
    use super::*;
    pub(super) const BOARD: KnownBoard = KnownBoard::Gamma601;
    pub(super) const MARKER: &str = "firmware_board=601";
    pub(super) const TARGET: BoardTarget = BoardTarget::Gamma601;
    pub(super) const ASIC: AsicTarget = AsicTarget::Bm1370;
    pub(super) const ATTESTED: AttestedBoard = AttestedBoard::Gamma601;
    pub(super) const PLATFORM_BOARD: PlatformBoard = PlatformBoard::Gamma601;
    pub(super) const PLATFORM_ASIC: PlatformAsic = PlatformAsic::Bm1370;
    pub(super) const FAMILY: AsicFamily = AsicFamily::Bm1370;
    /// Every 601 build admits BM1370 dispatch since the promotion (ADR-0041 item 15) but the
    /// bounded vout, chip and hash checks (`build.rs`, `gamma601_dispatch`).
    pub(super) const DISPATCH_ADMITTED: bool = cfg!(gamma601_dispatch);
    /// The live voltage and frequency path drives the DS4432U and BM1366 frames; the 601 keeps
    /// its prepared profile until a BM1370 live path lands.
    pub(super) const LIVE_HARDWARE_CHANGES: bool = false;
    pub(super) const WORKER_MODEL: &str = "bitaxe-gamma";
    /// Signed by the owner's development update authority for this board (ADR-0041 item 10).
    pub(super) const WORKER_CAPABILITY: &str = include_str!("../bwg/gamma601-capability.json");
    /// The 601 never ran the 205's original acceptance campaign (ADR-0041 item 13).
    pub(super) const SOAK_PREREQUISITE: SoakPrerequisite = SoakPrerequisite::NoUnfinishedCampaign;
    /// The 601's self-test kind (ADR-0041 item 14).
    pub(super) const SELF_TEST_KIND: &str = "gamma601_full_v1";
    /// The TPS546's own throttle temperature, as the 601's safety verdict uses it.
    pub(super) const MAYBE_SELF_TEST_VR_MAX_C: Option<f32> =
        Some(bitaxe_safety::thermal::TPS546_VR_THROTTLE_TEMP_C as f32);
    pub(super) const fn catalog() -> BoardCatalogEntry {
        bitaxe_config::gamma_601_catalog_entry()
    }
}

/// The board this image was built for.
pub(crate) const BUILT: KnownBoard = built::BOARD;
const BUILD_MARKER: &str = built::MARKER;
/// The built board and ASIC for the boot line and the startup screen.
pub(crate) const BOARD_TARGET: BoardTarget = built::TARGET;
pub(crate) const ASIC_TARGET: AsicTarget = built::ASIC;
/// The board the runtime boot attestation names.
pub(crate) const ATTESTED_BOARD: AttestedBoard = built::ATTESTED;
/// The board and ASIC the platform identity reports.
pub(crate) const PLATFORM_BOARD: PlatformBoard = built::PLATFORM_BOARD;
pub(crate) const PLATFORM_ASIC: PlatformAsic = built::PLATFORM_ASIC;
/// The built board's chip family: its job-id step, job interval and result layout.
pub(crate) const ASIC_FAMILY: AsicFamily = built::FAMILY;
/// Whether this build may drive its ASIC for mining at all.
pub(crate) const MINING_DISPATCH_ADMITTED: bool = built::DISPATCH_ADMITTED;
/// Whether a lease may change its voltage and frequency live (ADR-0037 decision 4).
pub(crate) const LIVE_HARDWARE_CHANGES: bool = built::LIVE_HARDWARE_CHANGES;
/// Whether this build may start mining at boot from the saved preference. The 601's overheat test
/// build never does: a run starts only from an explicit resume, once its contract's pool and
/// profile are saved, never from a saved default pool at boot (ADR-0041 items 11 and 15).
pub(crate) const MINES_ON_BOOT: bool = !cfg!(feature = "gamma601-overheat-trip-test");
/// The board model the Worker capability and the pool subscription name, as the Gate's
/// reference-firmware board pairs spell it.
pub(crate) const WORKER_MODEL: &str = built::WORKER_MODEL;
/// The built board's signed Worker capability.
pub(crate) const WORKER_CAPABILITY: &str = built::WORKER_CAPABILITY;
/// What a soak allowance requires of the board's original acceptance campaign.
pub(crate) const SOAK_PREREQUISITE: SoakPrerequisite = built::SOAK_PREREQUISITE;
/// The kind a self-test admission tuple must name on this board.
pub(crate) const SELF_TEST_KIND: &str = built::SELF_TEST_KIND;

/// The built board's self-test contract, from its catalog entry (ADR-0041 item 14).
pub(crate) fn self_test_contract() -> bitaxe_safety::self_test::HardwareSelfTestContract {
    bitaxe_safety::self_test::HardwareSelfTestContract::for_board(
        SELF_TEST_KIND,
        catalog_entry(),
        built::MAYBE_SELF_TEST_VR_MAX_C,
    )
}
/// The ranges the built board's mining-safety facts must hold.
pub(crate) const MINING_SAFETY_LIMITS: MiningSafetyLimits =
    MiningSafetyLimits::for_board(built::catalog());
/// The built board's catalog version, such as "205".
pub(crate) const BOARD_VERSION: &str = built::catalog().board_version();

/// The built board's upstream `devicemodel` default, such as `ultra` or `gamma`, from its
/// `config-<board>.cvs` defaults; `None` only if the defaults lack the built board.
pub(crate) fn device_model() -> Option<&'static str> {
    bitaxe_config::board_profile_defaults()
        .iter()
        .find(|defaults| defaults.board_version() == BOARD_VERSION)
        .map(|defaults| defaults.device_model())
}

/// The built board's catalog entry, for the API and the screen.
pub(crate) const fn catalog_entry() -> BoardCatalogEntry {
    built::catalog()
}

/// The chip-address spacing results decode against: the 256 addresses shared over the chain.
pub(crate) fn result_address_interval() -> u16 {
    256 / u16::from(catalog_entry().asic_count())
}

static STORED: OnceLock<StoredBoard> = OnceLock::new();

/// Records what the stored `boardversion` names; later calls keep the first reading.
/// `settings_read` says whether the settings snapshot loaded.
pub(crate) fn initialize(settings_read: bool) {
    let snapshot = crate::settings_adapter::current_settings_snapshot();
    let stored = match snapshot.maybe_stored_value("boardversion") {
        None if !settings_read => StoredBoard::Unreadable,
        None => StoredBoard::Missing,
        Some(stored) => match &stored.value {
            StoredValueKind::String(value) => StoredBoard::from_stored(Some(value)),
            _ => StoredBoard::Other,
        },
    };
    let _ = STORED.set(stored);
    log::info!("{BUILD_MARKER}");
    log::info!("{}", marker());
}

/// What the stored `boardversion` names; `Missing` until read.
pub(crate) fn stored() -> StoredBoard {
    STORED.get().copied().unwrap_or(StoredBoard::Missing)
}

/// How the stored board relates to the built one.
pub(crate) fn identity() -> BoardIdentity {
    BoardIdentity::classify(BUILT, stored())
}

/// Whether hardware control beyond the safe-state actions may run.
pub(crate) fn hardware_control_permitted() -> bool {
    identity().permits_hardware_control()
}

/// The test-only overheat build's retained boot line (ADR-0040; ADR-0041 item 12): the build's
/// one variant marker, per board, and the lowered trip the overheat harness reads.
#[cfg(feature = "overheat-trip-test")]
pub(crate) fn overheat_trip_boot_line() -> String {
    #[cfg(not(feature = "board-gamma-601"))]
    const VARIANT_MARKER: &str = "firmware_variant=overheat-trip-test";
    #[cfg(feature = "board-gamma-601")]
    const VARIANT_MARKER: &str = "firmware_variant=gamma601-overheat-trip-test";
    format!(
        "{VARIANT_MARKER} local_overheat_trip_c={}",
        bitaxe_runtime::local_mining::LOCAL_OVERHEAT_TRIP_C
    )
}

/// The board's own periodic USB record, if its build has one: the Gamma 601 safe boot's.
#[cfg(feature = "board-gamma-601")]
pub(crate) fn maybe_board_runtime_marker() -> Option<String> {
    crate::gamma601::maybe_usb_marker()
}

#[cfg(not(feature = "board-gamma-601"))]
pub(crate) fn maybe_board_runtime_marker() -> Option<String> {
    None
}

/// The periodic USB record.
pub(crate) fn marker() -> String {
    bitaxe_core::board_identity::usb_marker(BUILT, stored())
}
