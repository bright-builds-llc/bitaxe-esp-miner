// Reference: reference/esp-miner/config-205.cvs

/// Upstream pool defaults for an Ultra 205 configuration seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolDefaults {
    url: &'static str,
    port: u16,
    tls: u16,
    cert: &'static str,
    user: &'static str,
    password: &'static str,
    difficulty: u16,
    extranonce_subscribe: u16,
}

impl PoolDefaults {
    /// Returns the upstream pool URL.
    #[must_use]
    pub const fn url(&self) -> &'static str {
        self.url
    }

    /// Returns the upstream pool port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the upstream TLS mode value.
    #[must_use]
    pub const fn tls(&self) -> u16 {
        self.tls
    }

    /// Returns the upstream certificate value.
    #[must_use]
    pub const fn cert(&self) -> &'static str {
        self.cert
    }

    /// Returns the upstream public pool user value.
    #[must_use]
    pub const fn user(&self) -> &'static str {
        self.user
    }

    /// Returns the upstream pool password value.
    #[must_use]
    pub const fn password(&self) -> &'static str {
        self.password
    }

    /// Returns the upstream suggested difficulty.
    #[must_use]
    pub const fn difficulty(&self) -> u16 {
        self.difficulty
    }

    /// Returns the upstream extranonce-subscribe flag value.
    #[must_use]
    pub const fn extranonce_subscribe(&self) -> u16 {
        self.extranonce_subscribe
    }
}

/// Exact Ultra 205 defaults seeded by the pinned upstream reference file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ultra205Defaults {
    hostname: &'static str,
    primary_pool: PoolDefaults,
    fallback_pool: PoolDefaults,
    asic_frequency_mhz: u16,
    asic_voltage_mv: u16,
    asic_model: &'static str,
    device_model: &'static str,
    board_version: &'static str,
    rotation: u16,
    auto_fan_speed: bool,
    manual_fan_speed: u16,
    self_test: bool,
    overheat_mode: bool,
}

impl Ultra205Defaults {
    /// Returns the upstream hostname default.
    #[must_use]
    pub const fn hostname(&self) -> &'static str {
        self.hostname
    }

    /// Returns the primary pool defaults.
    #[must_use]
    pub const fn primary_pool(&self) -> PoolDefaults {
        self.primary_pool
    }

    /// Returns the fallback pool defaults.
    #[must_use]
    pub const fn fallback_pool(&self) -> PoolDefaults {
        self.fallback_pool
    }

    /// Returns the ASIC frequency default in MHz.
    #[must_use]
    pub const fn asic_frequency_mhz(&self) -> u16 {
        self.asic_frequency_mhz
    }

    /// Returns the ASIC voltage default in millivolts.
    #[must_use]
    pub const fn asic_voltage_mv(&self) -> u16 {
        self.asic_voltage_mv
    }

    /// Returns the ASIC model default.
    #[must_use]
    pub const fn asic_model(&self) -> &'static str {
        self.asic_model
    }

    /// Returns the device model default.
    #[must_use]
    pub const fn device_model(&self) -> &'static str {
        self.device_model
    }

    /// Returns the board version default.
    #[must_use]
    pub const fn board_version(&self) -> &'static str {
        self.board_version
    }

    /// Returns the display rotation default.
    #[must_use]
    pub const fn rotation(&self) -> u16 {
        self.rotation
    }

    /// Returns whether automatic fan speed is enabled by default.
    #[must_use]
    pub const fn auto_fan_speed(&self) -> bool {
        self.auto_fan_speed
    }

    /// Returns the manual fan speed default.
    #[must_use]
    pub const fn manual_fan_speed(&self) -> u16 {
        self.manual_fan_speed
    }

    /// Returns whether self-test is enabled by default.
    #[must_use]
    pub const fn self_test(&self) -> bool {
        self.self_test
    }

    /// Returns whether overheat mode is enabled by default.
    #[must_use]
    pub const fn overheat_mode(&self) -> bool {
        self.overheat_mode
    }
}

const PUBLIC_POOL_USER: &str =
    "bc1qnp980s5fpp8l94p5cvttmtdqy8rvrq74qly2yrfmzkdsntqzlc5qkc4rkq.bitaxe";

const ULTRA_205_DEFAULTS: Ultra205Defaults = Ultra205Defaults {
    hostname: "bitaxe",
    primary_pool: PoolDefaults {
        url: "public-pool.io",
        port: 3333,
        tls: 0,
        cert: "x",
        user: PUBLIC_POOL_USER,
        password: "x",
        difficulty: 1000,
        extranonce_subscribe: 0,
    },
    fallback_pool: PoolDefaults {
        url: "solo.ckpool.org",
        port: 3333,
        tls: 0,
        cert: "x",
        user: PUBLIC_POOL_USER,
        password: "x",
        difficulty: 1000,
        extranonce_subscribe: 0,
    },
    asic_frequency_mhz: 485,
    asic_voltage_mv: 1200,
    asic_model: "BM1366",
    device_model: "ultra",
    board_version: "205",
    rotation: 0,
    auto_fan_speed: true,
    manual_fan_speed: 100,
    self_test: true,
    overheat_mode: false,
};

/// Returns the exact Ultra 205 defaults from the pinned reference seed file.
#[must_use]
pub const fn ultra_205_defaults() -> Ultra205Defaults {
    ULTRA_205_DEFAULTS
}
