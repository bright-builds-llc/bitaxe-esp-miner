use anyhow::{bail, Context, Result};
use esp_idf_part::{AppType, Error as PartitionError, Partition, PartitionTable, SubType, Type};

use crate::esp32s3_image::{self, ExpectedApplication};

const PARTITION_TABLE_OFFSET: usize = 0x8000;
const PARTITION_TABLE_LEN: usize = 0x1000;
const FACTORY_PARTITION_OFFSET: u32 = 0x10000;
const FACTORY_PARTITION_SIZE: u32 = 0x400000;

pub(crate) struct ExpectedApplicationIdentity<'a> {
    pub(crate) build_label: &'a str,
    pub(crate) source_commit: &'a str,
    pub(crate) app_elf_sha256: &'a [u8],
}

pub(crate) fn validate_factory_ota_identity(
    factory_bytes: &[u8],
    ota_bytes: &[u8],
    expected: ExpectedApplicationIdentity<'_>,
) -> Result<()> {
    let ota_validation = esp32s3_image::validate(
        ota_bytes,
        ExpectedApplication {
            build_label: expected.build_label,
            source_commit: expected.source_commit,
            app_elf_sha256: expected.app_elf_sha256,
        },
    )?;

    let factory_partition = parse_factory_partition(factory_bytes)?;
    validate_factory_layout(&factory_partition)?;
    let factory_offset = usize::try_from(factory_partition.offset())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    let partition_size = usize::try_from(factory_partition.size())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    if ota_bytes.len() > partition_size {
        bail!("identity_admission=blocked reason=factory_partition_ota_oversized");
    }
    let factory_ota_end = factory_offset
        .checked_add(ota_bytes.len())
        .context("identity_admission=blocked reason=factory_partition_range_overflow")?;
    let Some(factory_ota) = factory_bytes.get(factory_offset..factory_ota_end) else {
        bail!("identity_admission=blocked reason=factory_image_undersized");
    };
    let factory_validation = esp32s3_image::validate(
        factory_ota,
        ExpectedApplication {
            build_label: expected.build_label,
            source_commit: expected.source_commit,
            app_elf_sha256: expected.app_elf_sha256,
        },
    )?;
    if factory_ota != ota_bytes || factory_validation != ota_validation {
        bail!("identity_admission=blocked reason=factory_ota_image_mismatch");
    }

    Ok(())
}

fn parse_factory_partition(factory_bytes: &[u8]) -> Result<Partition> {
    let table_end = PARTITION_TABLE_OFFSET
        .checked_add(PARTITION_TABLE_LEN)
        .context("identity_admission=blocked reason=factory_partition_table_range_overflow")?;
    let Some(table_bytes) = factory_bytes.get(PARTITION_TABLE_OFFSET..table_end) else {
        bail!("identity_admission=blocked reason=factory_partition_table_truncated");
    };
    let table = match PartitionTable::try_from_bytes(table_bytes) {
        Ok(table) => table,
        Err(PartitionError::MultipleFactoryPartitions) => {
            bail!("identity_admission=blocked reason=factory_partition_duplicate");
        }
        Err(_) => bail!("identity_admission=blocked reason=factory_partition_table_invalid"),
    };
    let matches = table
        .partitions()
        .iter()
        .filter(|partition| {
            partition.ty() == Type::App && partition.subtype() == SubType::App(AppType::Factory)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => bail!("identity_admission=blocked reason=factory_partition_missing"),
        [partition] => Ok((*partition).clone()),
        _ => bail!("identity_admission=blocked reason=factory_partition_duplicate"),
    }
}

fn validate_factory_layout(partition: &Partition) -> Result<()> {
    if partition.offset() != FACTORY_PARTITION_OFFSET || partition.size() != FACTORY_PARTITION_SIZE
    {
        bail!("identity_admission=blocked reason=factory_partition_layout_mismatch");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use esp_idf_part::Flags;
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

    fn validate_fixture(factory: &[u8], ota: &[u8]) -> Result<()> {
        validate_factory_ota_identity(
            factory,
            ota,
            ExpectedApplicationIdentity {
                build_label: BUILD_LABEL,
                source_commit: SOURCE_COMMIT,
                app_elf_sha256: &APP_SHA,
            },
        )
    }

    #[derive(Clone, Copy)]
    enum LayoutFixtureKind {
        DescriptorNotDrom,
        DestinationAdjacent,
        DestinationOverlap,
        AliasAdjacent,
        AliasOverlap,
        ZeroLengthInsideRange,
    }

    fn assert_layout_rejected_at_ota_and_factory(
        fixture_kind: LayoutFixtureKind,
        expected_reason: &str,
    ) {
        // Arrange
        let malformed = layout_fixture(fixture_kind);
        let valid = match fixture_kind {
            LayoutFixtureKind::DescriptorNotDrom => ota_fixture(),
            LayoutFixtureKind::DestinationOverlap => {
                layout_fixture(LayoutFixtureKind::DestinationAdjacent)
            }
            LayoutFixtureKind::AliasOverlap => layout_fixture(LayoutFixtureKind::AliasAdjacent),
            _ => panic!("fixture must represent a rejected layout"),
        };
        assert_eq!(malformed.len(), valid.len(), "paired fixture length");
        let malformed_factory = factory_fixture(&factory_table(), &malformed);

        // Act
        let ota_error = validate_fixture(&malformed_factory, &malformed)
            .expect_err("malformed OTA layout")
            .to_string();
        let factory_error = validate_fixture(&malformed_factory, &valid)
            .expect_err("malformed factory layout")
            .to_string();

        // Assert
        let expected = format!("identity_admission=blocked reason={expected_reason}");
        assert_eq!(ota_error, expected);
        assert_eq!(factory_error, expected);
    }

    fn layout_fixture(fixture_kind: LayoutFixtureKind) -> Vec<u8> {
        let mut image = ota_fixture();
        match fixture_kind {
            LayoutFixtureKind::DescriptorNotDrom => {
                image[24..28].copy_from_slice(&0x3fc8_8000_u32.to_le_bytes());
            }
            LayoutFixtureKind::DestinationAdjacent => {
                append_segment(&mut image, 0x4037_4004, &[0; 4]);
            }
            LayoutFixtureKind::DestinationOverlap => {
                append_segment(&mut image, 0x4037_4000, &[0; 4]);
            }
            LayoutFixtureKind::AliasAdjacent | LayoutFixtureKind::AliasOverlap => {
                image[4..8].copy_from_slice(&0x4037_8000_u32.to_le_bytes());
                let executable_header = second_segment_header(&image);
                image[executable_header..executable_header + 4]
                    .copy_from_slice(&0x4037_8000_u32.to_le_bytes());
                let dram_address = if matches!(fixture_kind, LayoutFixtureKind::AliasAdjacent) {
                    0x3fc8_8004
                } else {
                    0x3fc8_8000
                };
                append_segment(&mut image, dram_address, &[0; 4]);
            }
            LayoutFixtureKind::ZeroLengthInsideRange => {
                append_segment(&mut image, 0x4037_4000, &[]);
            }
        }
        reseal_image(&mut image);
        image
    }

    fn append_segment(image: &mut Vec<u8>, load_address: u32, payload: &[u8]) {
        let data_end = segment_data_end(image);
        image.truncate(data_end);
        image[1] = image[1].checked_add(1).expect("fixture segment count");
        image.extend_from_slice(&load_address.to_le_bytes());
        image.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("fixture payload length")
                .to_le_bytes(),
        );
        image.extend_from_slice(payload);
    }

    fn segment_data_end(image: &[u8]) -> usize {
        let mut cursor = ESP_IMAGE_HEADER_LEN;
        for _ in 0..usize::from(image[1]) {
            let payload_len = usize::try_from(u32::from_le_bytes(
                image[cursor + 4..cursor + 8]
                    .try_into()
                    .expect("fixture segment length"),
            ))
            .expect("fixture payload length");
            cursor += ESP_SEGMENT_HEADER_LEN + payload_len;
        }
        cursor
    }

    fn second_segment_header(image: &[u8]) -> usize {
        let first_payload = first_payload_range(image);
        first_payload.end
    }

    fn ota_fixture() -> Vec<u8> {
        let mut descriptor = vec![0_u8; ESP_APP_DESCRIPTOR_LEN];
        descriptor[..4].copy_from_slice(&ESP_APP_DESCRIPTOR_MAGIC.to_le_bytes());
        descriptor[APP_VERSION_OFFSET..APP_VERSION_OFFSET + BUILD_LABEL.len()]
            .copy_from_slice(BUILD_LABEL.as_bytes());
        descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN]
            .copy_from_slice(&APP_SHA);
        descriptor[180] = 16;
        let mut payload = descriptor;
        payload.extend_from_slice(SOURCE_COMMIT.as_bytes());
        payload.extend_from_slice(&[0x5a; 4]);

        let mut image = vec![0_u8; ESP_IMAGE_HEADER_LEN];
        image[0] = ESP_IMAGE_MAGIC;
        image[1] = 2;
        image[2] = 2;
        image[3] = 0x4f;
        image[4..8].copy_from_slice(&0x4037_4000_u32.to_le_bytes());
        image[8] = 0xee;
        image[12..14].copy_from_slice(&ESP32_S3_CHIP_ID.to_le_bytes());
        image[15..17].copy_from_slice(&0_u16.to_le_bytes());
        image[17..19].copy_from_slice(&SUPPORTED_MAX_CHIP_REV_FULL.to_le_bytes());
        image[23] = 1;
        image.extend_from_slice(&0x3c00_0020_u32.to_le_bytes());
        image.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        image.extend_from_slice(&payload);
        image.extend_from_slice(&0x4037_4000_u32.to_le_bytes());
        image.extend_from_slice(&4_u32.to_le_bytes());
        image.extend_from_slice(&[0x13, 0, 0, 0]);
        reseal_image(&mut image);
        image
    }

    fn reseal_image(image: &mut Vec<u8>) {
        let segment_count = usize::from(image[1]);
        let mut cursor = ESP_IMAGE_HEADER_LEN;
        let mut checksum = ESP_IMAGE_CHECKSUM_SEED;
        for _ in 0..segment_count {
            let payload_start = cursor + ESP_SEGMENT_HEADER_LEN;
            let payload_len = usize::try_from(u32::from_le_bytes([
                image[cursor + 4],
                image[cursor + 5],
                image[cursor + 6],
                image[cursor + 7],
            ]))
            .expect("fixture payload length");
            let payload_end = payload_start + payload_len;
            checksum = image[payload_start..payload_end]
                .iter()
                .fold(checksum, |value, byte| value ^ byte);
            cursor = payload_end;
        }
        let padding_len = (15 - (cursor % 16)) % 16;
        image.truncate(cursor);
        image.resize(cursor + padding_len, 0);
        image.push(checksum);
        let digest = Sha256::digest(&*image);
        image.extend_from_slice(&digest);
    }

    fn first_payload_range(image: &[u8]) -> Range<usize> {
        let start = ESP_IMAGE_HEADER_LEN + ESP_SEGMENT_HEADER_LEN;
        let payload_len = usize::try_from(u32::from_le_bytes([
            image[ESP_IMAGE_HEADER_LEN + 4],
            image[ESP_IMAGE_HEADER_LEN + 5],
            image[ESP_IMAGE_HEADER_LEN + 6],
            image[ESP_IMAGE_HEADER_LEN + 7],
        ]))
        .expect("fixture payload length");
        start..start + payload_len
    }

    fn factory_table() -> Vec<u8> {
        partition_table(vec![factory_partition("factory", FACTORY_PARTITION_OFFSET)])
    }

    fn factory_partition(name: &str, offset: u32) -> Partition {
        Partition::new(
            name,
            Type::App,
            SubType::App(AppType::Factory),
            offset,
            FACTORY_PARTITION_SIZE,
            Flags::empty(),
        )
    }

    fn partition_table(partitions: Vec<Partition>) -> Vec<u8> {
        PartitionTable::new(partitions)
            .to_bin()
            .expect("partition table")
    }

    fn factory_fixture(table: &[u8], ota: &[u8]) -> Vec<u8> {
        let factory_offset = FACTORY_PARTITION_OFFSET as usize;
        let mut factory = vec![0xff; factory_offset + ota.len()];
        factory[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + table.len()]
            .copy_from_slice(table);
        factory[factory_offset..factory_offset + ota.len()].copy_from_slice(ota);
        factory
    }
}
