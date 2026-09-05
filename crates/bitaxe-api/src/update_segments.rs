//! State-preserving Ultra 205 update geometry, including flash-sector erasure.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One artifact written by an ordinary firmware update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSegment {
    pub artifact_kind: String,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid state-preserving update geometry")]
pub struct InvalidUpdateSegments;

/// Validates all five writes, including rounded erase ranges; NVS is never included.
pub fn validate_update_segments(segments: &[UpdateSegment]) -> Result<(), InvalidUpdateSegments> {
    let required = [
        ("bootloader", 0, 0x8000),
        ("partition_table_binary", 0x8000, 0x1000),
        ("firmware_ota_image", 0x10000, 0x400000),
        ("www_spiffs_image", 0x410000, 0x300000),
        ("otadata_initial", 0xf10000, 0x2000),
    ];
    if segments.len() != required.len() {
        return Err(InvalidUpdateSegments);
    }
    for (segment, (kind, offset, capacity)) in segments.iter().zip(required) {
        let rounded = segment
            .length
            .checked_add(0xfff)
            .ok_or(InvalidUpdateSegments)?
            & !0xfff;
        if segment.artifact_kind != kind
            || segment.offset != offset
            || segment.length == 0
            || rounded > capacity
        {
            return Err(InvalidUpdateSegments);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments() -> Vec<UpdateSegment> {
        [
            ("bootloader", 0, 0x6000),
            ("partition_table_binary", 0x8000, 0xc00),
            ("firmware_ota_image", 0x10000, 0x200001),
            ("www_spiffs_image", 0x410000, 0x300000),
            ("otadata_initial", 0xf10000, 0x2000),
        ]
        .into_iter()
        .map(|(kind, offset, length)| UpdateSegment {
            artifact_kind: kind.to_owned(),
            offset,
            length,
        })
        .collect()
    }

    #[test]
    fn ordinary_segments_exclude_nvs_and_unrelated_partition_erase_ranges() {
        // Arrange
        let segments = segments();
        // Act / Assert
        assert!(validate_update_segments(&segments).is_ok());
        for segment in segments {
            let end = segment.offset + ((segment.length + 0xfff) & !0xfff);
            assert!(end <= 0x9000 || segment.offset >= 0xf000);
            assert!(end <= 0xf12000);
        }
    }

    #[test]
    fn partition_table_cannot_erase_the_first_nvs_sector() {
        // Arrange
        let mut segments = segments();
        segments[1].length = 0x1001;
        // Act / Assert
        assert!(validate_update_segments(&segments).is_err());
    }

    #[test]
    fn extra_or_shifted_or_overflowing_segments_fail_closed() {
        // Arrange / Act / Assert
        let mut shifted = segments();
        shifted[2].offset = 0x9000;
        assert!(validate_update_segments(&shifted).is_err());
        let mut extra = segments();
        extra.push(extra[0].clone());
        assert!(validate_update_segments(&extra).is_err());
        let mut overflow = segments();
        overflow[0].length = u32::MAX;
        assert!(validate_update_segments(&overflow).is_err());
    }
}
