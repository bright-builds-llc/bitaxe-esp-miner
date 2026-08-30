use sha2::{Digest, Sha256};

use super::*;

impl StrictHttpClient {
    pub fn get_theme(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until("GET", "/api/theme", deadline, DEFAULT_CONNECT_TIMEOUT)
    }

    pub fn patch_system_settings_once(
        &self,
        body: &[u8],
        deadline: Instant,
    ) -> Result<ExchangeObservation> {
        if body.len() > MAX_BODY_BYTES {
            bail!("settings request exceeds the strict body bound");
        }
        self.exchange_until_with_body(
            "PATCH",
            "/api/system",
            body,
            "application/json",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }

    pub fn post_theme_once(&self, body: &[u8], deadline: Instant) -> Result<ExchangeObservation> {
        if body.len() > 1_023 {
            bail!("theme request exceeds the strict body bound");
        }
        self.exchange_until_with_body(
            "POST",
            "/api/theme",
            body,
            "application/json",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }
}

pub fn strict_http_evaluator_sha256() -> String {
    let mut digest = Sha256::new();
    digest.update(include_str!("lib.rs").as_bytes());
    digest.update(include_str!("observation.rs").as_bytes());
    digest.update(include_str!("recovery.rs").as_bytes());
    let output = digest.finalize();
    let mut encoded = String::with_capacity(output.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in output {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
