use std::ops::Range;

use anyhow::{bail, Context, Result};
use esp_idf_part::{AppType, Error as PartitionError, Partition, PartitionTable, SubType, Type};

const ESP_IMAGE_MAGIC: u8 = 0xe9;
const ESP_IMAGE_HEADER_LEN: usize = 24;
const ESP_IMAGE_MAX_SEGMENTS: usize = 16;
const ESP_SEGMENT_HEADER_LEN: usize = 8;
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
    let segment_payloads = validate_ota_identity(ota_bytes, &expected)?;
    if !segment_payloads
        .iter()
        .any(|range| contains_bytes(&ota_bytes[range.clone()], expected.source_commit.as_bytes()))
    {
        bail!("identity_admission=blocked reason=embedded_source_commit_mismatch");
    }

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
) -> Result<Vec<Range<usize>>> {
    let Some(header) = ota_bytes.get(..ESP_IMAGE_HEADER_LEN) else {
        bail!("identity_admission=blocked reason=ota_image_header_truncated");
    };
    if header[0] != ESP_IMAGE_MAGIC {
        bail!("identity_admission=blocked reason=ota_image_magic_invalid");
    }
    let segment_count = usize::from(header[1]);
    if segment_count == 0 || segment_count > ESP_IMAGE_MAX_SEGMENTS {
        bail!("identity_admission=blocked reason=ota_segment_count_invalid");
    }

    let mut payloads = Vec::with_capacity(segment_count);
    let mut cursor = ESP_IMAGE_HEADER_LEN;
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

    Ok(payloads)
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
        let commit_start = ota.len() - SOURCE_COMMIT.len();
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

        let mut image = vec![0_u8; ESP_IMAGE_HEADER_LEN];
        image[0] = ESP_IMAGE_MAGIC;
        image[1] = 1;
        image.extend_from_slice(&0x3c00_0020_u32.to_le_bytes());
        image.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        image.extend_from_slice(&payload);
        image
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
