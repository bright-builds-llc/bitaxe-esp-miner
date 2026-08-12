use super::*;

const NUMBERED_DOCUMENT: &str = "key,type,encoding,value\n\
main,namespace,,\n\
stratumport,data,u16,3333\n\
asicfrequency,data,u16,485\n\
asicvoltage,data,u16,1200\n\
asicmodel,data,string,BM1366\n\
devicemodel,data,string,ultra\n\
boardversion,data,string,205\n\
rotation,data,u16,0\n\
autofanspeed,data,u16,1\n\
fanspeed,data,u16,100\n\
selftest,data,u16,1\n\
overheat_mode,data,u16,0\n";

#[test]
fn parser_accepts_exact_numbered_seed_projection() {
    // Arrange
    let source_path = "reference/esp-miner/config-205.cvs";

    // Act
    let projection = parse_reference_seed(source_path, NUMBERED_DOCUMENT)
        .expect("production-shaped numbered seed must parse");

    // Assert
    assert_eq!(projection, expected_seed(source_path));
}

#[test]
fn parser_rejects_missing_required_field() {
    // Arrange
    let document = NUMBERED_DOCUMENT.replace("asicmodel,data,string,BM1366\n", "");

    // Act
    let error = parse_reference_seed("reference/esp-miner/config-205.cvs", &document)
        .expect_err("missing ASIC model must fail closed");

    // Assert
    assert!(error.contains("missing key asicmodel"));
}

#[test]
fn parser_rejects_wrong_field_encoding() {
    // Arrange
    let document = NUMBERED_DOCUMENT.replace(
        "asicfrequency,data,u16,485",
        "asicfrequency,data,string,485",
    );

    // Act
    let error = parse_reference_seed("reference/esp-miner/config-205.cvs", &document)
        .expect_err("wrong encoding must fail closed");

    // Assert
    assert!(error.contains("expected data/u16, found data/string"));
}

#[test]
fn parser_rejects_noncanonical_numeric_value() {
    // Arrange
    let document =
        NUMBERED_DOCUMENT.replace("asicfrequency,data,u16,485", "asicfrequency,data,u16,0485");

    // Act
    let error = parse_reference_seed("reference/esp-miner/config-205.cvs", &document)
        .expect_err("noncanonical integer must fail closed");

    // Assert
    assert!(error.contains("key asicfrequency is not a canonical u16"));
}

#[test]
fn parser_rejects_duplicate_key() {
    // Arrange
    let document = format!("{NUMBERED_DOCUMENT}rotation,data,u16,0\n");

    // Act
    let error = parse_reference_seed("reference/esp-miner/config-205.cvs", &document)
        .expect_err("duplicate field must fail closed");

    // Assert
    assert!(error.contains("duplicate key rotation"));
}

#[test]
fn projection_validation_accepts_exact_closed_inventory() {
    // Arrange
    let expected = vec![expected_seed("reference/esp-miner/config-205.cvs")];
    let actual = expected.clone();

    // Act
    let errors = validate_projections(&expected, &actual);

    // Assert
    assert!(errors.is_empty());
}

#[test]
fn projection_validation_rejects_missing_and_extra_sources() {
    // Arrange
    let expected = vec![expected_seed("reference/esp-miner/config-205.cvs")];
    let actual = vec![expected_seed("reference/esp-miner/config-207.cvs")];

    // Act
    let errors = validate_projections(&expected, &actual);

    // Assert
    assert!(errors
        .iter()
        .any(|error| error.contains("missing reference/esp-miner/config-205.cvs")));
    assert!(errors
        .iter()
        .any(|error| error.contains("unmodeled seed reference/esp-miner/config-207.cvs")));
}

#[test]
fn projection_validation_rejects_value_drift() {
    // Arrange
    let expected = vec![expected_seed("reference/esp-miner/config-205.cvs")];
    let mut drifted = expected[0].clone();
    drifted.asic_voltage_mv = 1250;

    // Act
    let errors = validate_projections(&expected, &[drifted]);

    // Assert
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("does not match pinned reference seed"));
}

#[test]
fn projection_validation_rejects_duplicate_seed_identity() {
    // Arrange
    let seed = expected_seed("reference/esp-miner/config-205.cvs");
    let expected = vec![seed.clone(), seed];

    // Act
    let errors = validate_projections(&expected, &[]);

    // Assert
    assert!(errors
        .iter()
        .any(|error| error.contains("duplicate seed id 205")));
    assert!(errors
        .iter()
        .any(|error| error.contains("duplicate source path")));
}

fn expected_seed(source_path: &str) -> SeedProjection {
    let seed_id = source_path
        .strip_prefix("reference/esp-miner/config-")
        .and_then(|value| value.strip_suffix(".cvs"))
        .expect("test source path must use reference seed shape");
    SeedProjection {
        seed_id: seed_id.to_owned(),
        source_path: source_path.to_owned(),
        seed_kind: SeedKind::Numbered,
        board_version: "205".to_owned(),
        device_model: "ultra".to_owned(),
        asic_model: "BM1366".to_owned(),
        asic_frequency_mhz: 485,
        asic_voltage_mv: 1200,
        rotation: 0,
        auto_fan_speed: true,
        manual_fan_speed: 100,
        self_test: true,
        overheat_mode: false,
        primary_pool_port: 3333,
    }
}
