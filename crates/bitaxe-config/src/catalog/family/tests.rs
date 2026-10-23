use super::*;
use crate::catalog::{board_catalog, gamma_601_catalog_entry, ultra_205_catalog_entry};

#[test]
fn every_catalog_board_has_family_facts() {
    // Act
    let facts: Vec<FamilyFacts> = board_catalog()
        .iter()
        .map(BoardCatalogEntry::family_facts)
        .collect();
    // Assert
    assert_eq!(facts.len(), board_catalog().len());
}

#[test]
fn the_gamma_family_matches_upstream() {
    // Act
    let facts = gamma_601_catalog_entry().family_facts();
    // Assert
    assert_eq!(facts, facts_for(40, 5, 5));
}

#[test]
fn the_ultra_family_matches_upstream() {
    // Act
    let facts = ultra_205_catalog_entry().family_facts();
    // Assert
    assert_eq!(facts, facts_for(25, 5, 5));
}

#[test]
fn only_tps546_boards_carry_a_core_voltage_ceiling() {
    // Act
    let ceilings = [ultra_205_catalog_entry(), gamma_601_catalog_entry()]
        .map(|entry| entry.maybe_core_voltage_ceiling_mv());
    // Assert
    assert_eq!(ceilings, [None, Some(1_200)]);
}

fn facts_for(
    max_power_watts: u16,
    power_offset_watts: u16,
    nominal_voltage_volts: u16,
) -> FamilyFacts {
    FamilyFacts {
        max_power_watts,
        power_offset_watts,
        nominal_voltage_volts,
    }
}
