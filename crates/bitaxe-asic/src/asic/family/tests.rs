use super::*;

#[test]
fn each_family_steps_its_job_ids_by_its_own_amount() {
    // Act
    let steps = [AsicFamily::Bm1366, AsicFamily::Bm1370].map(AsicFamily::job_id_step);
    // Assert
    assert_eq!(steps, [8, 24]);
}

#[test]
fn the_first_job_id_is_one_step_from_zero() {
    // Act
    let first = [AsicFamily::Bm1366, AsicFamily::Bm1370].map(|family| family.first_job_id().raw());
    // Assert
    assert_eq!(first, [8, 24]);
}

#[test]
fn the_bm1370_cycles_through_sixteen_job_ids() {
    // Arrange
    let mut job_id = JobId::new(0);
    let mut seen = std::collections::BTreeSet::new();
    // Act
    for _ in 0..32 {
        job_id = job_id.advance(AsicFamily::Bm1370);
        seen.insert(job_id.raw());
    }
    // Assert
    assert_eq!(seen.len(), 16);
}

#[test]
fn the_job_interval_follows_each_chip_s_timeout_and_chain_length() {
    // Act
    let intervals = [
        AsicFamily::Bm1366.job_interval_ms(1),
        AsicFamily::Bm1370.job_interval_ms(1),
        AsicFamily::Bm1370.job_interval_ms(3),
        AsicFamily::Bm1370.job_interval_ms(0),
    ];
    // Assert
    assert_eq!(intervals, [2_000, 500, 125, 500]);
}

#[test]
fn the_bm1366_job_interval_matches_its_mining_ready_helper() {
    // Act
    let same = (0..9).all(|asics| {
        AsicFamily::Bm1366.job_interval_ms(asics)
            == crate::bm1366::mining_ready::bm1366_job_interval_ms(asics)
    });
    // Assert
    assert!(same);
}

#[test]
fn catalog_models_map_to_their_families() {
    // Act
    let families = ["BM1366", "BM1370", "BM1368"].map(AsicFamily::from_model);
    // Assert
    assert_eq!(
        families,
        [Some(AsicFamily::Bm1366), Some(AsicFamily::Bm1370), None]
    );
}

#[test]
fn a_damaged_frame_is_discarded_by_either_family() {
    // Arrange
    let frame = [0x55; 11];
    // Act
    let results = [AsicFamily::Bm1366, AsicFamily::Bm1370]
        .map(|family| family.classify_production_result(&frame, &ValidJobIds::empty(), 256));
    // Assert
    assert_eq!(
        results,
        [ProductionResult::Discarded(crate::asic::ResultDiscardReason::InvalidPreamble); 2]
    );
}

#[test]
fn the_bm1366_family_counts_a_bm1366_chip_id_frame() {
    // Arrange
    let mut frame = vec![
        0xaa, 0x55, 0x13, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    // The frame's CRC is a zero residue over everything after the preamble.
    frame[10] = (0..=u8::MAX)
        .find(|&last| {
            frame[10] = last;
            crate::bm1366::crc::crc5(&frame[2..]) == 0
        })
        .expect("a residue-zero byte exists");
    // Act
    let dispositions = [AsicFamily::Bm1366, AsicFamily::Bm1370]
        .map(|family| family.classify_chip_id_count_frame(&frame));
    // Assert
    assert_eq!(
        dispositions,
        [
            Ok(super::super::ChipIdCountFrameDisposition::Accept),
            Ok(super::super::ChipIdCountFrameDisposition::SoftRetry)
        ]
    );
}

#[test]
fn a_family_fault_keeps_the_chip_s_own_message() {
    // Act
    let fault = AsicFamily::Bm1366
        .classify_chip_id_count_frame(&[0xaa])
        .expect_err("a short frame fails the count");
    // Assert
    assert_eq!(
        fault.to_string(),
        crate::bm1366::chip_detect::classify_chip_id_count_frame(&[0xaa])
            .expect_err("a short frame fails the count")
            .to_string()
    );
}
