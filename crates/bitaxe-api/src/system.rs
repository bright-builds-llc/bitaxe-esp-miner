//! Pure `/api/system/info` response mapping.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/openapi.yaml`

use crate::{ApiSnapshot, SystemInfoWire};

/// Maps a typed API snapshot into the AxeOS system-info wire response.
#[must_use]
pub fn system_info_from_snapshot(snapshot: &ApiSnapshot) -> SystemInfoWire {
    SystemInfoWire::from_snapshot(snapshot)
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::messages::PoolDifficulty;
    use bitaxe_stratum::v1::state::{HashrateInputs, MiningRuntimeState, ShareDifficulty};

    use crate::system::system_info_from_snapshot;
    use crate::ApiSnapshot;

    #[test]
    fn system_info_maps_share_pool_fallback_and_hashrate_values_from_mining_runtime_state() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        let mut mining = MiningRuntimeState::default();
        mining.record_accepted_share(ShareDifficulty::new(8_192.0));
        mining.record_rejected_share("low difficulty");
        mining.record_rejected_share("low difficulty");
        mining.set_pool_difficulty(PoolDifficulty {
            difficulty: 1_638.4,
        });
        mining.set_fallback_active(true);
        mining.record_hashrate_inputs(HashrateInputs {
            hashes_done: 20_000_000_000,
            elapsed_ms: 10_000,
            rolling_hashrate_hs: 2_000_000_000.0,
        });
        snapshot.mining = mining;

        // Act
        let response = system_info_from_snapshot(&snapshot);

        // Assert
        assert_eq!(response.shares_accepted, 1);
        assert_eq!(response.shares_rejected, 2);
        assert_eq!(response.pool_difficulty, 1_638.4);
        assert_eq!(response.is_using_fallback_stratum, 1);
        assert_eq!(response.hash_rate, 2.0);
        assert_eq!(
            response.shares_rejected_reasons[0].message,
            "low difficulty"
        );
        assert_eq!(response.shares_rejected_reasons[0].count, 2);
    }
}
