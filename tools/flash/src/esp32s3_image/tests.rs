use super::*;

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const BUILD_LABEL: &str = "0123456789ab-dev";
const APP_SHA: [u8; APP_ELF_SHA_LEN] = [0x5a; APP_ELF_SHA_LEN];

#[test]
fn esp32s3_image_accepts_supported_memory_families() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 4]),
        (0x4037_0000, vec![0; 4]),
        (0x4200_0168, vec![0; 4]),
        (0x5000_0000, vec![0; 4]),
        (0x600f_e000, vec![0; 4]),
    ]);

    // Act
    let result = validate_fixture(&image);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn esp32s3_image_accepts_zero_length_segments() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0, Vec::new()),
        (4, Vec::new()),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let result = validate_fixture(&image);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn esp32s3_image_rejects_empty_descriptor_segment() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, Vec::new()),
        (0x3c00_0028, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("empty descriptor segment");

    // Assert
    assert_eq!(error, ImageValidationError::AppDescriptorSegmentEmpty);
    assert_eq!(error.reason(), "app_descriptor_segment_empty");
}

#[test]
fn esp32s3_image_rejects_descriptor_segment_in_dram() {
    assert_descriptor_segment_family_rejected(0x3fc8_8000);
}

#[test]
fn esp32s3_image_rejects_descriptor_segment_in_iram() {
    assert_descriptor_segment_family_rejected(0x4037_0000);
}

#[test]
fn esp32s3_image_rejects_descriptor_segment_in_irom() {
    assert_descriptor_segment_family_rejected(0x4200_0020);
}

#[test]
fn esp32s3_image_rejects_descriptor_segment_in_rtc_data() {
    assert_descriptor_segment_family_rejected(0x5000_0000);
}

#[test]
fn esp32s3_image_rejects_descriptor_segment_in_rtc_fast() {
    assert_descriptor_segment_family_rejected(0x600f_e000);
}

#[test]
fn esp32s3_image_rejects_descriptor_shifted_inside_drom_segment() {
    // Arrange
    let mut shifted_descriptor = vec![0_u8; 4];
    shifted_descriptor.extend_from_slice(&descriptor_payload());
    let image = image_fixture(&[(0x3c00_0020, shifted_descriptor), (0x4037_0000, vec![0; 4])]);

    // Act
    let error = validate_fixture(&image).expect_err("shifted descriptor");

    // Assert
    assert_eq!(error, ImageValidationError::DescriptorMagicInvalid);
}

#[test]
fn esp32s3_image_rejects_partial_destination_overlap() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 16]),
        (0x3fc8_8008, vec![0; 16]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("partial destination overlap");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentDestinationOverlap);
    assert_eq!(error.reason(), "ota_segment_destination_overlap");
}

#[test]
fn esp32s3_image_rejects_contained_destination_overlap() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 32]),
        (0x3fc8_8008, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("contained destination overlap");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentDestinationOverlap);
    assert_eq!(error.reason(), "ota_segment_destination_overlap");
}

#[test]
fn esp32s3_image_rejects_identical_destination_ranges() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 8]),
        (0x3fc8_8000, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("identical destination ranges");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentDestinationOverlap);
    assert_eq!(error.reason(), "ota_segment_destination_overlap");
}

#[test]
fn esp32s3_image_rejects_dram_iram_alias_overlap() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 16]),
        (0x4037_8008, vec![0; 16]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("D/IRAM alias overlap");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentAliasOverlap);
    assert_eq!(error.reason(), "ota_segment_alias_overlap");
}

#[test]
fn esp32s3_image_accepts_exact_destination_adjacency() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 8]),
        (0x3fc8_8008, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let result = validate_fixture(&image);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn esp32s3_image_accepts_exact_dram_iram_alias_adjacency() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 8]),
        (0x4037_8008, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let result = validate_fixture(&image);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn esp32s3_image_accepts_zero_length_segment_inside_destination_range() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x3fc8_8000, vec![0; 16]),
        (0x3fc8_8004, Vec::new()),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let result = validate_fixture(&image);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn esp32s3_image_rejects_each_noncanonical_header_policy_field() {
    let mutations = [
        (2, 0),
        (3, 0),
        (8, 0),
        (9, 1),
        (10, 1),
        (11, 1),
        (14, 1),
        (15, 1),
        (16, 1),
        (17, 98),
        (18, 1),
        (19, 1),
        (20, 1),
        (21, 1),
        (22, 1),
        (23, 0),
    ];
    for (offset, value) in mutations {
        // Arrange
        let mut image = image_fixture(&[
            (0x3c00_0020, descriptor_payload()),
            (0x4037_0000, vec![0; 4]),
        ]);
        image[offset] = value;
        reseal(&mut image);

        // Act
        let error = validate_fixture(&image).expect_err("header policy mutation");

        // Assert
        assert_eq!(error, ImageValidationError::HeaderPolicyUnsupported);
    }
}

#[test]
fn esp32s3_image_rejects_representative_excluded_addresses() {
    for address in [
        0x403c_b700,
        0x3fcd_b700,
        0x3fce_2700,
        0x403d_2700,
        0x3fce_7710,
        0x403d_7710,
    ] {
        // Arrange
        let image = image_fixture(&[
            (0x3c00_0020, descriptor_payload()),
            (address, vec![0; 4]),
            (0x4037_0000, vec![0; 4]),
        ]);

        // Act
        let error = validate_fixture(&image).expect_err("excluded address");

        // Assert
        assert_eq!(error, ImageValidationError::SegmentLoadAddressUnsupported);
    }
}

#[test]
fn esp32s3_image_rejects_mapped_offset_mismatch() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0024, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("mapped mismatch");

    // Assert
    assert_eq!(error, ImageValidationError::MappedSegmentMisaligned);
}

#[test]
fn esp32s3_image_rejects_zero_length_mapped_offset_mismatch() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x4200_0004, Vec::new()),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("zero-length mapped mismatch");

    // Assert
    assert_eq!(error, ImageValidationError::MappedSegmentMisaligned);
}

#[test]
fn esp32s3_image_rejects_entry_outside_executable_segment() {
    // Arrange
    let mut image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);
    image[4..8].copy_from_slice(&0x3fc8_8000_u32.to_le_bytes());
    reseal(&mut image);

    // Act
    let error = validate_fixture(&image).expect_err("non-executable entry");

    // Assert
    assert_eq!(error, ImageValidationError::EntryAddressUnsupported);
}

#[test]
fn esp32s3_image_rejects_segment_length_boundary() {
    // Arrange
    let mut image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);
    image[28..32].copy_from_slice(&MAX_SEGMENT_LEN.to_le_bytes());

    // Act
    let error = validate_fixture(&image).expect_err("16 MiB segment");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentLengthInvalid);
}

#[test]
fn esp32s3_image_rejects_unaligned_segment_length() {
    // Arrange
    let mut image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);
    image[28..32].copy_from_slice(&2_u32.to_le_bytes());

    // Act
    let error = validate_fixture(&image).expect_err("unaligned segment");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentLengthInvalid);
}

#[test]
fn esp32s3_image_rejects_load_address_overflow() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (u32::MAX - 3, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("address overflow");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentRangeOverflow);
}

#[test]
fn esp32s3_image_rejects_segment_crossing_family_boundary() {
    // Arrange
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (IRAM.end - 4, vec![0; 8]),
        (0x4037_0000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("crossing segment");

    // Assert
    assert_eq!(error, ImageValidationError::SegmentLoadAddressUnsupported);
}

#[test]
fn esp32s3_image_rejects_truncation_at_structural_boundaries() {
    let image = image_fixture(&[
        (0x3c00_0020, descriptor_payload()),
        (0x4037_0000, vec![0; 4]),
    ]);
    for truncated_len in [
        0,
        IMAGE_HEADER_LEN - 1,
        IMAGE_HEADER_LEN,
        40,
        image.len() - 1,
    ] {
        // Arrange
        let truncated = &image[..truncated_len];

        // Act
        let result = validate_fixture(truncated);

        // Assert
        assert!(result.is_err(), "accepted truncation at {truncated_len}");
    }
}

fn validate_fixture(image: &[u8]) -> Result<ValidatedEsp32S3Image, ImageValidationError> {
    validate(
        image,
        ExpectedApplication {
            build_label: BUILD_LABEL,
            source_commit: SOURCE_COMMIT,
            app_elf_sha256: &APP_SHA,
        },
    )
}

fn assert_descriptor_segment_family_rejected(load_address: u32) {
    // Arrange
    let image = image_fixture(&[
        (load_address, descriptor_payload()),
        (0x4037_4000, vec![0; 4]),
    ]);

    // Act
    let error = validate_fixture(&image).expect_err("descriptor outside DROM");

    // Assert
    assert_eq!(error, ImageValidationError::AppDescriptorSegmentNotDrom);
    assert_eq!(error.reason(), "app_descriptor_segment_not_drom");
}

fn descriptor_payload() -> Vec<u8> {
    let mut descriptor = vec![0_u8; APP_DESCRIPTOR_LEN];
    descriptor[..4].copy_from_slice(&ESP_APP_DESCRIPTOR_MAGIC.to_le_bytes());
    descriptor[APP_VERSION_OFFSET..APP_VERSION_OFFSET + BUILD_LABEL.len()]
        .copy_from_slice(BUILD_LABEL.as_bytes());
    descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN].copy_from_slice(&APP_SHA);
    descriptor[APP_MMU_PAGE_SIZE_OFFSET] = APP_MMU_PAGE_SIZE_LOG2;
    descriptor.extend_from_slice(SOURCE_COMMIT.as_bytes());
    descriptor
}

fn image_fixture(segments: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut image = vec![0_u8; IMAGE_HEADER_LEN];
    image[0] = IMAGE_MAGIC;
    image[1] = u8::try_from(segments.len()).expect("fixture segment count");
    image[2] = SPI_MODE_DIO;
    image[3] = SPI_SPEED_80MHZ_SIZE_16MB;
    image[4..8].copy_from_slice(&0x4037_0000_u32.to_le_bytes());
    image[8] = SPI_WP_PIN_DEFAULT;
    image[12..14].copy_from_slice(&ESP32_S3_CHIP_ID.to_le_bytes());
    image[17..19].copy_from_slice(&MAX_CHIP_REV_FULL.to_le_bytes());
    image[23] = 1;
    for (load_address, payload) in segments {
        assert_eq!(payload.len() % 4, 0, "fixture segment alignment");
        image.extend_from_slice(&load_address.to_le_bytes());
        image.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("fixture payload length")
                .to_le_bytes(),
        );
        image.extend_from_slice(payload);
    }
    reseal(&mut image);
    image
}

fn reseal(image: &mut Vec<u8>) {
    let mut cursor = IMAGE_HEADER_LEN;
    let mut checksum = CHECKSUM_SEED;
    for _ in 0..usize::from(image[1]) {
        let payload_start = cursor + SEGMENT_HEADER_LEN;
        let payload_len =
            usize::try_from(read_u32(image, cursor + 4).expect("fixture segment length"))
                .expect("fixture payload length");
        let payload_end = payload_start + payload_len;
        checksum = image[payload_start..payload_end]
            .iter()
            .fold(checksum, |value, byte| value ^ byte);
        cursor = payload_end;
    }
    let padding_len = (IMAGE_ALIGNMENT - 1 - (cursor % IMAGE_ALIGNMENT)) % IMAGE_ALIGNMENT;
    image.truncate(cursor);
    image.resize(cursor + padding_len, 0);
    image.push(checksum);
    let digest = Sha256::digest(&*image);
    image.extend_from_slice(&digest);
}
