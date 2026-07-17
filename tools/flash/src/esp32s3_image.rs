use std::{fmt, ops::Range};

use sha2::{Digest, Sha256};

const IMAGE_MAGIC: u8 = 0xe9;
const IMAGE_HEADER_LEN: usize = 24;
const MAX_SEGMENTS: usize = 16;
const SEGMENT_HEADER_LEN: usize = 8;
const SPI_MODE_DIO: u8 = 2;
const SPI_SPEED_80MHZ_SIZE_16MB: u8 = 0x4f;
const SPI_WP_PIN_DEFAULT: u8 = 0xee;
const ESP32_S3_CHIP_ID: u16 = 9;
const MAX_CHIP_REV_FULL: u16 = 99;
const CHECKSUM_SEED: u8 = 0xef;
const IMAGE_ALIGNMENT: usize = 16;
const DIGEST_LEN: usize = 32;
const MAX_SEGMENT_LEN: u32 = 16 * 1024 * 1024;
const MMU_PAGE_SIZE: u32 = 64 * 1024;
const SOC_I_D_OFFSET: u32 = 0x006f_0000;
const ESP_APP_DESCRIPTOR_MAGIC: u32 = 0xABCD_5432;
const APP_DESCRIPTOR_LEN: usize = 256;
const APP_VERSION_OFFSET: usize = 16;
const APP_VERSION_LEN: usize = 32;
const APP_ELF_SHA_OFFSET: usize = 144;
const APP_ELF_SHA_LEN: usize = 32;
const APP_MMU_PAGE_SIZE_OFFSET: usize = 180;
const APP_MMU_PAGE_SIZE_LOG2: u8 = 16;

const DROM: Range<u32> = 0x3c00_0000..0x3e00_0000;
const DRAM: Range<u32> = 0x3fc8_8000..0x3fcd_b700;
const IRAM: Range<u32> = 0x4037_0000..0x403c_b700;
const IROM: Range<u32> = 0x4200_0000..0x4400_0000;
const RTC_DATA: Range<u32> = 0x5000_0000..0x5000_2000;
const RTC_FAST: Range<u32> = 0x600f_e000..0x6010_0000;

pub(crate) struct ExpectedApplication<'a> {
    pub(crate) build_label: &'a str,
    pub(crate) source_commit: &'a str,
    pub(crate) app_elf_sha256: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ValidatedEsp32S3Image {
    _layout: ValidatedSegmentLayout,
    _entry_address: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageValidationError {
    HeaderTruncated,
    MagicInvalid,
    SegmentCountInvalid,
    HeaderPolicyUnsupported,
    ChipIdMismatch,
    SegmentHeaderTruncated,
    SegmentLengthInvalid,
    SegmentRangeOverflow,
    SegmentTruncated,
    SegmentLoadAddressUnsupported,
    MappedSegmentMisaligned,
    EntryAddressUnaligned,
    EntryAddressUnsupported,
    AppDescriptorSegmentEmpty,
    AppDescriptorSegmentNotDrom,
    SegmentDestinationOverlap,
    SegmentAliasOverlap,
    DescriptorMissing,
    DescriptorTruncated,
    DescriptorMagicInvalid,
    DescriptorVersionInvalid,
    DescriptorVersionMismatch,
    DescriptorShaInvalid,
    DescriptorShaMismatch,
    DescriptorMmuPageSizeMismatch,
    EmbeddedSourceCommitMismatch,
    TrailerRangeOverflow,
    SegmentChecksumTruncated,
    AlignmentPaddingInvalid,
    SegmentChecksumMismatch,
    AppendedSha256Truncated,
    AppendedSha256Mismatch,
    TrailingData,
}

impl ImageValidationError {
    fn reason(self) -> &'static str {
        match self {
            Self::HeaderTruncated => "ota_image_header_truncated",
            Self::MagicInvalid => "ota_image_magic_invalid",
            Self::SegmentCountInvalid => "ota_segment_count_invalid",
            Self::HeaderPolicyUnsupported => "ota_header_policy_unsupported",
            Self::ChipIdMismatch => "ota_chip_id_mismatch",
            Self::SegmentHeaderTruncated => "ota_segment_header_truncated",
            Self::SegmentLengthInvalid => "ota_segment_length_invalid",
            Self::SegmentRangeOverflow => "ota_segment_range_overflow",
            Self::SegmentTruncated => "ota_segment_truncated",
            Self::SegmentLoadAddressUnsupported => "ota_segment_load_address_unsupported",
            Self::MappedSegmentMisaligned => "ota_mapped_segment_misaligned",
            Self::EntryAddressUnaligned => "ota_entry_address_unaligned",
            Self::EntryAddressUnsupported => "ota_entry_address_unsupported",
            Self::AppDescriptorSegmentEmpty => "app_descriptor_segment_empty",
            Self::AppDescriptorSegmentNotDrom => "app_descriptor_segment_not_drom",
            Self::SegmentDestinationOverlap => "ota_segment_destination_overlap",
            Self::SegmentAliasOverlap => "ota_segment_alias_overlap",
            Self::DescriptorMissing => "app_descriptor_missing",
            Self::DescriptorTruncated => "app_descriptor_truncated",
            Self::DescriptorMagicInvalid => "app_descriptor_magic_invalid",
            Self::DescriptorVersionInvalid => "app_descriptor_version_invalid",
            Self::DescriptorVersionMismatch => "app_descriptor_version_mismatch",
            Self::DescriptorShaInvalid => "app_descriptor_sha_invalid",
            Self::DescriptorShaMismatch => "app_descriptor_sha_mismatch",
            Self::DescriptorMmuPageSizeMismatch => "app_descriptor_mmu_page_size_mismatch",
            Self::EmbeddedSourceCommitMismatch => "embedded_source_commit_mismatch",
            Self::TrailerRangeOverflow => "ota_trailer_range_overflow",
            Self::SegmentChecksumTruncated => "ota_segment_checksum_truncated",
            Self::AlignmentPaddingInvalid => "ota_alignment_padding_invalid",
            Self::SegmentChecksumMismatch => "ota_segment_checksum_mismatch",
            Self::AppendedSha256Truncated => "ota_appended_sha256_truncated",
            Self::AppendedSha256Mismatch => "ota_appended_sha256_mismatch",
            Self::TrailingData => "ota_trailing_data",
        }
    }
}

impl fmt::Display for ImageValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "identity_admission=blocked reason={}",
            self.reason()
        )
    }
}

impl std::error::Error for ImageValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryFamily {
    Drom,
    Dram,
    Iram,
    Irom,
    RtcData,
    RtcFast,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedSegment {
    load_address: u32,
    end_address: u32,
    payload: Range<usize>,
    maybe_memory_family: Option<MemoryFamily>,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedDestinationRange {
    range: Range<u32>,
    memory_family: MemoryFamily,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedSegmentLayout {
    segments: Vec<ValidatedSegment>,
    destination_ranges: Vec<ValidatedDestinationRange>,
}

pub(crate) fn validate(
    image: &[u8],
    expected: ExpectedApplication<'_>,
) -> Result<ValidatedEsp32S3Image, ImageValidationError> {
    let header = image
        .get(..IMAGE_HEADER_LEN)
        .ok_or(ImageValidationError::HeaderTruncated)?;
    validate_header(header)?;

    let segment_count = usize::from(header[1]);
    let entry_address = read_u32(header, 4).ok_or(ImageValidationError::HeaderTruncated)?;
    let mut segments = Vec::with_capacity(segment_count);
    let mut cursor = IMAGE_HEADER_LEN;
    let mut checksum = CHECKSUM_SEED;

    for _ in 0..segment_count {
        let header_end = cursor
            .checked_add(SEGMENT_HEADER_LEN)
            .ok_or(ImageValidationError::SegmentRangeOverflow)?;
        let segment_header = image
            .get(cursor..header_end)
            .ok_or(ImageValidationError::SegmentHeaderTruncated)?;
        let load_address =
            read_u32(segment_header, 0).ok_or(ImageValidationError::SegmentHeaderTruncated)?;
        let data_len =
            read_u32(segment_header, 4).ok_or(ImageValidationError::SegmentHeaderTruncated)?;
        if data_len % 4 != 0 || data_len >= MAX_SEGMENT_LEN {
            return Err(ImageValidationError::SegmentLengthInvalid);
        }
        let data_len_usize =
            usize::try_from(data_len).map_err(|_| ImageValidationError::SegmentRangeOverflow)?;
        let payload_end = header_end
            .checked_add(data_len_usize)
            .ok_or(ImageValidationError::SegmentRangeOverflow)?;
        let payload = image
            .get(header_end..payload_end)
            .ok_or(ImageValidationError::SegmentTruncated)?;
        let (end_address, maybe_memory_family) =
            validate_load_address(load_address, data_len, header_end)?;
        checksum = payload.iter().fold(checksum, |value, byte| value ^ byte);
        segments.push(ValidatedSegment {
            load_address,
            end_address,
            payload: header_end..payload_end,
            maybe_memory_family,
        });
        cursor = payload_end;
    }

    let layout = ValidatedSegmentLayout::try_new(segments)?;
    validate_entry_address(entry_address, &layout)?;
    validate_descriptor(image, &layout, &expected)?;
    validate_trailer(image, cursor, checksum)?;

    Ok(ValidatedEsp32S3Image {
        _layout: layout,
        _entry_address: entry_address,
    })
}

impl ValidatedSegmentLayout {
    fn try_new(segments: Vec<ValidatedSegment>) -> Result<Self, ImageValidationError> {
        validate_descriptor_segment(&segments)?;
        let destination_ranges = checked_destination_ranges(&segments);
        validate_destination_disjointness(&destination_ranges)?;
        validate_alias_disjointness(&destination_ranges)?;

        Ok(Self {
            segments,
            destination_ranges,
        })
    }
}

fn validate_descriptor_segment(segments: &[ValidatedSegment]) -> Result<(), ImageValidationError> {
    let descriptor_segment = segments
        .first()
        .ok_or(ImageValidationError::DescriptorMissing)?;
    if descriptor_segment.load_address == descriptor_segment.end_address {
        return Err(ImageValidationError::AppDescriptorSegmentEmpty);
    }
    if descriptor_segment.payload.len() < APP_DESCRIPTOR_LEN {
        return Err(ImageValidationError::DescriptorTruncated);
    }
    if descriptor_segment.maybe_memory_family != Some(MemoryFamily::Drom) {
        return Err(ImageValidationError::AppDescriptorSegmentNotDrom);
    }

    Ok(())
}

fn checked_destination_ranges(segments: &[ValidatedSegment]) -> Vec<ValidatedDestinationRange> {
    segments
        .iter()
        .filter_map(|segment| {
            let memory_family = segment.maybe_memory_family?;
            (segment.load_address < segment.end_address).then_some(ValidatedDestinationRange {
                range: segment.load_address..segment.end_address,
                memory_family,
            })
        })
        .collect()
}

fn validate_destination_disjointness(
    ranges: &[ValidatedDestinationRange],
) -> Result<(), ImageValidationError> {
    for (index, left) in ranges.iter().enumerate() {
        for right in &ranges[index + 1..] {
            if ranges_overlap(&left.range, &right.range) {
                return Err(ImageValidationError::SegmentDestinationOverlap);
            }
        }
    }

    Ok(())
}

fn validate_alias_disjointness(
    ranges: &[ValidatedDestinationRange],
) -> Result<(), ImageValidationError> {
    for dram in ranges
        .iter()
        .filter(|range| range.memory_family == MemoryFamily::Dram)
    {
        for iram in ranges
            .iter()
            .filter(|range| range.memory_family == MemoryFamily::Iram)
        {
            let normalized_iram = iram
                .range
                .start
                .checked_sub(SOC_I_D_OFFSET)
                .ok_or(ImageValidationError::SegmentRangeOverflow)?
                ..iram
                    .range
                    .end
                    .checked_sub(SOC_I_D_OFFSET)
                    .ok_or(ImageValidationError::SegmentRangeOverflow)?;
            if ranges_overlap(&dram.range, &normalized_iram) {
                return Err(ImageValidationError::SegmentAliasOverlap);
            }
        }
    }

    Ok(())
}

fn ranges_overlap(left: &Range<u32>, right: &Range<u32>) -> bool {
    left.start < right.end && right.start < left.end
}

fn validate_header(header: &[u8]) -> Result<(), ImageValidationError> {
    if header[0] != IMAGE_MAGIC {
        return Err(ImageValidationError::MagicInvalid);
    }
    let segment_count = usize::from(header[1]);
    if segment_count == 0 || segment_count > MAX_SEGMENTS {
        return Err(ImageValidationError::SegmentCountInvalid);
    }
    if header[2] != SPI_MODE_DIO
        || header[3] != SPI_SPEED_80MHZ_SIZE_16MB
        || header[8] != SPI_WP_PIN_DEFAULT
        || header[9..12].iter().any(|byte| *byte != 0)
        || header[14] != 0
        || read_u16(header, 15) != Some(0)
        || read_u16(header, 17) != Some(MAX_CHIP_REV_FULL)
        || header[19..23].iter().any(|byte| *byte != 0)
        || header[23] != 1
    {
        return Err(ImageValidationError::HeaderPolicyUnsupported);
    }
    if read_u16(header, 12) != Some(ESP32_S3_CHIP_ID) {
        return Err(ImageValidationError::ChipIdMismatch);
    }

    Ok(())
}

fn validate_load_address(
    load_address: u32,
    data_len: u32,
    payload_offset: usize,
) -> Result<(u32, Option<MemoryFamily>), ImageValidationError> {
    if data_len == 0 {
        let maybe_memory_family = mapped_memory_family(load_address);
        if maybe_memory_family.is_some() {
            validate_mapped_congruence(load_address, payload_offset)?;
        }
        return Ok((load_address, maybe_memory_family));
    }

    let end_address = load_address
        .checked_add(data_len)
        .ok_or(ImageValidationError::SegmentRangeOverflow)?;
    let memory_family = memory_family_for_range(load_address, end_address)
        .ok_or(ImageValidationError::SegmentLoadAddressUnsupported)?;
    if matches!(memory_family, MemoryFamily::Drom | MemoryFamily::Irom) {
        validate_mapped_congruence(load_address, payload_offset)?;
    }

    Ok((end_address, Some(memory_family)))
}

fn memory_family_for_range(start: u32, end: u32) -> Option<MemoryFamily> {
    for (range, family) in [
        (DROM, MemoryFamily::Drom),
        (DRAM, MemoryFamily::Dram),
        (IRAM, MemoryFamily::Iram),
        (IROM, MemoryFamily::Irom),
        (RTC_DATA, MemoryFamily::RtcData),
        (RTC_FAST, MemoryFamily::RtcFast),
    ] {
        if start >= range.start && end <= range.end {
            return Some(family);
        }
    }

    None
}

fn mapped_memory_family(load_address: u32) -> Option<MemoryFamily> {
    if DROM.contains(&load_address) {
        return Some(MemoryFamily::Drom);
    }
    if IROM.contains(&load_address) {
        return Some(MemoryFamily::Irom);
    }

    None
}

fn validate_mapped_congruence(
    load_address: u32,
    payload_offset: usize,
) -> Result<(), ImageValidationError> {
    let page_size = usize::try_from(MMU_PAGE_SIZE)
        .map_err(|_| ImageValidationError::MappedSegmentMisaligned)?;
    let address_offset = usize::try_from(load_address % MMU_PAGE_SIZE)
        .map_err(|_| ImageValidationError::MappedSegmentMisaligned)?;
    if payload_offset % page_size != address_offset {
        return Err(ImageValidationError::MappedSegmentMisaligned);
    }

    Ok(())
}

fn validate_entry_address(
    entry_address: u32,
    layout: &ValidatedSegmentLayout,
) -> Result<(), ImageValidationError> {
    if entry_address % 4 != 0 {
        return Err(ImageValidationError::EntryAddressUnaligned);
    }
    let contained = layout.segments.iter().any(|segment| {
        matches!(
            segment.maybe_memory_family,
            Some(MemoryFamily::Iram | MemoryFamily::Irom)
        ) && segment.load_address < segment.end_address
            && entry_address >= segment.load_address
            && entry_address < segment.end_address
    });
    if !contained {
        return Err(ImageValidationError::EntryAddressUnsupported);
    }

    Ok(())
}

fn validate_descriptor(
    image: &[u8],
    layout: &ValidatedSegmentLayout,
    expected: &ExpectedApplication<'_>,
) -> Result<(), ImageValidationError> {
    let first_payload = layout
        .segments
        .first()
        .ok_or(ImageValidationError::DescriptorMissing)?;
    let descriptor_end = first_payload
        .payload
        .start
        .checked_add(APP_DESCRIPTOR_LEN)
        .ok_or(ImageValidationError::DescriptorTruncated)?;
    if descriptor_end > first_payload.payload.end {
        return Err(ImageValidationError::DescriptorTruncated);
    }
    let descriptor = image
        .get(first_payload.payload.start..descriptor_end)
        .ok_or(ImageValidationError::DescriptorTruncated)?;
    if read_u32(descriptor, 0) != Some(ESP_APP_DESCRIPTOR_MAGIC) {
        return Err(ImageValidationError::DescriptorMagicInvalid);
    }
    let version_bytes = descriptor
        .get(APP_VERSION_OFFSET..APP_VERSION_OFFSET + APP_VERSION_LEN)
        .ok_or(ImageValidationError::DescriptorTruncated)?;
    let version = parse_fixed_string(version_bytes)?;
    if version != expected.build_label {
        return Err(ImageValidationError::DescriptorVersionMismatch);
    }
    if expected.app_elf_sha256.len() != APP_ELF_SHA_LEN {
        return Err(ImageValidationError::DescriptorShaInvalid);
    }
    let descriptor_sha = descriptor
        .get(APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN)
        .ok_or(ImageValidationError::DescriptorTruncated)?;
    if descriptor_sha != expected.app_elf_sha256 {
        return Err(ImageValidationError::DescriptorShaMismatch);
    }
    if descriptor.get(APP_MMU_PAGE_SIZE_OFFSET).copied() != Some(APP_MMU_PAGE_SIZE_LOG2) {
        return Err(ImageValidationError::DescriptorMmuPageSizeMismatch);
    }
    if !layout.segments.iter().any(|segment| {
        image
            .get(segment.payload.clone())
            .is_some_and(|payload| contains_bytes(payload, expected.source_commit.as_bytes()))
    }) {
        return Err(ImageValidationError::EmbeddedSourceCommitMismatch);
    }

    Ok(())
}

fn parse_fixed_string(bytes: &[u8]) -> Result<&str, ImageValidationError> {
    let value_end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[value_end..].iter().any(|byte| *byte != 0) {
        return Err(ImageValidationError::DescriptorVersionInvalid);
    }
    let value = std::str::from_utf8(&bytes[..value_end])
        .map_err(|_| ImageValidationError::DescriptorVersionInvalid)?;
    if value.is_empty() || !value.is_ascii() {
        return Err(ImageValidationError::DescriptorVersionInvalid);
    }

    Ok(value)
}

fn validate_trailer(
    image: &[u8],
    segment_end: usize,
    checksum: u8,
) -> Result<(), ImageValidationError> {
    let padding_len = (IMAGE_ALIGNMENT - 1 - (segment_end % IMAGE_ALIGNMENT)) % IMAGE_ALIGNMENT;
    let checksum_index = segment_end
        .checked_add(padding_len)
        .ok_or(ImageValidationError::TrailerRangeOverflow)?;
    let padding = image
        .get(segment_end..checksum_index)
        .ok_or(ImageValidationError::SegmentChecksumTruncated)?;
    if padding.iter().any(|byte| *byte != 0) {
        return Err(ImageValidationError::AlignmentPaddingInvalid);
    }
    let stored_checksum = image
        .get(checksum_index)
        .ok_or(ImageValidationError::SegmentChecksumTruncated)?;
    if *stored_checksum != checksum {
        return Err(ImageValidationError::SegmentChecksumMismatch);
    }
    let image_end = checksum_index
        .checked_add(1)
        .ok_or(ImageValidationError::TrailerRangeOverflow)?;
    let digest_end = image_end
        .checked_add(DIGEST_LEN)
        .ok_or(ImageValidationError::TrailerRangeOverflow)?;
    let appended_digest = image
        .get(image_end..digest_end)
        .ok_or(ImageValidationError::AppendedSha256Truncated)?;
    if image.len() > digest_end {
        return Err(ImageValidationError::TrailingData);
    }
    let expected_digest = Sha256::digest(
        image
            .get(..image_end)
            .ok_or(ImageValidationError::TrailerRangeOverflow)?,
    );
    if appended_digest != expected_digest.as_slice() {
        return Err(ImageValidationError::AppendedSha256Mismatch);
    }

    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let value = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let value = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
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
        descriptor[APP_ELF_SHA_OFFSET..APP_ELF_SHA_OFFSET + APP_ELF_SHA_LEN]
            .copy_from_slice(&APP_SHA);
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
}
