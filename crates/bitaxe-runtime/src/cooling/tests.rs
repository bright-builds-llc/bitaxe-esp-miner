use super::*;
use bitaxe_config::catalog::gamma_601_catalog_entry;
use bitaxe_safety::observation::{
    BootSessionId, Observation, ObservationSequence, UnavailableReason,
};

const WINDOW_START_MS: u64 = 10_000;

#[derive(Clone, Copy)]
enum Chip {
    Celsius(f64),
    Unavailable(UnavailableReason),
}

fn sampled<T>(value: T, sequence: u64, at_ms: u64) -> Observation<T> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(sequence),
        MonotonicMillis::new(at_ms),
    )
    .expect("fixture sequence should advance")
    .0
}

/// Mining-safe facts for either board, all taken at `at_ms`.
fn observations(chip: Chip, vr_celsius: f64, sequence: u64, at_ms: u64) -> TelemetryObservations {
    TelemetryObservations {
        power_watts: sampled(6.0, sequence, at_ms),
        bus_voltage_volts: sampled(5.0, sequence, at_ms),
        current_amps: sampled(1.0, sequence, at_ms),
        core_voltage_actual_mv: sampled(0.0, sequence, at_ms),
        chip_temp_celsius: match chip {
            Chip::Celsius(celsius) => sampled(celsius, sequence, at_ms),
            Chip::Unavailable(reason) => Observation::unavailable(reason),
        },
        vr_temp_celsius: sampled(vr_celsius, sequence, at_ms),
        fan_rpm: sampled(3_000, sequence, at_ms),
    }
}

fn gamma_601() -> MiningSafetyLimits {
    MiningSafetyLimits::for_board(gamma_601_catalog_entry())
}

const UNPOWERED: Chip = Chip::Unavailable(UnavailableReason::AsicUnpowered);
const NOT_INITIALIZED: Chip = Chip::Unavailable(UnavailableReason::AsicNotInitialized);

#[test]
fn the_ultra_205_baseline_is_its_chip_at_45_c() {
    // Act
    let safe = [45.0, 45.1].map(|celsius| {
        baseline_safe(
            &MiningSafetyLimits::ultra_205(),
            &observations(Chip::Celsius(celsius), 0.0, 1, 1_000),
            1_000,
        )
    });
    // Assert
    assert_eq!(safe, [true, false]);
}

#[test]
fn the_gamma_601_baseline_needs_its_chip_unpowered_and_its_regulator_at_95_c() {
    // Arrange
    let cases = [
        (UNPOWERED, 95.0),
        (UNPOWERED, 95.1),
        (NOT_INITIALIZED, 40.0),
    ];
    // Act
    let safe = cases.map(|(chip, vr_celsius)| {
        baseline_safe(
            &gamma_601(),
            &observations(chip, vr_celsius, 1, 1_000),
            1_000,
        )
    });
    // Assert
    assert_eq!(safe, [true, false, false]);
}

#[test]
fn the_ultra_205_stop_proof_needs_only_a_cool_reading_after_the_window_opened() {
    // Arrange
    let limits = MiningSafetyLimits::ultra_205();
    let readings_at = [WINDOW_START_MS, WINDOW_START_MS + 1];
    // Act
    let proven = readings_at.map(|at_ms| {
        let observations = observations(Chip::Celsius(44.0), 0.0, 1, at_ms);
        stop_cooling_proven(&limits, &observations, WINDOW_START_MS, at_ms + 1)
    });
    // Assert
    assert_eq!(proven, [false, true]);
}

#[test]
fn the_gamma_601_stop_proof_needs_30_s_since_the_window_opened() {
    // Arrange
    let nows = [
        WINDOW_START_MS + ASIC_DIODE_STOP_COOLING_MS - 1,
        WINDOW_START_MS + ASIC_DIODE_STOP_COOLING_MS,
    ];
    // Act
    let proven = nows.map(|now_ms| {
        let observations = observations(UNPOWERED, 60.0, 1, now_ms);
        stop_cooling_proven(&gamma_601(), &observations, WINDOW_START_MS, now_ms)
    });
    // Assert
    assert_eq!(proven, [false, true]);
}

#[test]
fn the_gamma_601_stop_proof_needs_its_sample_after_the_window_opened() {
    // Arrange
    let now_ms = WINDOW_START_MS + ASIC_DIODE_STOP_COOLING_MS;
    let observations = observations(UNPOWERED, 60.0, 1, WINDOW_START_MS);
    // Act
    let proven = stop_cooling_proven(&gamma_601(), &observations, WINDOW_START_MS, now_ms);
    // Assert
    assert!(!proven);
}

#[test]
fn the_gamma_601_stop_proof_needs_its_chip_unpowered() {
    // Arrange
    let now_ms = WINDOW_START_MS + ASIC_DIODE_STOP_COOLING_MS;
    let observations = observations(NOT_INITIALIZED, 60.0, 1, now_ms);
    // Act
    let proven = stop_cooling_proven(&gamma_601(), &observations, WINDOW_START_MS, now_ms);
    // Assert
    assert!(!proven);
}

/// A fan that applies its command at once and reports a new sample every 250 ms.
struct Backend {
    limits: MiningSafetyLimits,
    chip: Chip,
    now_ms: u64,
    sequence: u64,
    commanded: bool,
}

impl Backend {
    fn new(limits: MiningSafetyLimits, chip: Chip) -> Self {
        Self {
            limits,
            chip,
            now_ms: 1_000,
            sequence: 1,
            commanded: false,
        }
    }
}

impl CoolingBackend for Backend {
    fn now_ms(&self) -> u64 {
        self.now_ms
    }
    fn admitted(&self) -> bool {
        true
    }
    fn safety_limits(&self) -> MiningSafetyLimits {
        self.limits
    }
    fn observations(&self) -> TelemetryObservations {
        observations(self.chip, 50.0, self.sequence, self.now_ms)
    }
    fn command(&mut self, _command: FanCommand) -> Result<(), CoolingError> {
        self.commanded = true;
        Ok(())
    }
    fn poll(&mut self) -> Result<FanReply, CoolingError> {
        Ok(if self.commanded {
            FanReply::Applied
        } else {
            FanReply::Pending
        })
    }
    fn wait(&mut self) {
        self.now_ms += 250;
        self.sequence += 1;
    }
}

#[test]
fn a_gamma_601_with_its_chip_unpowered_qualifies() {
    // Arrange
    let mut backend = Backend::new(gamma_601(), UNPOWERED);
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert_eq!(result, Ok(3_000));
}

#[test]
fn a_gamma_601_with_its_chip_powered_is_not_admitted() {
    // Arrange: a reboot that left the regulator on.
    let mut backend = Backend::new(gamma_601(), NOT_INITIALIZED);
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert_eq!(result, Err(CoolingError::Rejected));
}

#[test]
fn the_ultra_205_qualifies_on_mining_safe_facts_alone() {
    // Arrange: a warm chip below the 75 C limit, as before.
    let mut backend = Backend::new(MiningSafetyLimits::ultra_205(), Chip::Celsius(60.0));
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert_eq!(result, Ok(3_000));
}

#[test]
fn the_ultra_205_rejects_an_unpowered_chip_reason() {
    // Arrange
    let mut backend = Backend::new(MiningSafetyLimits::ultra_205(), UNPOWERED);
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert_eq!(result, Err(CoolingError::Rejected));
}
