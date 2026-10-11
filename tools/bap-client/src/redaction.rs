//! What the client may print. Credentials, network identity and pool identity never
//! leave the process; every other value is device telemetry.

/// Response and setting tokens whose values are sensitive.
pub const SENSITIVE_TOKENS: [&str; 8] = [
    "ssid",
    "password",
    "wifi_ssid",
    "wifi_password",
    "wifi_ip",
    "pool",
    "poolPort",
    "poolUser",
];

#[must_use]
pub fn is_sensitive(token: &str) -> bool {
    SENSITIVE_TOKENS.contains(&token)
}

/// The printable form of a value: sensitive values become their length only.
#[must_use]
pub fn printable(token: &str, value: &str) -> String {
    if is_sensitive(token) {
        format!("redacted:{}_bytes", value.len())
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_and_identity_values_are_redacted() {
        for token in SENSITIVE_TOKENS {
            assert_eq!(printable(token, "secret"), "redacted:6_bytes", "{token}");
        }
    }

    #[test]
    fn telemetry_values_are_printed() {
        assert_eq!(printable("hashrate", "512.3"), "512.3");
    }
}
