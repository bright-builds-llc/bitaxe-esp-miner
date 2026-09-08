//! Fixed independent header/digest; never derived from the implementation under test.
use super::*;
use crate::v1::{
    messages::{PoolDifficulty, VersionMask},
    production_work::{
        CorrelationOutcome, NonSubmitReason, ProductionNonceObservation, ProductionWorkRegistry,
    },
};
use bitaxe_asic::bm1366::{
    result::{parse_bm1366_result_frame, Bm1366ParsedResult},
    work::{Bm1366JobId, Bm1366WorkFields},
};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/nonzero-version-known-answer/fixture.json"
    ))
    .expect("public fixed fixture")
}
fn number(f: &Value, key: &str) -> u32 {
    u32::try_from(f[key].as_u64().expect("fixture integer")).expect("fixture u32")
}
fn bytes(f: &Value, key: &str) -> Vec<u8> {
    f[key]
        .as_str()
        .expect("fixture hex")
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII hex"), 16).expect("hex byte")
        })
        .collect()
}
fn work(f: &Value, difficulty: f64) -> MiningWork {
    MiningWork {
        stratum_job_id: "synthetic-known-answer".to_owned(),
        asic_job_id: Bm1366JobId::new(0x28),
        fields: Bm1366WorkFields {
            starting_nonce: 0u32.to_le_bytes(),
            nbits: number(f, "nbits").to_le_bytes(),
            ntime: number(f, "ntime").to_le_bytes(),
            merkle_root: bytes(f, "work_merkle_hex")
                .try_into()
                .expect("32-byte merkle"),
            prev_block_hash: bytes(f, "work_prev_hash_hex")
                .try_into()
                .expect("32-byte prevhash"),
            version: number(f, "base_version").to_le_bytes(),
        },
        extranonce2: "00000000".to_owned(),
        ntime: number(f, "ntime"),
        maybe_pool_difficulty: Some(PoolDifficulty { difficulty }),
        clean_jobs: false,
        maybe_version_mask: Some(VersionMask { mask: 0x1fffe000 }),
    }
}
#[test]
fn nonzero_wire_version_reconstructs_independent_header_and_sha256d() {
    // Arrange
    let f = fixture();
    let work = work(&f, 1.0);
    let mut registry = ProductionWorkRegistry::new();
    registry.enqueue_pool_work(work.clone()).expect("enqueue");
    let dispatched = registry.dispatch_next().expect("dispatch");
    // Act
    let parsed =
        parse_bm1366_result_frame(&bytes(&f, "uart_result_hex"), registry.valid_jobs(), 256)
            .expect("known wire response");
    let Bm1366ParsedResult::JobNonce(result) = parsed else {
        panic!("expected nonce")
    };
    let header = reconstructed_header(&work, result);
    let hash = double_sha256(&header);
    // Assert
    assert_eq!(
        dispatched.work_payload.payload().bytes().as_slice(),
        bytes(&f, "work_payload_hex")
    );
    assert_eq!(result.version_bits, number(&f, "asic_version_bits"));
    assert_ne!(result.version_bits, 0);
    assert_eq!(result.nonce, number(&f, "nonce"));
    assert_eq!(header.as_slice(), bytes(&f, "header_hex"));
    assert_eq!(hash.as_slice(), bytes(&f, "sha256d_hex"));
    let wrong_version = Bm1366NonceResult {
        version_bits: 0,
        ..result
    };
    assert_ne!(
        double_sha256(&reconstructed_header(&work, wrong_version)),
        hash
    );
    let wrong_nonce = Bm1366NonceResult {
        nonce: result.nonce.swap_bytes(),
        ..result
    };
    assert_ne!(
        double_sha256(&reconstructed_header(&work, wrong_nonce)),
        hash
    );
}
#[test]
fn nonzero_wire_nonce_crosses_existing_correlation_and_target_qualification() {
    let f = fixture();
    let expected = f["bitcoin_difficulty"]
        .as_f64()
        .expect("independent numeric difficulty");
    for (target, qualifies) in [(expected / 2.0, true), (expected * 2.0, false)] {
        // Arrange
        let work = work(&f, target);
        let mut registry = ProductionWorkRegistry::new();
        registry.enqueue_pool_work(work).expect("enqueue");
        registry.dispatch_next().expect("dispatch");
        let parsed =
            parse_bm1366_result_frame(&bytes(&f, "uart_result_hex"), registry.valid_jobs(), 256)
                .expect("known wire response");
        let Bm1366ParsedResult::JobNonce(result) = parsed else {
            panic!("expected nonce")
        };
        // Act
        let receipt = registry.correlate_nonce_result_with_receipt(ProductionNonceObservation {
            observed_generation: registry.generation(),
            result,
        });
        // Assert
        let candidate = receipt
            .maybe_scoreboard_candidate
            .expect("candidate retains validated hash result");
        assert!((candidate.difficulty() - expected).abs() / expected < 1e-12);
        assert!(!candidate.matches_expected_asic_filter(),"synthetic hash misses software-expected hardware filter even when below synthetic pool target");
        if qualifies {
            assert!(matches!(
                receipt.outcome,
                CorrelationOutcome::SubmitIntent(_)
            ));
        } else {
            assert!(matches!(
                receipt.outcome,
                CorrelationOutcome::Ignored {
                    reason: NonSubmitReason::BelowPoolTarget
                }
            ));
        }
    }
}
#[test]
fn known_reference_candidate_can_match_asic_filter_but_miss_pool_target() {
    // Arrange: independent upstream-derived reference has software difficulty about683.
    let mut f = fixture();
    f["base_version"] = Value::from(0x20000004u32);
    f["ntime"] = Value::from(0x647025b5u32);
    f["nbits"] = Value::from(0x1705ae3au32);
    let prev = crate::v1::coinbase::hex_32(
        "0c859545a3498373a57452fac22eb7113df2a465000543520000000000000000",
        "public reference prevhash",
    )
    .expect("hex");
    let merkle = crate::v1::coinbase::hex_32(
        "5bdc1968499c3393873edf8e07a1c3a50a97fc3a9d1a376bbf77087dd63778eb",
        "public reference merkle",
    )
    .expect("hex");
    let mut work = work(&f, 1000.0);
    let mut prev_header = prev;
    for chunk in prev_header.chunks_exact_mut(4) {
        chunk.reverse();
    }
    work.fields.prev_block_hash = reverse_32bit_words(prev_header);
    work.fields.merkle_root = reverse_32bit_words(merkle);
    let mut registry = ProductionWorkRegistry::new();
    registry.enqueue_pool_work(work).expect("enqueue");
    registry.dispatch_next().expect("dispatch");
    // Act
    let receipt = registry.correlate_nonce_result_with_receipt(ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: Bm1366NonceResult {
            job_id: Bm1366JobId::new(0x28),
            nonce: 0x0a029ed1,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            version_bits: 0,
        },
    });
    // Assert
    assert!(matches!(
        receipt.outcome,
        CorrelationOutcome::Ignored {
            reason: NonSubmitReason::BelowPoolTarget
        }
    ));
    assert!(receipt
        .maybe_scoreboard_candidate
        .expect("candidate")
        .matches_expected_asic_filter());
}
