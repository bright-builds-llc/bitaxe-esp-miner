use super::*;

mod fixtures;

use esp_idf_part::Flags;
use fixtures::*;
use sha2::{Digest, Sha256};
use std::ops::Range;

const ESP_IMAGE_MAGIC: u8 = 0xe9;
const ESP_IMAGE_HEADER_LEN: usize = 24;
const ESP_SEGMENT_HEADER_LEN: usize = 8;
const ESP32_S3_CHIP_ID: u16 = 9;
const SUPPORTED_MAX_CHIP_REV_FULL: u16 = 99;
const ESP_IMAGE_CHECKSUM_SEED: u8 = 0xef;
const ESP_APP_DESCRIPTOR_MAGIC: u32 = 0xABCD_5432;
const ESP_APP_DESCRIPTOR_LEN: usize = 256;
const APP_VERSION_OFFSET: usize = 16;
const APP_ELF_SHA_OFFSET: usize = 144;
const APP_ELF_SHA_LEN: usize = 32;

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const BUILD_LABEL: &str = "0123456789ab-dev";
const APP_SHA: [u8; 32] = [0x5a; 32];

#[test]
fn package_admission_accepts_matching_structural_images() {
    // Arrange
    let ota = ota_fixture();
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let result = validate_fixture(&factory, &ota);

    // Assert
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn package_admission_rejects_non_drom_descriptor_in_ota_and_factory() {
    assert_layout_rejected_at_ota_and_factory(
        LayoutFixtureKind::DescriptorNotDrom,
        "app_descriptor_segment_not_drom",
    );
}

#[test]
fn package_admission_rejects_destination_overlap_in_ota_and_factory() {
    assert_layout_rejected_at_ota_and_factory(
        LayoutFixtureKind::DestinationOverlap,
        "ota_segment_destination_overlap",
    );
}

#[test]
fn package_admission_rejects_alias_overlap_in_ota_and_factory() {
    assert_layout_rejected_at_ota_and_factory(
        LayoutFixtureKind::AliasOverlap,
        "ota_segment_alias_overlap",
    );
}

#[test]
fn package_admission_accepts_exact_destination_and_alias_adjacency() {
    // Arrange
    let direct = layout_fixture(LayoutFixtureKind::DestinationAdjacent);
    let alias = layout_fixture(LayoutFixtureKind::AliasAdjacent);

    // Act
    let direct_result = validate_fixture(&factory_fixture(&factory_table(), &direct), &direct);
    let alias_result = validate_fixture(&factory_fixture(&factory_table(), &alias), &alias);

    // Assert
    assert!(direct_result.is_ok(), "{direct_result:#?}");
    assert!(alias_result.is_ok(), "{alias_result:#?}");
}

#[test]
fn package_admission_accepts_range_free_zero_length_segment() {
    // Arrange
    let ota = layout_fixture(LayoutFixtureKind::ZeroLengthInsideRange);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let result = validate_fixture(&factory, &ota);

    // Assert
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn package_admission_rejects_unsupported_spi_header() {
    // Arrange
    let mut ota = ota_fixture();
    ota[2] = 0;
    reseal_image(&mut ota);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("unsupported SPI mode");

    // Assert
    assert!(error.to_string().contains("ota_header_policy_unsupported"));
}

#[test]
fn package_admission_rejects_unaligned_entry_point() {
    // Arrange
    let mut ota = ota_fixture();
    ota[4..8].copy_from_slice(&0x4037_4001_u32.to_le_bytes());
    reseal_image(&mut ota);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("unaligned entry point");

    // Assert
    assert!(error.to_string().contains("ota_entry_address_unaligned"));
}

#[test]
fn package_admission_rejects_nonempty_low_load_address() {
    // Arrange
    let mut ota = ota_fixture();
    ota[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_image(&mut ota);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("low load address");

    // Assert
    assert!(error
        .to_string()
        .contains("ota_segment_load_address_unsupported"));
}

#[test]
fn package_admission_rejects_unsupported_descriptor_mmu_page_size() {
    // Arrange
    let mut ota = ota_fixture();
    ota[32 + 180] = 15;
    reseal_image(&mut ota);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("MMU page size");

    // Assert
    assert!(error
        .to_string()
        .contains("app_descriptor_mmu_page_size_mismatch"));
}

#[test]
fn package_admission_rejects_foreign_chip_id() {
    // Arrange
    let mut ota = ota_fixture();
    ota[12..14].copy_from_slice(&0_u16.to_le_bytes());
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("foreign chip");

    // Assert
    assert!(error.to_string().contains("ota_chip_id_mismatch"));
}

#[test]
fn package_admission_rejects_unsupported_header_policy() {
    // Arrange
    let mut ota = ota_fixture();
    ota[19] = 1;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("reserved header byte");

    // Assert
    assert!(error.to_string().contains("ota_header_policy_unsupported"));
}

#[test]
fn package_admission_rejects_segment_checksum_mismatch() {
    // Arrange
    let mut ota = ota_fixture();
    let payload_range = first_payload_range(&ota);
    ota[payload_range.end - 1] ^= 0x01;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("checksum mismatch");

    // Assert
    assert!(error.to_string().contains("ota_segment_checksum_mismatch"));
}

#[test]
fn package_admission_rejects_nonzero_alignment_padding() {
    // Arrange
    let mut ota = ota_fixture();
    let checksum_index = ota.len() - 32 - 1;
    ota[checksum_index - 1] = 1;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("nonzero padding");

    // Assert
    assert!(error.to_string().contains("ota_alignment_padding_invalid"));
}

#[test]
fn package_admission_rejects_missing_hash_declaration() {
    // Arrange
    let mut ota = ota_fixture();
    ota[23] = 0;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("hash declaration");

    // Assert
    assert!(error.to_string().contains("ota_header_policy_unsupported"));
}

#[test]
fn package_admission_rejects_appended_digest_mismatch() {
    // Arrange
    let mut ota = ota_fixture();
    let digest_index = ota.len() - 1;
    ota[digest_index] ^= 0x01;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("digest mismatch");

    // Assert
    assert!(error.to_string().contains("ota_appended_sha256_mismatch"));
}

#[test]
fn package_admission_rejects_truncated_appended_digest() {
    // Arrange
    let mut ota = ota_fixture();
    ota.pop();
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("truncated digest");

    // Assert
    assert!(error.to_string().contains("ota_appended_sha256_truncated"));
}

#[test]
fn package_admission_rejects_trailing_data() {
    // Arrange
    let mut ota = ota_fixture();
    ota.push(0);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("trailing data");

    // Assert
    assert!(error.to_string().contains("ota_trailing_data"));
}

#[test]
fn package_admission_rejects_truncated_ota_header() {
    // Arrange
    let ota = vec![ESP_IMAGE_MAGIC; ESP_IMAGE_HEADER_LEN - 1];
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("truncated header");

    // Assert
    assert!(error.to_string().contains("ota_image_header_truncated"));
}

#[test]
fn package_admission_rejects_overrunning_segment() {
    // Arrange
    let mut ota = ota_fixture();
    ota[28..32].copy_from_slice(&0x00ff_fffc_u32.to_le_bytes());
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("overrunning segment");

    // Assert
    assert!(error.to_string().contains("ota_segment_truncated"));
}

#[test]
fn package_admission_rejects_truncated_segment_header() {
    // Arrange
    let mut ota = ota_fixture();
    ota.truncate(first_payload_range(&ota).end + 4);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("segment header");

    // Assert
    assert!(error.to_string().contains("ota_segment_header_truncated"));
}

#[test]
fn package_admission_rejects_truncated_descriptor() {
    // Arrange
    let mut ota = ota_fixture();
    ota[1] = 1;
    ota[24..28].copy_from_slice(&0x4037_4000_u32.to_le_bytes());
    ota.truncate(ESP_IMAGE_HEADER_LEN + ESP_SEGMENT_HEADER_LEN + 64);
    ota[28..32].copy_from_slice(&64_u32.to_le_bytes());
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("descriptor");

    // Assert
    assert!(error.to_string().contains("app_descriptor_truncated"));
}

#[test]
fn package_admission_rejects_invalid_descriptor_magic() {
    // Arrange
    let mut ota = ota_fixture();
    ota[32..36].fill(0);
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("descriptor magic");

    // Assert
    assert!(error.to_string().contains("app_descriptor_magic_invalid"));
}

#[test]
fn package_admission_rejects_descriptor_version_mismatch() {
    // Arrange
    let mut ota = ota_fixture();
    ota[32 + APP_VERSION_OFFSET] = b'f';
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("version mismatch");

    // Assert
    assert!(error
        .to_string()
        .contains("app_descriptor_version_mismatch"));
}

#[test]
fn package_admission_rejects_descriptor_sha_mismatch() {
    // Arrange
    let mut ota = ota_fixture();
    ota[32 + APP_ELF_SHA_OFFSET] ^= 0x01;
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("SHA mismatch");

    // Assert
    assert!(error.to_string().contains("app_descriptor_sha_mismatch"));
}

#[test]
fn package_admission_ignores_source_commit_outside_validated_segments() {
    // Arrange
    let mut ota = ota_fixture();
    let commit_start = ESP_IMAGE_HEADER_LEN + ESP_SEGMENT_HEADER_LEN + ESP_APP_DESCRIPTOR_LEN;
    ota[commit_start..commit_start + SOURCE_COMMIT.len()].fill(b'x');
    reseal_image(&mut ota);
    ota.extend_from_slice(SOURCE_COMMIT.as_bytes());
    let factory = factory_fixture(&factory_table(), &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("out-of-segment commit");

    // Assert
    assert!(error
        .to_string()
        .contains("embedded_source_commit_mismatch"));
}

#[test]
fn package_admission_rejects_malformed_partition_table() {
    // Arrange
    let ota = ota_fixture();
    let factory = factory_fixture(&[0_u8; 64], &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("malformed table");

    // Assert
    assert!(error
        .to_string()
        .contains("factory_partition_table_invalid"));
}

#[test]
fn package_admission_rejects_missing_factory_partition() {
    // Arrange
    let ota = ota_fixture();
    let table = partition_table(vec![Partition::new(
        "ota_0",
        Type::App,
        SubType::App(AppType::Ota_0),
        FACTORY_PARTITION_OFFSET,
        FACTORY_PARTITION_SIZE,
        Flags::empty(),
    )]);
    let factory = factory_fixture(&table, &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("missing factory");

    // Assert
    assert!(error.to_string().contains("factory_partition_missing"));
}

#[test]
fn package_admission_rejects_duplicate_factory_partitions() {
    // Arrange
    let ota = ota_fixture();
    let table = partition_table(vec![
        factory_partition("factory", FACTORY_PARTITION_OFFSET),
        factory_partition("factory2", 0x420000),
    ]);
    let factory = factory_fixture(&table, &ota);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("duplicate factory");

    // Assert
    assert!(
        error.to_string().contains("factory_partition_duplicate"),
        "{error:#}"
    );
}

#[test]
fn package_admission_rejects_undersized_factory_image() {
    // Arrange
    let ota = ota_fixture();
    let mut factory = factory_fixture(&factory_table(), &ota);
    factory.truncate(FACTORY_PARTITION_OFFSET as usize + ota.len() - 1);

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("undersized factory");

    // Assert
    assert!(error.to_string().contains("factory_image_undersized"));
}

#[test]
fn package_admission_rejects_factory_ota_mismatch() {
    // Arrange
    let ota = ota_fixture();
    let mut factory = factory_fixture(&factory_table(), &ota);
    factory[FACTORY_PARTITION_OFFSET as usize + 40] ^= 0x01;

    // Act
    let error = validate_fixture(&factory, &ota).expect_err("factory mismatch");

    // Assert
    assert!(error.to_string().contains("ota_segment_checksum_mismatch"));
}
