//! Device-local pool-configuration continuity (BWG-007).
//!
//! The digest covers every stored primary and fallback pool key. It lives only in RAM and is
//! compared on the device; only the resulting boolean is ever reported. The digest type has no
//! serialization, no byte accessor and a redacting `Debug`, so neither a pool value nor its digest
//! can leave the device or reach a log.
use bitaxe_api::SettingsAdapterFailure;
use bitaxe_config::nvs::StoredValueKind;
use sha2::{Digest, Sha256};
use std::fmt;
use zeroize::Zeroize;

/// Every stored primary and fallback pool key, in canonical (sorted) order. Includes the legacy
/// `fbstratumxnsum` key that factory defaults still write beside `stratumfbxnsub`.
pub(crate) const POOL_CONFIGURATION_KEYS: &[&str] = &[
    "fbstratumcert",
    "fbstratumdecode",
    "fbstratumdiff",
    "fbstratumpass",
    "fbstratumport",
    "fbstratumprot",
    "fbstratumtls",
    "fbstratumurl",
    "fbstratumuser",
    "fbstratumxnsum",
    "fbsv2authpubk",
    "fbsv2chantype",
    "stratumcert",
    "stratumdecode",
    "stratumdiff",
    "stratumfbxnsub",
    "stratumpass",
    "stratumport",
    "stratumprot",
    "stratumtls",
    "stratumurl",
    "stratumuser",
    "stratumxnsub",
    "sv2authpubkey",
    "sv2chantype",
    "usefbstartum",
];

const DOMAIN: &[u8] = b"worker-pool-configuration-v1\0";

/// SHA-256 over the canonical stored pool configuration; comparable on the device only.
#[derive(Eq, PartialEq)]
pub(crate) struct PoolConfigurationDigest([u8; 32]);

impl fmt::Debug for PoolConfigurationDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PoolConfigurationDigest([private])")
    }
}

impl Drop for PoolConfigurationDigest {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Hashes every pool key in canonical order; an absent key hashes as a distinct marker.
#[inline(never)]
pub(crate) fn digest(
    mut read: impl FnMut(&str) -> Result<Option<StoredValueKind>, SettingsAdapterFailure>,
) -> Result<PoolConfigurationDigest, SettingsAdapterFailure> {
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    for key in POOL_CONFIGURATION_KEYS {
        hash.update((key.len() as u16).to_be_bytes());
        hash.update(key.as_bytes());
        match read(key)? {
            None => hash.update([0]),
            Some(StoredValueKind::String(mut value)) => {
                hash.update([1]);
                hash.update((value.len() as u32).to_be_bytes());
                hash.update(value.as_bytes());
                value.zeroize();
            }
            Some(StoredValueKind::U16(value)) => {
                hash.update([2]);
                hash.update(value.to_be_bytes());
            }
            Some(StoredValueKind::I32(value)) => {
                hash.update([3]);
                hash.update(value.to_be_bytes());
            }
            Some(StoredValueKind::U64(value)) => {
                hash.update([4]);
                hash.update(value.to_be_bytes());
            }
        }
    }
    Ok(PoolConfigurationDigest(hash.finalize().into()))
}

/// True only when the boot capture exists and the current configuration reads back identical.
/// A missing capture or an unreadable current configuration cannot prove continuity, so both
/// fail closed.
pub(crate) fn unchanged_since_boot(
    maybe_boot: Option<&PoolConfigurationDigest>,
    current: Result<PoolConfigurationDigest, SettingsAdapterFailure>,
) -> bool {
    let (Some(boot), Ok(current)) = (maybe_boot, current) else {
        return false;
    };
    *boot == current
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitaxe_config::{all_settings_schema, project_settings_schema};

    fn stored(overrides: &[(&str, StoredValueKind)]) -> Vec<(String, StoredValueKind)> {
        let mut values: Vec<(String, StoredValueKind)> = [
            (
                "stratumurl",
                StoredValueKind::String("pool.example".to_owned()),
            ),
            ("stratumport", StoredValueKind::U16(3333)),
            (
                "stratumuser",
                StoredValueKind::String("owner.worker".to_owned()),
            ),
            ("stratumpass", StoredValueKind::String("x".to_owned())),
            (
                "fbstratumurl",
                StoredValueKind::String("fallback.example".to_owned()),
            ),
            ("fbstratumport", StoredValueKind::U16(4444)),
            ("rotation", StoredValueKind::U16(0)),
            ("hostname", StoredValueKind::String("bitaxe".to_owned())),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
        for (key, value) in overrides {
            values.retain(|(existing, _)| existing != key);
            values.push(((*key).to_owned(), value.clone()));
        }
        values
    }

    fn digest_of(values: &[(String, StoredValueKind)]) -> PoolConfigurationDigest {
        digest(|key| {
            Ok(values
                .iter()
                .find(|(stored_key, _)| stored_key == key)
                .map(|(_, value)| value.clone()))
        })
        .expect("pool configuration digest")
    }

    fn unchanged(boot: &[(String, StoredValueKind)], now: &[(String, StoredValueKind)]) -> bool {
        unchanged_since_boot(Some(&digest_of(boot)), Ok(digest_of(now)))
    }

    fn is_pool_schema_key(key: &str) -> bool {
        key.starts_with("stratum")
            || key.starts_with("fbstratum")
            || key.starts_with("sv2")
            || key.starts_with("fbsv2")
            || key == "usefbstartum"
    }

    #[test]
    fn identical_stored_configuration_is_unchanged() {
        // Arrange
        let boot = stored(&[]);

        // Act
        let result = unchanged(&boot, &stored(&[]));

        // Assert
        assert!(result);
    }

    #[test]
    fn a_change_to_any_pool_key_flips_the_boolean() {
        // Arrange
        let boot = stored(&[]);

        for key in POOL_CONFIGURATION_KEYS {
            // Act
            let added = unchanged(&boot, &stored(&[(key, StoredValueKind::U16(7))]));
            let replaced = unchanged(
                &stored(&[(key, StoredValueKind::String("a".to_owned()))]),
                &stored(&[(key, StoredValueKind::String("b".to_owned()))]),
            );

            // Assert
            assert!(!added, "{key} addition must be detected");
            assert!(!replaced, "{key} replacement must be detected");
        }
    }

    #[test]
    fn removing_a_stored_pool_key_flips_the_boolean() {
        // Arrange
        let boot = stored(&[]);
        let mut now = stored(&[]);
        now.retain(|(key, _)| key != "stratumpass");

        // Act
        let result = unchanged(&boot, &now);

        // Assert
        assert!(!result);
    }

    #[test]
    fn a_change_to_a_non_pool_setting_does_not_flip_the_boolean() {
        // Arrange
        let boot = stored(&[]);
        let now = stored(&[
            ("rotation", StoredValueKind::U16(180)),
            ("hostname", StoredValueKind::String("renamed".to_owned())),
            ("wifipass", StoredValueKind::String("changed".to_owned())),
        ]);

        // Act
        let result = unchanged(&boot, &now);

        // Assert
        assert!(result);
    }

    #[test]
    fn storage_order_does_not_change_the_digest() {
        // Arrange
        let forward = stored(&[]);
        let mut reversed = forward.clone();
        reversed.reverse();

        // Act
        let result = unchanged(&forward, &reversed);

        // Assert
        assert!(result);
    }

    #[test]
    fn digest_is_stable_across_repeated_reads() {
        // Arrange
        let values = stored(&[]);

        // Act
        let first = digest_of(&values);
        let second = digest_of(&values);

        // Assert
        assert_eq!(first, second);
    }

    #[test]
    fn missing_boot_capture_fails_closed() {
        // Arrange
        let current = digest_of(&stored(&[]));

        // Act
        let result = unchanged_since_boot(None, Ok(current));

        // Assert
        assert!(!result);
    }

    #[test]
    fn unreadable_current_configuration_fails_closed() {
        // Arrange
        let boot = digest_of(&stored(&[]));

        // Act
        let result = unchanged_since_boot(Some(&boot), Err(SettingsAdapterFailure::failed("read")));

        // Assert
        assert!(!result);
    }

    #[test]
    fn reads_exactly_the_canonical_pool_keys_in_sorted_order() {
        // Arrange
        let mut requested = Vec::new();

        // Act
        digest(|key| {
            requested.push(key.to_owned());
            Ok(None)
        })
        .expect("empty pool configuration");

        // Assert
        assert_eq!(requested, POOL_CONFIGURATION_KEYS);
        assert!(POOL_CONFIGURATION_KEYS
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn covers_every_stored_primary_and_fallback_pool_schema_key() {
        // Arrange
        let schema_pool_keys: Vec<String> = all_settings_schema()
            .into_iter()
            .chain(project_settings_schema())
            .map(|row| row.key.as_str().to_owned())
            .filter(|key| is_pool_schema_key(key))
            .collect();

        // Act
        let uncovered: Vec<&String> = schema_pool_keys
            .iter()
            .filter(|key| !POOL_CONFIGURATION_KEYS.contains(&key.as_str()))
            .collect();
        let unknown: Vec<&&str> = POOL_CONFIGURATION_KEYS
            .iter()
            .filter(|key| **key != "fbstratumxnsum" && !schema_pool_keys.iter().any(|k| k == *key))
            .collect();

        // Assert
        assert!(uncovered.is_empty(), "uncovered pool keys: {uncovered:?}");
        assert!(unknown.is_empty(), "keys outside the schema: {unknown:?}");
    }

    #[test]
    fn debug_output_redacts_the_digest() {
        // Arrange
        let digest = digest_of(&stored(&[]));

        // Act
        let rendered = format!("{digest:?}");

        // Assert
        assert_eq!(rendered, "PoolConfigurationDigest([private])");
    }
}
