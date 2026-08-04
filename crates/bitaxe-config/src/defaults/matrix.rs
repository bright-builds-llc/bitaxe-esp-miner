/// Whether an upstream defaults seed is a normal numbered profile or the
/// explicit custom override template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardProfileSeedKind {
    /// A numbered seed that can be selected by its board version.
    Numbered,
    /// The upstream custom template, which must never shadow a numbered seed.
    CustomOverride,
}

/// Exact board-profile discriminators from one pinned upstream CSV seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardProfileDefaults {
    seed_id: &'static str,
    source_path: &'static str,
    seed_kind: BoardProfileSeedKind,
    board_version: &'static str,
    device_model: &'static str,
    asic_model: &'static str,
    asic_frequency_mhz: u16,
    asic_voltage_mv: u16,
    rotation: u16,
    auto_fan_speed: bool,
    manual_fan_speed: u16,
    self_test: bool,
    overheat_mode: bool,
    primary_pool_port: u16,
}

impl BoardProfileDefaults {
    /// Returns the stable seed identifier (`102` through `801`, or `custom`).
    #[must_use]
    pub const fn seed_id(&self) -> &'static str {
        self.seed_id
    }

    /// Returns the pinned-reference source path for this seed.
    #[must_use]
    pub const fn source_path(&self) -> &'static str {
        self.source_path
    }

    /// Returns the seed classification.
    #[must_use]
    pub const fn seed_kind(&self) -> BoardProfileSeedKind {
        self.seed_kind
    }

    /// Returns whether normal board-version selection may choose this seed.
    #[must_use]
    pub const fn is_selectable(&self) -> bool {
        matches!(self.seed_kind, BoardProfileSeedKind::Numbered)
    }

    /// Returns the board-version discriminator.
    #[must_use]
    pub const fn board_version(&self) -> &'static str {
        self.board_version
    }

    /// Returns the upstream device-model discriminator.
    #[must_use]
    pub const fn device_model(&self) -> &'static str {
        self.device_model
    }

    /// Returns the upstream ASIC-model discriminator.
    #[must_use]
    pub const fn asic_model(&self) -> &'static str {
        self.asic_model
    }

    /// Returns the seeded ASIC frequency in MHz.
    #[must_use]
    pub const fn asic_frequency_mhz(&self) -> u16 {
        self.asic_frequency_mhz
    }

    /// Returns the seeded ASIC voltage in millivolts.
    #[must_use]
    pub const fn asic_voltage_mv(&self) -> u16 {
        self.asic_voltage_mv
    }

    /// Returns the display rotation seed.
    #[must_use]
    pub const fn rotation(&self) -> u16 {
        self.rotation
    }

    /// Returns whether automatic fan control is seeded on.
    #[must_use]
    pub const fn auto_fan_speed(&self) -> bool {
        self.auto_fan_speed
    }

    /// Returns the manual fan-speed seed.
    #[must_use]
    pub const fn manual_fan_speed(&self) -> u16 {
        self.manual_fan_speed
    }

    /// Returns whether boot self-test is seeded on.
    #[must_use]
    pub const fn self_test(&self) -> bool {
        self.self_test
    }

    /// Returns whether overheat mode is seeded on.
    #[must_use]
    pub const fn overheat_mode(&self) -> bool {
        self.overheat_mode
    }

    /// Returns the primary pool port seed.
    #[must_use]
    pub const fn primary_pool_port(&self) -> u16 {
        self.primary_pool_port
    }
}

const COMMON_ROTATION: u16 = 0;
const COMMON_AUTO_FAN_SPEED: bool = true;
const COMMON_MANUAL_FAN_SPEED: u16 = 100;
const COMMON_SELF_TEST: bool = true;
const COMMON_OVERHEAT_MODE: bool = false;
const COMMON_POOL_PORT: u16 = 3333;

macro_rules! numbered {
    ($board_version:literal, $device_model:literal, $asic_model:literal, $frequency:literal, $voltage:literal) => {
        BoardProfileDefaults {
            seed_id: $board_version,
            source_path: concat!("reference/esp-miner/config-", $board_version, ".cvs"),
            seed_kind: BoardProfileSeedKind::Numbered,
            board_version: $board_version,
            device_model: $device_model,
            asic_model: $asic_model,
            asic_frequency_mhz: $frequency,
            asic_voltage_mv: $voltage,
            rotation: COMMON_ROTATION,
            auto_fan_speed: COMMON_AUTO_FAN_SPEED,
            manual_fan_speed: COMMON_MANUAL_FAN_SPEED,
            self_test: COMMON_SELF_TEST,
            overheat_mode: COMMON_OVERHEAT_MODE,
            primary_pool_port: COMMON_POOL_PORT,
        }
    };
}

const BOARD_PROFILE_DEFAULTS: &[BoardProfileDefaults] = &[
    numbered!("102", "max", "BM1397", 425, 1400),
    numbered!("201", "ultra", "BM1366", 485, 1200),
    numbered!("202", "ultra", "BM1366", 485, 1200),
    numbered!("203", "ultra", "BM1366", 485, 1200),
    numbered!("204", "ultra", "BM1366", 485, 1200),
    numbered!("205", "ultra", "BM1366", 485, 1200),
    numbered!("207", "ultra", "BM1366", 485, 1200),
    numbered!("302", "hex", "BM1366", 485, 1200),
    numbered!("303", "hex", "BM1366", 485, 1200),
    numbered!("400", "supra", "BM1368", 490, 1166),
    numbered!("401", "supra", "BM1368", 490, 1166),
    numbered!("402", "supra", "BM1368", 490, 1166),
    numbered!("403", "supra", "BM1368", 490, 1166),
    numbered!("601", "gamma", "BM1370", 525, 1150),
    numbered!("602", "gamma", "BM1370", 525, 1150),
    numbered!("603", "gamma", "BM1370", 525, 1150),
    numbered!("650", "gammaduo", "BM1370", 400, 1150),
    numbered!("701", "suprahex", "BM1368", 490, 1166),
    numbered!("702", "suprahex", "BM1368", 490, 1166),
    numbered!("801", "gammaturbo", "BM1370", 525, 1150),
    BoardProfileDefaults {
        seed_id: "custom",
        source_path: "reference/esp-miner/config-custom.cvs",
        seed_kind: BoardProfileSeedKind::CustomOverride,
        board_version: "207",
        device_model: "ultra",
        asic_model: "BM1366",
        asic_frequency_mhz: 485,
        asic_voltage_mv: 1200,
        rotation: COMMON_ROTATION,
        auto_fan_speed: COMMON_AUTO_FAN_SPEED,
        manual_fan_speed: COMMON_MANUAL_FAN_SPEED,
        self_test: COMMON_SELF_TEST,
        overheat_mode: COMMON_OVERHEAT_MODE,
        primary_pool_port: 21496,
    },
];

/// Returns the exact board-profile defaults from all pinned upstream seeds.
#[must_use]
pub const fn board_profile_defaults() -> &'static [BoardProfileDefaults] {
    BOARD_PROFILE_DEFAULTS
}

#[cfg(test)]
mod tests;
