use std::fmt;

use ed25519_dalek::{Signer, SigningKey};
use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

use crate::codec::base64_url;
use crate::possession::{PossessionClaims, PossessionError, PossessionRequest, PossessionResponse};

/// Persistent signing identity reconstructed only from the private NVS seed.
pub struct DeviceIdentity {
    signing_key: SigningKey,
}

pub trait DeviceIdentitySeedStore {
    fn load_seed(&self) -> Result<Option<Vec<u8>>, IdentityLoadError>;
    fn store_seed_atomic(&mut self, seed: &[u8; 32]) -> Result<(), IdentityLoadError>;
}

pub trait DeviceIdentitySeedGenerator {
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), IdentityLoadError>;
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IdentityLoadError {
    #[error("Device Identity storage failed")]
    Storage,
    #[error("Device Identity entropy failed")]
    Entropy,
    #[error("Device Identity is corrupt")]
    Corrupt,
}

impl IdentityLoadError {
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::Storage => "identity_storage_failed",
            Self::Entropy => "identity_entropy_failed",
            Self::Corrupt => "identity_corrupt",
        }
    }
}

pub fn load_or_generate_device_identity(
    store: &mut impl DeviceIdentitySeedStore,
    generator: &mut impl DeviceIdentitySeedGenerator,
) -> Result<DeviceIdentity, IdentityLoadError> {
    if let Some(persisted) = store.load_seed()? {
        let persisted = Zeroizing::new(persisted);
        if persisted.len() != 32 {
            return Err(IdentityLoadError::Corrupt);
        }
        let mut seed = Zeroizing::new([0_u8; 32]);
        seed.copy_from_slice(&persisted);
        return Ok(DeviceIdentity::from_seed(*seed));
    }
    let mut seed = Zeroizing::new([0_u8; 32]);
    generator.fill_seed(&mut seed)?;
    if seed.iter().all(|byte| *byte == 0) {
        return Err(IdentityLoadError::Entropy);
    }
    store.store_seed_atomic(&seed)?;
    Ok(DeviceIdentity::from_seed(*seed))
}

impl DeviceIdentity {
    #[must_use]
    pub fn from_seed(mut seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        seed.zeroize();
        Self { signing_key }
    }

    /// Signs only a parsed closed possession request.
    pub fn prove(
        &self,
        request: &PossessionRequest,
    ) -> Result<PossessionResponse, PossessionError> {
        let claims = PossessionClaims::from_request(
            request,
            base64_url(self.signing_key.verifying_key().to_bytes()),
        );
        let protected = base64_url(
            crate::codec::canonical_json(&serde_json::json!({
                "alg": "Ed25519",
                "typ": "bwg-worker-possession+jws",
            }))?
            .as_bytes(),
        );
        let payload = base64_url(crate::codec::canonical_json(&claims)?.as_bytes());
        let signing_input = format!("{protected}.{payload}");
        let signature = self.signing_key.sign(signing_input.as_bytes());
        Ok(PossessionResponse::new(
            request.request_id().to_owned(),
            claims,
            format!("{signing_input}.{}", base64_url(signature.to_bytes())),
        ))
    }
}

impl fmt::Debug for DeviceIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DeviceIdentity([redacted])")
    }
}
