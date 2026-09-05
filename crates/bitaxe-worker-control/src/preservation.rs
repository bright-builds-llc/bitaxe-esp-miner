//! Private continuity evidence, separate from mining qualification or authority.
use serde::{Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StateFingerprint([u8; 32]);

impl StateFingerprint {
    /// Hashes explicitly nonsecret state; never pass credentials or private key material.
    #[must_use]
    pub fn of_public_state(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
    #[must_use]
    pub const fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }
}
impl Serialize for StateFingerprint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut text = String::with_capacity(64);
        const HEX: &[u8] = b"0123456789abcdef";
        for byte in self.0 {
            text.push(char::from(HEX[(byte >> 4) as usize]));
            text.push(char::from(HEX[(byte & 15) as usize]));
        }
        serializer.serialize_str(&text)
    }
}
impl fmt::Debug for StateFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StateFingerprint([private])")
    }
}

#[derive(Debug, Serialize)]
pub struct SettingsPreservation {
    pub(crate) fingerprint: StateFingerprint,
    pub(crate) mine_on_boot: bool,
}
impl SettingsPreservation {
    #[must_use]
    pub const fn new(fingerprint: StateFingerprint, mine_on_boot: bool) -> Self {
        Self {
            fingerprint,
            mine_on_boot,
        }
    }
}
