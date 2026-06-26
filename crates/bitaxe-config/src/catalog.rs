// Reference: reference/esp-miner/main/device_config.h

/// Evidence scope attached to a board catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationScope {
    /// Ultra 205 is the active V1 evidence target.
    ActiveUltra205,
    /// Catalog data exists, but no hardware evidence has verified this board.
    NotHardwareVerified,
}

/// ASIC profile values copied into typed, pure Rust data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsicProfile {
    model: &'static str,
    chip_id: u16,
    default_frequency_mhz: u16,
    frequency_options: &'static [u16],
    default_voltage_mv: u16,
    voltage_options: &'static [u16],
    core_count: u16,
    small_core_count: u16,
    hash_domains: u8,
    default_asic_timeout: u16,
}

impl AsicProfile {
    /// Returns the ASIC model name.
    #[must_use]
    pub const fn model(&self) -> &'static str {
        self.model
    }

    /// Returns the upstream chip identifier.
    #[must_use]
    pub const fn chip_id(&self) -> u16 {
        self.chip_id
    }

    /// Returns the default frequency in MHz.
    #[must_use]
    pub const fn default_frequency_mhz(&self) -> u16 {
        self.default_frequency_mhz
    }

    /// Returns supported frequency options in MHz.
    #[must_use]
    pub const fn frequency_options(&self) -> &'static [u16] {
        self.frequency_options
    }

    /// Returns the default voltage in millivolts.
    #[must_use]
    pub const fn default_voltage_mv(&self) -> u16 {
        self.default_voltage_mv
    }

    /// Returns supported voltage options in millivolts.
    #[must_use]
    pub const fn voltage_options(&self) -> &'static [u16] {
        self.voltage_options
    }

    /// Returns the normal core count.
    #[must_use]
    pub const fn core_count(&self) -> u16 {
        self.core_count
    }

    /// Returns the small-core count.
    #[must_use]
    pub const fn small_core_count(&self) -> u16 {
        self.small_core_count
    }

    /// Returns the hash-domain count.
    #[must_use]
    pub const fn hash_domains(&self) -> u8 {
        self.hash_domains
    }

    /// Returns the default ASIC timeout.
    #[must_use]
    pub const fn default_asic_timeout(&self) -> u16 {
        self.default_asic_timeout
    }
}

/// Hardware capability flags from the upstream board catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCapabilities {
    ds4432u: bool,
    ina260: bool,
    tps546: bool,
    plug_sense: bool,
    asic_enable: bool,
}

impl BoardCapabilities {
    /// Returns whether the board declares DS4432U support.
    #[must_use]
    pub const fn ds4432u(&self) -> bool {
        self.ds4432u
    }

    /// Returns whether the board declares INA260 support.
    #[must_use]
    pub const fn ina260(&self) -> bool {
        self.ina260
    }

    /// Returns whether the board declares TPS546 support.
    #[must_use]
    pub const fn tps546(&self) -> bool {
        self.tps546
    }

    /// Returns whether the board declares plug-sense support.
    #[must_use]
    pub const fn plug_sense(&self) -> bool {
        self.plug_sense
    }

    /// Returns whether the board declares ASIC-enable support.
    #[must_use]
    pub const fn asic_enable(&self) -> bool {
        self.asic_enable
    }
}

/// Board catalog entry with explicit evidence scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCatalogEntry {
    board_version: &'static str,
    family: &'static str,
    asic: AsicProfile,
    asic_count: u8,
    capabilities: BoardCapabilities,
    power_consumption_target: u16,
    verification_scope: VerificationScope,
}

impl BoardCatalogEntry {
    /// Returns the upstream board-version string.
    #[must_use]
    pub const fn board_version(&self) -> &'static str {
        self.board_version
    }

    /// Returns the upstream family name.
    #[must_use]
    pub const fn family(&self) -> &'static str {
        self.family
    }

    /// Returns the ASIC profile for this board family.
    #[must_use]
    pub const fn asic(&self) -> AsicProfile {
        self.asic
    }

    /// Returns the ASIC count for this board family.
    #[must_use]
    pub const fn asic_count(&self) -> u8 {
        self.asic_count
    }

    /// Returns board capability flags.
    #[must_use]
    pub const fn capabilities(&self) -> BoardCapabilities {
        self.capabilities
    }

    /// Returns the upstream power-consumption target.
    #[must_use]
    pub const fn power_consumption_target(&self) -> u16 {
        self.power_consumption_target
    }

    /// Returns this entry's evidence scope.
    #[must_use]
    pub const fn verification_scope(&self) -> VerificationScope {
        self.verification_scope
    }
}

const BM1397_FREQUENCY_OPTIONS: &[u16] = &[400, 425, 450, 475, 485, 500, 525, 550, 575, 600];
const BM1366_FREQUENCY_OPTIONS: &[u16] = &[400, 425, 450, 475, 485, 500, 525, 550, 575];
const BM1368_FREQUENCY_OPTIONS: &[u16] = &[400, 425, 450, 475, 485, 490, 500, 525, 550, 575];
const BM1370_FREQUENCY_OPTIONS: &[u16] = &[400, 490, 525, 550, 600, 625];
const BM1370_XP_FREQUENCY_OPTIONS: &[u16] = &[350, 375, 380, 400, 410];

const BM1397_VOLTAGE_OPTIONS: &[u16] = &[1100, 1150, 1200, 1250, 1300, 1350, 1400, 1450, 1500];
const BM1366_VOLTAGE_OPTIONS: &[u16] = &[1100, 1150, 1200, 1250, 1300];
const BM1368_VOLTAGE_OPTIONS: &[u16] = &[1100, 1150, 1166, 1200, 1250, 1300];
const BM1370_VOLTAGE_OPTIONS: &[u16] = &[1000, 1060, 1100, 1150, 1200, 1250];

const ASIC_BM1397: AsicProfile = AsicProfile {
    model: "BM1397",
    chip_id: 1397,
    default_frequency_mhz: 425,
    frequency_options: BM1397_FREQUENCY_OPTIONS,
    default_voltage_mv: 1400,
    voltage_options: BM1397_VOLTAGE_OPTIONS,
    core_count: 168,
    small_core_count: 672,
    hash_domains: 1,
    default_asic_timeout: 20,
};

const ASIC_BM1366: AsicProfile = AsicProfile {
    model: "BM1366",
    chip_id: 1366,
    default_frequency_mhz: 485,
    frequency_options: BM1366_FREQUENCY_OPTIONS,
    default_voltage_mv: 1200,
    voltage_options: BM1366_VOLTAGE_OPTIONS,
    core_count: 112,
    small_core_count: 894,
    hash_domains: 4,
    default_asic_timeout: 2000,
};

const ASIC_BM1368: AsicProfile = AsicProfile {
    model: "BM1368",
    chip_id: 1368,
    default_frequency_mhz: 490,
    frequency_options: BM1368_FREQUENCY_OPTIONS,
    default_voltage_mv: 1166,
    voltage_options: BM1368_VOLTAGE_OPTIONS,
    core_count: 80,
    small_core_count: 1276,
    hash_domains: 4,
    default_asic_timeout: 500,
};

const ASIC_BM1370: AsicProfile = AsicProfile {
    model: "BM1370",
    chip_id: 1370,
    default_frequency_mhz: 525,
    frequency_options: BM1370_FREQUENCY_OPTIONS,
    default_voltage_mv: 1150,
    voltage_options: BM1370_VOLTAGE_OPTIONS,
    core_count: 128,
    small_core_count: 2040,
    hash_domains: 4,
    default_asic_timeout: 500,
};

const ASIC_BM1370_XP: AsicProfile = AsicProfile {
    model: "BM1370XP",
    chip_id: 1370,
    default_frequency_mhz: 400,
    frequency_options: BM1370_XP_FREQUENCY_OPTIONS,
    default_voltage_mv: 1150,
    voltage_options: BM1370_VOLTAGE_OPTIONS,
    core_count: 128,
    small_core_count: 2040,
    hash_domains: 4,
    default_asic_timeout: 500,
};

const DS4432U_INA260_CAPABILITIES: BoardCapabilities = BoardCapabilities {
    ds4432u: true,
    ina260: true,
    tps546: false,
    plug_sense: true,
    asic_enable: true,
};

const DS4432U_INA260_NO_ASIC_ENABLE_CAPABILITIES: BoardCapabilities = BoardCapabilities {
    ds4432u: true,
    ina260: true,
    tps546: false,
    plug_sense: true,
    asic_enable: false,
};

const TPS546_CAPABILITIES: BoardCapabilities = BoardCapabilities {
    ds4432u: false,
    ina260: false,
    tps546: true,
    plug_sense: false,
    asic_enable: false,
};

const BOARD_CATALOG: &[BoardCatalogEntry] = &[
    entry(
        "2.2",
        "Max",
        ASIC_BM1397,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "102",
        "Max",
        ASIC_BM1397,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "0.11",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "201",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "202",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "203",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "204",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_NO_ASIC_ENABLE_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "205",
        "Ultra",
        ASIC_BM1366,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::ActiveUltra205,
    ),
    entry(
        "207",
        "Ultra",
        ASIC_BM1366,
        1,
        TPS546_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "302",
        "Hex",
        ASIC_BM1366,
        6,
        TPS546_CAPABILITIES,
        40,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "303",
        "Hex",
        ASIC_BM1366,
        6,
        TPS546_CAPABILITIES,
        40,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "400",
        "Supra",
        ASIC_BM1368,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "401",
        "Supra",
        ASIC_BM1368,
        1,
        DS4432U_INA260_CAPABILITIES,
        12,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "402",
        "Supra",
        ASIC_BM1368,
        1,
        TPS546_CAPABILITIES,
        8,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "403",
        "Supra",
        ASIC_BM1368,
        1,
        TPS546_CAPABILITIES,
        8,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "600",
        "Gamma",
        ASIC_BM1370,
        1,
        TPS546_CAPABILITIES,
        19,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "601",
        "Gamma",
        ASIC_BM1370,
        1,
        TPS546_CAPABILITIES,
        19,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "602",
        "Gamma",
        ASIC_BM1370,
        1,
        TPS546_CAPABILITIES,
        22,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "603",
        "Gamma",
        ASIC_BM1370,
        1,
        TPS546_CAPABILITIES,
        22,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "650",
        "GammaDuo",
        ASIC_BM1370_XP,
        2,
        TPS546_CAPABILITIES,
        35,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "701",
        "SupraHex",
        ASIC_BM1368,
        6,
        TPS546_CAPABILITIES,
        90,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "702",
        "SupraHex",
        ASIC_BM1368,
        6,
        TPS546_CAPABILITIES,
        90,
        VerificationScope::NotHardwareVerified,
    ),
    entry(
        "801",
        "GammaTurbo",
        ASIC_BM1370,
        2,
        TPS546_CAPABILITIES,
        36,
        VerificationScope::NotHardwareVerified,
    ),
];

const fn entry(
    board_version: &'static str,
    family: &'static str,
    asic: AsicProfile,
    asic_count: u8,
    capabilities: BoardCapabilities,
    power_consumption_target: u16,
    verification_scope: VerificationScope,
) -> BoardCatalogEntry {
    BoardCatalogEntry {
        board_version,
        family,
        asic,
        asic_count,
        capabilities,
        power_consumption_target,
        verification_scope,
    }
}

/// Returns the Ultra 205 board catalog entry.
#[must_use]
pub const fn ultra_205_catalog_entry() -> BoardCatalogEntry {
    BOARD_CATALOG[7]
}

/// Returns every upstream board catalog entry represented by this phase.
#[must_use]
pub const fn board_catalog() -> &'static [BoardCatalogEntry] {
    BOARD_CATALOG
}
