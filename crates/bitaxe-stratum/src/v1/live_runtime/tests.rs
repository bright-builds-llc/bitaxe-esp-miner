use bitaxe_asic::bm1366::result::Bm1366NonceResult;

use super::*;

mod classification;
mod lifecycle;
mod work;

fn runtime() -> LiveStratumRuntime {
    LiveStratumRuntime::new_with_generation(
        LiveRuntimeConfig {
            model: "ultra".to_owned(),
            version: "205".to_owned(),
            credentials: LivePoolCredentials {
                username: "synthetic-user".to_owned(),
                password: "synthetic-secret".to_owned(),
            },
        },
        PoolSessionGeneration::initial(),
    )
}

fn runtime_with_extranonce() -> LiveStratumRuntime {
    let mut runtime = runtime();
    runtime
        .apply_server_message(StratumV1ServerMessage::SetExtranonce(extranonce()))
        .expect("extranonce assignment should apply");
    runtime
}

fn extranonce() -> ExtranonceAssignment {
    ExtranonceAssignment {
        extranonce1: "4de05269".to_owned(),
        extranonce2_len: 4,
    }
}

fn notify(clean_jobs: bool) -> MiningNotify {
    MiningNotify {
        job_id: "synthetic-job".to_owned(),
        prev_block_hash: "00".repeat(32),
        coinbase_1: "0200000001".to_owned(),
        coinbase_2: "ffffffff".to_owned(),
        merkle_branches: Vec::new(),
        version: 0x2000_0004,
        nbits: 0x1705_ae3a,
        ntime: 0x6470_25b5,
        clean_jobs,
    }
}

fn response(success: bool) -> StratumResponse {
    StratumResponse {
        maybe_id: None,
        success,
        maybe_error: None,
        maybe_extranonce: None,
        maybe_version_mask: None,
    }
}

fn nonce_result(job_id: Bm1366JobId) -> Bm1366NonceResult {
    Bm1366NonceResult {
        job_id,
        nonce: 0x1234_5678,
        asic_index: 0,
        core_id: 1,
        small_core_id: 0,
        version_bits: 0x0000_2000,
    }
}
