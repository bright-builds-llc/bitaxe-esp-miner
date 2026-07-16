use std::ops::Range;

use anyhow::{bail, Context, Result};
use esp_idf_part::{AppType, Error as PartitionError, Partition, PartitionTable, SubType, Type};
use sha2::{Digest, Sha256};

const ESP_IMAGE_MAGIC: u8 = 0xe9;
const ESP_IMAGE_HEADER_LEN: usize = 24;
const ESP_IMAGE_MAX_SEGMENTS: usize = 16;
const ESP_SEGMENT_HEADER_LEN: usize = 8;
const ESP32_S3_CHIP_ID: u16 = 9;
const SUPPORTED_MIN_CHIP_REV: u8 = 0;
const SUPPORTED_MIN_CHIP_REV_FULL: u16 = 0;
const SUPPORTED_MAX_CHIP_REV_FULL: u16 = 99;
const ESP_IMAGE_CHECKSUM_SEED: u8 = 0xef;
const ESP_IMAGE_ALIGNMENT: usize = 16;
const ESP_IMAGE_DIGEST_LEN: usize = 32;
const ESP_APP_DESCRIPTOR_MAGIC: u32 = 0xABCD_5432;
const ESP_APP_DESCRIPTOR_LEN: usize = 256;
const APP_VERSION_OFFSET: usize = 16;
const APP_VERSION_LEN: usize = 32;
const APP_ELF_SHA_OFFSET: usize = 144;
const APP_ELF_SHA_LEN: usize = 32;
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
    validate_ota_identity(ota_bytes, &expected)?;

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
    if factory_ota != ota_bytes {
        bail!("identity_admission=blocked reason=factory_ota_image_mismatch");
    }

    Ok(())
}

fn validate_ota_identity(
    ota_bytes: &[u8],
    expected: &ExpectedApplicationIdentity<'_>,
) -> Result<()> {
    let Some(header) = ota_bytes.get(..ESP_IMAGE_HEADER_LEN) else {
        bail!("identity_admission=blocked reason=ota_image_header_truncated");
    };
    if header[0] != ESP_IMAGE_MAGIC {
        bail!("identity_admission=blocked reason=ota_image_magic_invalid");
    }
    validate_esp32_s3_header(header)?;
    let segment_count = usize::from(header[1]);
    if segment_count == 0 || segment_count > ESP_IMAGE_MAX_SEGMENTS {
        bail!("identity_admission=blocked reason=ota_segment_count_invalid");
    }

    let mut payloads = Vec::with_capacity(segment_count);
    let mut cursor = ESP_IMAGE_HEADER_LEN;
    let mut checksum = ESP_IMAGE_CHECKSUM_SEED;
    for _ in 0..segment_count {
        let segment_header_end = cursor
            .checked_add(ESP_SEGMENT_HEADER_LEN)
            .context("identity_admission=blocked reason=ota_segment_range_overflow")?;
        let Some(segment_header) = ota_bytes.get(cursor..segment_header_end) else {
            bail!("identity_admission=blocked reason=ota_segment_header_truncated");
        };
        let data_len = usize::try_from(u32::from_le_bytes([
            segment_header[4],
            segment_header[5],
            segment_header[6],
            segment_header[7],
        ]))
        .context("identity_admission=blocked reason=ota_segment_range_overflow")?;
        let payload_end = segment_header_end
            .checked_add(data_len)
            .context("identity_admission=blocked reason=ota_segment_range_overflow")?;
        if payload_end > ota_bytes.len() {
            bail!("identity_admission=blocked reason=ota_segment_truncated");
        }
        for byte in &ota_bytes[segment_header_end..payload_end] {
            checksum ^= byte;
        }
        payloads.push(segment_header_end..payload_end);
        cursor = payload_end;
    }
    let descriptor_range = descriptor_range(&payloads)?;
    let descriptor = &ota_bytes[descriptor_range];
    let magic = u32::from_le_bytes([descriptor[0], descriptor[1], descriptor[2], descriptor[3]]);
    if magic != ESP_APP_DESCRIPTOR_MAGIC {
        bail!("identity_admission=blocked reason=app_descriptor_magic_invalid");
    }
    let version =
        parse_fixed_string(&descriptor[APP_VERSION_OFFSET..APP_VERSION_OFFSET + APP_VERSION_LEN])?;
    if version != expected.build_label {
        bail!("identity_admission=blocked reason=app_descriptor_version_mismatch");
    }
    if expected.app_elf_sha256.len() != APP_ELF_SHA_LEN {
        bail!("identity_admission=blocked reason=app_descriptor_sha_invalid");
    }
    if descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN]
        != *expected.app_elf_sha256
    {
        bail!("identity_admission=blocked reason=app_descriptor_sha_mismatch");
    }
    if !payloads
        .iter()
        .any(|range| contains_bytes(&ota_bytes[range.clone()], expected.source_commit.as_bytes()))
    {
        bail!("identity_admission=blocked reason=embedded_source_commit_mismatch");
    }
    validate_esp_image_trailer(ota_bytes, header[23], cursor, checksum)?;

    Ok(())
}

fn validate_esp32_s3_header(header: &[u8]) -> Result<()> {
    let chip_id = u16::from_le_bytes([header[12], header[13]]);
    if chip_id != ESP32_S3_CHIP_ID {
        bail!("identity_admission=blocked reason=ota_chip_id_mismatch");
    }

    let min_chip_rev = header[14];
    let min_chip_rev_full = u16::from_le_bytes([header[15], header[16]]);
    let max_chip_rev_full = u16::from_le_bytes([header[17], header[18]]);
    let reserved = &header[19..23];
    if min_chip_rev != SUPPORTED_MIN_CHIP_REV
        || min_chip_rev_full != SUPPORTED_MIN_CHIP_REV_FULL
        || max_chip_rev_full != SUPPORTED_MAX_CHIP_REV_FULL
        || reserved.iter().any(|byte| *byte != 0)
    {
        bail!("identity_admission=blocked reason=ota_header_policy_unsupported");
    }
    if header[23] > 1 {
        bail!("identity_admission=blocked reason=ota_hash_declaration_invalid");
    }

    Ok(())
}

fn validate_esp_image_trailer(
    ota_bytes: &[u8],
    hash_appended: u8,
    segment_end: usize,
    checksum: u8,
) -> Result<()> {
    let padding_len =
        (ESP_IMAGE_ALIGNMENT - 1 - (segment_end % ESP_IMAGE_ALIGNMENT)) % ESP_IMAGE_ALIGNMENT;
    let checksum_index = segment_end
        .checked_add(padding_len)
        .context("identity_admission=blocked reason=ota_trailer_range_overflow")?;
    let Some(padding) = ota_bytes.get(segment_end..checksum_index) else {
        bail!("identity_admission=blocked reason=ota_segment_checksum_truncated");
    };
    if padding.iter().any(|byte| *byte != 0) {
        bail!("identity_admission=blocked reason=ota_alignment_padding_invalid");
    }
    let Some(stored_checksum) = ota_bytes.get(checksum_index) else {
        bail!("identity_admission=blocked reason=ota_segment_checksum_truncated");
    };
    if *stored_checksum != checksum {
        bail!("identity_admission=blocked reason=ota_segment_checksum_mismatch");
    }
    let image_end = checksum_index
        .checked_add(1)
        .context("identity_admission=blocked reason=ota_trailer_range_overflow")?;

    if hash_appended == 0 {
        if ota_bytes.len() != image_end {
            bail!("identity_admission=blocked reason=ota_hash_declaration_mismatch");
        }
        return Ok(());
    }

    let digest_end = image_end
        .checked_add(ESP_IMAGE_DIGEST_LEN)
        .context("identity_admission=blocked reason=ota_trailer_range_overflow")?;
    let Some(appended_digest) = ota_bytes.get(image_end..digest_end) else {
        bail!("identity_admission=blocked reason=ota_appended_sha256_truncated");
    };
    if ota_bytes.len() > digest_end {
        bail!("identity_admission=blocked reason=ota_trailing_data");
    }
    let expected_digest = Sha256::digest(&ota_bytes[..image_end]);
    if appended_digest != expected_digest.as_slice() {
        bail!("identity_admission=blocked reason=ota_appended_sha256_mismatch");
    }

    Ok(())
}

fn descriptor_range(payloads: &[Range<usize>]) -> Result<Range<usize>> {
    let Some(first_payload) = payloads.first() else {
        bail!("identity_admission=blocked reason=app_descriptor_missing");
    };
    let descriptor_end = first_payload
        .start
        .checked_add(ESP_APP_DESCRIPTOR_LEN)
        .context("identity_admission=blocked reason=app_descriptor_range_overflow")?;
    if descriptor_end > first_payload.end {
        bail!("identity_admission=blocked reason=app_descriptor_truncated");
    }

    Ok(first_payload.start..descriptor_end)
}

fn parse_fixed_string(bytes: &[u8]) -> Result<&str> {
    let value_end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[value_end..].iter().any(|byte| *byte != 0) {
        bail!("identity_admission=blocked reason=app_descriptor_version_invalid");
    }
    let value = std::str::from_utf8(&bytes[..value_end]).map_err(|_| {
        anyhow::anyhow!("identity_admission=blocked reason=app_descriptor_version_invalid")
    })?;
    if value.is_empty() || !value.is_ascii() {
        bail!("identity_admission=blocked reason=app_descriptor_version_invalid");
    }

    Ok(value)
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

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use esp_idf_part::Flags;
    use sha2::{Digest, Sha256};

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
        assert!(error.to_string().contains("ota_hash_declaration_mismatch"));
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
        ota[28..32].copy_from_slice(&u32::MAX.to_le_bytes());
        let factory = factory_fixture(&factory_table(), &ota);

        // Act
        let error = validate_fixture(&factory, &ota).expect_err("overrunning segment");

        // Assert
        assert!(error.to_string().contains("ota_segment_truncated"));
    }

    #[test]
    fn package_admission_rejects_truncated_segment_header() {
        // Arrange
        let mut ota = vec![0_u8; ESP_IMAGE_HEADER_LEN];
        ota[0] = ESP_IMAGE_MAGIC;
        ota[1] = 1;
        ota[12..14].copy_from_slice(&ESP32_S3_CHIP_ID.to_le_bytes());
        ota[17..19].copy_from_slice(&SUPPORTED_MAX_CHIP_REV_FULL.to_le_bytes());
        ota[23] = 1;
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
        ota[commit_start..].fill(b'x');
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
        assert!(error.to_string().contains("factory_ota_image_mismatch"));
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

    fn ota_fixture() -> Vec<u8> {
        let mut descriptor = vec![0_u8; ESP_APP_DESCRIPTOR_LEN];
        descriptor[..4].copy_from_slice(&ESP_APP_DESCRIPTOR_MAGIC.to_le_bytes());
        descriptor[APP_VERSION_OFFSET..APP_VERSION_OFFSET + BUILD_LABEL.len()]
            .copy_from_slice(BUILD_LABEL.as_bytes());
        descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN]
            .copy_from_slice(&APP_SHA);
        let mut payload = descriptor;
        payload.extend_from_slice(SOURCE_COMMIT.as_bytes());
        payload.push(0x5a);

        let mut image = vec![0_u8; ESP_IMAGE_HEADER_LEN];
        image[0] = ESP_IMAGE_MAGIC;
        image[1] = 1;
        image[12..14].copy_from_slice(&ESP32_S3_CHIP_ID.to_le_bytes());
        image[15..17].copy_from_slice(&0_u16.to_le_bytes());
        image[17..19].copy_from_slice(&SUPPORTED_MAX_CHIP_REV_FULL.to_le_bytes());
        image[23] = 1;
        image.extend_from_slice(&0x3c00_0020_u32.to_le_bytes());
        image.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        image.extend_from_slice(&payload);
        let checksum = payload
            .iter()
            .fold(ESP_IMAGE_CHECKSUM_SEED, |checksum, byte| checksum ^ byte);
        let padding_len = (15 - (image.len() % 16)) % 16;
        image.resize(image.len() + padding_len, 0);
        image.push(checksum);
        let digest = Sha256::digest(&image);
        image.extend_from_slice(&digest);
        image
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
