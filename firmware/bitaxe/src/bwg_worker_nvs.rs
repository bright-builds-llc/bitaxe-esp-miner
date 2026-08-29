//! Exclusive NVS and ESP entropy adapters for BWG Device Identity and replay state.

use std::collections::BTreeMap;

use bitaxe_worker_control::{
    AcceptedSequenceStore, DeviceIdentitySeedGenerator, DeviceIdentitySeedStore, IdentityLoadError,
    LeaseAuthorizationError, PersistedWorkerEffectState, SequenceStoreResult,
};
use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_svc::sys;
use zeroize::{Zeroize, Zeroizing};

use crate::startup::BootMiningBaselineConfirmed;

const NAMESPACE: &str = "bwg_worker";
const IDENTITY_KEY: &str = "device_seed";
const SEQUENCE_KEY: &str = "lease_seq";
const EFFECT_PENDING_KEY: &str = "effect_pending";
const MAXIMUM_SEQUENCE_DOCUMENT_BYTES: usize = 512;

/// Sole owner of the dedicated BWG NVS namespace for one boot lifetime.
pub(crate) struct BwgWorkerNvs {
    nvs: EspNvs<NvsDefault>,
}

impl BwgWorkerNvs {
    pub(crate) fn open() -> Result<Self, IdentityLoadError> {
        let partition = crate::settings_adapter::default_nvs_partition()
            .map_err(|_| IdentityLoadError::Storage)?;
        let nvs =
            EspNvs::new(partition, NAMESPACE, true).map_err(|_| IdentityLoadError::Storage)?;
        Ok(Self { nvs })
    }

    fn sequence_state(&self) -> Result<BTreeMap<String, u64>, LeaseAuthorizationError> {
        let Some(length) = self
            .nvs
            .blob_len(SEQUENCE_KEY)
            .map_err(|_| LeaseAuthorizationError::Persistence)?
        else {
            return Ok(BTreeMap::new());
        };
        if length == 0 || length > MAXIMUM_SEQUENCE_DOCUMENT_BYTES {
            return Err(LeaseAuthorizationError::Persistence);
        }
        let mut bytes = vec![0_u8; length];
        let Some(value) = self
            .nvs
            .get_blob(SEQUENCE_KEY, &mut bytes)
            .map_err(|_| LeaseAuthorizationError::Persistence)?
        else {
            return Err(LeaseAuthorizationError::Persistence);
        };
        let state: BTreeMap<String, u64> =
            serde_json::from_slice(value).map_err(|_| LeaseAuthorizationError::Persistence)?;
        if state.len() > 8
            || state
                .iter()
                .any(|(key_id, sequence)| !valid_key_id(key_id) || *sequence == 0)
        {
            return Err(LeaseAuthorizationError::Persistence);
        }
        Ok(state)
    }

    fn store_sequence_state(
        &mut self,
        state: &BTreeMap<String, u64>,
    ) -> Result<(), LeaseAuthorizationError> {
        let bytes = serde_json::to_vec(state).map_err(|_| LeaseAuthorizationError::Persistence)?;
        if bytes.len() > MAXIMUM_SEQUENCE_DOCUMENT_BYTES {
            return Err(LeaseAuthorizationError::Persistence);
        }
        self.nvs
            .set_blob(SEQUENCE_KEY, &bytes)
            .map_err(|_| LeaseAuthorizationError::Persistence)?;
        if self.sequence_state()? != *state {
            return Err(LeaseAuthorizationError::Persistence);
        }
        Ok(())
    }

    pub(crate) fn confirm_reboot_baseline(
        &mut self,
        _proof: BootMiningBaselineConfirmed,
    ) -> Result<bool, LeaseAuthorizationError> {
        let current = self.effect_state()?;
        let confirmed = current.after_boot_baseline();
        if confirmed != current {
            self.store_effect_state(confirmed)?;
        }
        Ok(confirmed.requires_reboot_report())
    }

    fn effect_state(&self) -> Result<PersistedWorkerEffectState, LeaseAuthorizationError> {
        let value = self
            .nvs
            .get_u8(EFFECT_PENDING_KEY)
            .map_err(|_| LeaseAuthorizationError::Persistence)?;
        PersistedWorkerEffectState::parse(value)
    }

    fn store_effect_state(
        &mut self,
        state: PersistedWorkerEffectState,
    ) -> Result<(), LeaseAuthorizationError> {
        match state.stored_value() {
            Some(value) => self
                .nvs
                .set_u8(EFFECT_PENDING_KEY, value)
                .map_err(|_| LeaseAuthorizationError::Persistence)?,
            None => {
                self.nvs
                    .remove(EFFECT_PENDING_KEY)
                    .map_err(|_| LeaseAuthorizationError::Persistence)?;
            }
        }
        if self.effect_state()? == state {
            Ok(())
        } else {
            Err(LeaseAuthorizationError::Persistence)
        }
    }
}

impl DeviceIdentitySeedStore for BwgWorkerNvs {
    fn load_seed(&self) -> Result<Option<Vec<u8>>, IdentityLoadError> {
        let Some(length) = self
            .nvs
            .blob_len(IDENTITY_KEY)
            .map_err(|_| IdentityLoadError::Storage)?
        else {
            return Ok(None);
        };
        if length != 32 {
            return Err(IdentityLoadError::Corrupt);
        }
        let mut seed = vec![0_u8; length];
        let maybe_value = match self.nvs.get_blob(IDENTITY_KEY, &mut seed) {
            Ok(maybe_value) => maybe_value,
            Err(_) => {
                seed.zeroize();
                return Err(IdentityLoadError::Storage);
            }
        };
        if maybe_value.is_none() {
            seed.zeroize();
            return Err(IdentityLoadError::Storage);
        }
        Ok(Some(seed))
    }

    fn store_seed_atomic(&mut self, seed: &[u8; 32]) -> Result<(), IdentityLoadError> {
        self.nvs
            .set_blob(IDENTITY_KEY, seed)
            .map_err(|_| IdentityLoadError::Storage)?;
        let mut confirmed = Zeroizing::new([0_u8; 32]);
        let result = self
            .nvs
            .get_blob(IDENTITY_KEY, &mut confirmed[..])
            .map_err(|_| IdentityLoadError::Storage)?;
        let matches = result.is_some_and(|stored| stored == seed);
        if !matches {
            return Err(IdentityLoadError::Storage);
        }
        Ok(())
    }
}

impl AcceptedSequenceStore for BwgWorkerNvs {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.store_effect_state(PersistedWorkerEffectState::EffectPending)
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        self.store_effect_state(PersistedWorkerEffectState::Clear)
    }

    fn load(&self, key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError> {
        if !valid_key_id(key_id) {
            return Err(LeaseAuthorizationError::Persistence);
        }
        Ok(self.sequence_state()?.get(key_id).copied())
    }

    fn compare_and_store(
        &mut self,
        key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError> {
        if !valid_key_id(key_id) || next == 0 {
            return Err(LeaseAuthorizationError::Persistence);
        }
        let mut state = self.sequence_state()?;
        let current = state.get(key_id).copied();
        if current == Some(next) {
            return Ok(SequenceStoreResult::AlreadyCommitted);
        }
        if current != expected {
            return Ok(SequenceStoreResult::Stale);
        }
        state.insert(key_id.to_owned(), next);
        if state.len() > 8 {
            return Err(LeaseAuthorizationError::Persistence);
        }
        self.store_sequence_state(&state)?;
        Ok(SequenceStoreResult::Committed)
    }
}

/// ESP-IDF hardware entropy source used only for first identity creation.
pub(crate) struct EspDeviceIdentitySeedGenerator;

impl DeviceIdentitySeedGenerator for EspDeviceIdentitySeedGenerator {
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), IdentityLoadError> {
        unsafe { sys::esp_fill_random(seed.as_mut_ptr().cast(), seed.len()) };
        Ok(())
    }
}

fn valid_key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
