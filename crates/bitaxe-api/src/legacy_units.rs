const MILLI_UNITS_PER_UNIT: f64 = 1_000.0;

pub(crate) const fn millivolts_from_volts(volts: f64) -> f64 {
    volts * MILLI_UNITS_PER_UNIT
}

pub(crate) const fn milliamps_from_amps(amps: f64) -> f64 {
    amps * MILLI_UNITS_PER_UNIT
}
