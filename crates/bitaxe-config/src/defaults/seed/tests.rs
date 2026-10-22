use super::*;
use crate::StoredValueKind;

/// The fixture's rows, minus the namespace row and the Wi-Fi rows a factory install supplies.
fn fixture_rows(fixture: &str) -> Vec<(String, String, String)> {
    fixture
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .skip(1)
        .map(|line| {
            let fields: Vec<&str> = line.splitn(4, ',').collect();
            (
                fields[0].to_owned(),
                fields[2].to_owned(),
                fields[3].to_owned(),
            )
        })
        .filter(|(key, encoding, _)| !encoding.is_empty() && key != "wifissid" && key != "wifipass")
        .collect()
}

fn seed_rows(values: &[StoredValue]) -> Vec<(String, String, String)> {
    values
        .iter()
        .map(|value| {
            let (encoding, text) = match &value.value {
                StoredValueKind::String(text) => ("string", text.clone()),
                StoredValueKind::U16(number) => ("u16", number.to_string()),
                StoredValueKind::I32(number) => ("i32", number.to_string()),
                StoredValueKind::U64(number) => ("u64", number.to_string()),
            };
            (value.key.as_str().to_owned(), encoding.to_owned(), text)
        })
        .collect()
}

fn sorted(mut rows: Vec<(String, String, String)>) -> Vec<(String, String, String)> {
    rows.sort();
    rows
}

#[test]
fn the_601_seed_matches_its_upstream_config() {
    // Arrange
    let fixture = fixture_rows(include_str!("../../../fixtures/gamma-601-defaults.csv"));
    // Act
    let seed = seed_rows(&board_default_seed_values("601").expect("601 seed"));
    // Assert
    assert_eq!(sorted(seed), sorted(fixture));
}

#[test]
fn the_205_seed_is_unchanged() {
    // Act
    let seed = board_default_seed_values("205").expect("205 seed");
    // Assert
    assert_eq!(seed, ultra_205_default_seed_values());
}

#[test]
fn the_205_seed_matches_its_upstream_config() {
    // Arrange
    let fixture = fixture_rows(include_str!("../../../fixtures/ultra-205-defaults.csv"));
    // Act
    let seed = seed_rows(&board_default_seed_values("205").expect("205 seed"));
    // Assert
    assert_eq!(sorted(seed), sorted(fixture));
}

#[test]
fn a_board_this_project_does_not_build_has_no_seed() {
    // Act
    let seed = board_default_seed_values("602");
    // Assert
    assert_eq!(seed, None);
}
