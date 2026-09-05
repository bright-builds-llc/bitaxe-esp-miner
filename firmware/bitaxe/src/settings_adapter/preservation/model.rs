//! Versioned public-setting allowlist. Never read network, pool, host, swarm or scoreboard keys.
use bitaxe_api::SettingsAdapterFailure;
use bitaxe_config::nvs::StoredValueKind;
use bitaxe_worker_control::{SettingsPreservation, StateFingerprint};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

const KEYS: &[&str] = &[
    "asicfrequency_f",
    "asicvoltage",
    "autofanspeed",
    "display",
    "displayOffset",
    "displayTimeout",
    "invertscreen",
    "manualfanspeed",
    "mineonboot",
    "minfanspeed",
    "oc_enabled",
    "overheat_mode",
    "rotation",
    "statsFrequency",
    "temptarget",
    "themecolors",
    "themescheme",
];

pub(super) fn fingerprint(
    mut read: impl FnMut(&str) -> Result<Option<StoredValueKind>, SettingsAdapterFailure>,
) -> Result<SettingsPreservation, SettingsAdapterFailure> {
    let mut hash = Sha256::new();
    hash.update(b"worker-public-settings-v1\0");
    let mut mine_on_boot = true;
    for key in KEYS {
        hash.update((key.len() as u16).to_be_bytes());
        hash.update(key.as_bytes());
        let maybe_value = read(key)?;
        if *key == "mineonboot" {
            mine_on_boot = match &maybe_value {
                Some(StoredValueKind::U16(0)) => false,
                None | Some(StoredValueKind::U16(1)) => true,
                _ => {
                    return Err(SettingsAdapterFailure::failed(
                        "boot preference storage invalid",
                    ))
                }
            };
        }
        match maybe_value {
            None => hash.update([0]),
            Some(StoredValueKind::String(mut value)) => {
                if !matches!(*key, "asicfrequency_f" | "themecolors" | "themescheme") {
                    value.zeroize();
                    return Err(SettingsAdapterFailure::failed(
                        "public setting storage type invalid",
                    ));
                }
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
    Ok(SettingsPreservation::new(
        StateFingerprint::from_digest(hash.finalize().into()),
        mine_on_boot,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn digest(rotation: u16) -> String {
        let settings = fingerprint(|key| {
            Ok(match key {
                "rotation" => Some(StoredValueKind::U16(rotation)),
                "mineonboot" => Some(StoredValueKind::U16(0)),
                _ => None,
            })
        })
        .expect("public settings");
        serde_json::to_string(&settings).expect("fingerprint projection")
    }
    #[test]
    fn changed_public_setting_changes_the_fingerprint() {
        // Arrange / Act / Assert
        assert_ne!(digest(0), digest(180));
        assert_eq!(digest(180), digest(180));
    }
    #[test]
    fn private_network_and_owner_values_are_never_read_or_hashed() {
        // Arrange
        let mut requested = Vec::new();
        // Act
        fingerprint(|key| {
            requested.push(key.to_owned());
            Ok(None)
        })
        .expect("empty public settings");
        // Assert
        for forbidden in [
            "ssid",
            "wifipass",
            "hostname",
            "stratumurl",
            "stratumuser",
            "stratumpass",
            "stratumport",
            "swarmconfig",
            "scoreboard",
        ] {
            assert!(!requested.iter().any(|key| key == forbidden));
        }
        assert_eq!(requested.len(), KEYS.len());
    }
}
