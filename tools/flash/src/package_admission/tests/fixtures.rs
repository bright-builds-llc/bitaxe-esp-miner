use super::*;

pub(super) fn validate_fixture(factory: &[u8], ota: &[u8]) -> Result<()> {
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
pub(super) enum LayoutFixtureKind {
    DescriptorNotDrom,
    DestinationAdjacent,
    DestinationOverlap,
    AliasAdjacent,
    AliasOverlap,
    ZeroLengthInsideRange,
}

pub(super) fn assert_layout_rejected_at_ota_and_factory(
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

pub(super) fn layout_fixture(fixture_kind: LayoutFixtureKind) -> Vec<u8> {
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

pub(super) fn append_segment(image: &mut Vec<u8>, load_address: u32, payload: &[u8]) {
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

pub(super) fn segment_data_end(image: &[u8]) -> usize {
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

pub(super) fn second_segment_header(image: &[u8]) -> usize {
    let first_payload = first_payload_range(image);
    first_payload.end
}

pub(super) fn ota_fixture() -> Vec<u8> {
    let mut descriptor = vec![0_u8; ESP_APP_DESCRIPTOR_LEN];
    descriptor[..4].copy_from_slice(&ESP_APP_DESCRIPTOR_MAGIC.to_le_bytes());
    descriptor[APP_VERSION_OFFSET..APP_VERSION_OFFSET + BUILD_LABEL.len()]
        .copy_from_slice(BUILD_LABEL.as_bytes());
    descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN].copy_from_slice(&APP_SHA);
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

pub(super) fn reseal_image(image: &mut Vec<u8>) {
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

pub(super) fn first_payload_range(image: &[u8]) -> Range<usize> {
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

pub(super) fn factory_table() -> Vec<u8> {
    partition_table(vec![factory_partition("factory", FACTORY_PARTITION_OFFSET)])
}

pub(super) fn factory_partition(name: &str, offset: u32) -> Partition {
    Partition::new(
        name,
        Type::App,
        SubType::App(AppType::Factory),
        offset,
        FACTORY_PARTITION_SIZE,
        Flags::empty(),
    )
}

pub(super) fn partition_table(partitions: Vec<Partition>) -> Vec<u8> {
    PartitionTable::new(partitions)
        .to_bin()
        .expect("partition table")
}

pub(super) fn factory_fixture(table: &[u8], ota: &[u8]) -> Vec<u8> {
    let factory_offset = FACTORY_PARTITION_OFFSET as usize;
    let mut factory = vec![0xff; factory_offset + ota.len()];
    factory[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + table.len()].copy_from_slice(table);
    factory[factory_offset..factory_offset + ota.len()].copy_from_slice(ota);
    factory
}
