use bitaxe_asic::bm1366::{
    crc::crc5,
    result::{parse_bm1366_result_frame, Bm1366NonceResult, Bm1366ParsedResult},
};

use super::*;

#[test]
fn production_correlation_blocks_nonce_below_pool_difficulty() {
    // Arrange
    let job_id = Bm1366JobId::new(0x80);
    let mut registry = ProductionWorkRegistry::new();
    registry
        .enqueue_pool_work(reference_work(job_id, 1_000.0))
        .expect("reference work should enqueue");
    let _dispatch = registry
        .dispatch_next()
        .expect("reference work should dispatch");
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: Bm1366NonceResult {
            job_id,
            nonce: 0x276e_8947,
            asic_index: 0,
            core_id: 19,
            small_core_id: 0,
            version_bits: 0,
        },
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Ignored {
            reason: NonSubmitReason::BelowPoolTarget
        }
    );
}

#[test]
fn production_wire_result_qualifies_reference_share_without_nonce_swap() {
    // Arrange
    let job_id = Bm1366JobId::new(0x80);
    let submit_nonce = 0x276e_8947_u32;
    let mut registry = ProductionWorkRegistry::new();
    registry
        .enqueue_pool_work(reference_work(job_id, 1.0))
        .expect("reference work should enqueue");
    let _dispatch = registry
        .dispatch_next()
        .expect("reference work should dispatch");
    let frame = reference_result_frame(job_id, submit_nonce);
    let parsed = parse_bm1366_result_frame(&frame, registry.valid_jobs(), 256)
        .expect("reference result frame should parse");
    let Bm1366ParsedResult::JobNonce(result) = parsed else {
        panic!("expected job nonce");
    };
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result,
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    let CorrelationOutcome::SubmitIntent(intent) = outcome else {
        panic!("reference difficulty-18 result should qualify at pool difficulty 1");
    };
    assert_eq!(intent.submission.nonce, submit_nonce);
}

fn reference_work(job_id: Bm1366JobId, pool_difficulty: f64) -> MiningWork {
    let prev_block_hash = crate::v1::coinbase::hex_32(
        "d02b10fc0d4711eae1a805af50a8a83312a2215e00017f2b0000000000000000",
        "reference_prev_block_hash",
    )
    .expect("reference previous block hash should decode");
    let merkle_root = crate::v1::coinbase::hex_32(
        "6d0359c451434605c52a5a9ce074340be47c2c63840731f9edf1db3f26b1cdd9",
        "reference_merkle_root",
    )
    .expect("reference merkle root should decode");

    MiningWork {
        stratum_job_id: "reference-low-difficulty".to_owned(),
        asic_job_id: job_id,
        fields: Bm1366WorkFields {
            starting_nonce: 0_u32.to_le_bytes(),
            nbits: 0x1705_ae3a_u32.to_le_bytes(),
            ntime: 0x646f_f1a9_u32.to_le_bytes(),
            merkle_root: reverse_reference_words(merkle_root),
            prev_block_hash: reverse_reference_words(reverse_reference_word_bytes(prev_block_hash)),
            version: 0x2000_0004_u32.to_le_bytes(),
        },
        extranonce2: "00000000".to_owned(),
        ntime: 0x646f_f1a9,
        maybe_pool_difficulty: Some(PoolDifficulty {
            difficulty: pool_difficulty,
        }),
        clean_jobs: false,
        maybe_version_mask: None,
    }
}

fn reference_result_frame(job_id: Bm1366JobId, submit_nonce: u32) -> [u8; 11] {
    let mut body = [0; 8];
    body[0..4].copy_from_slice(&submit_nonce.to_le_bytes());
    body[4] = 0;
    body[5] = job_id.raw();
    body[6..8].copy_from_slice(&0_u16.to_be_bytes());
    let mut frame = [0; 11];
    frame[0..2].copy_from_slice(&0xaa55_u16.to_be_bytes());
    frame[2..10].copy_from_slice(&body);
    for residue in 0..32_u8 {
        frame[10] = 0x80 | residue;
        if crc5(&frame[2..]) == 0 {
            return frame;
        }
    }
    panic!("reference result body should admit a CRC5 residue");
}

fn reverse_reference_words(input: [u8; 32]) -> [u8; 32] {
    let mut output = [0; 32];
    for index in 0..8 {
        let source_start = (7 - index) * 4;
        let target_start = index * 4;
        output[target_start..target_start + 4]
            .copy_from_slice(&input[source_start..source_start + 4]);
    }
    output
}

fn reverse_reference_word_bytes(mut input: [u8; 32]) -> [u8; 32] {
    for word in input.chunks_exact_mut(4) {
        word.reverse();
    }
    input
}
