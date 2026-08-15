use bitaxe_http_transport::StrictHttpClient;

use super::is_lower_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RuntimeOriginObservation {
    pub(super) boot_session: String,
    pub(super) origin: String,
}

pub(super) fn parse_runtime_origin_observation(bytes: &[u8]) -> Option<RuntimeOriginObservation> {
    let document = std::str::from_utf8(bytes).ok()?;
    let mut maybe_observation: Option<RuntimeOriginObservation> = None;
    for line in document.lines() {
        let Some((_, fields)) = line.split_once("runtime_origin ") else {
            continue;
        };
        let mut maybe_session = None;
        let mut maybe_origin = None;
        let mut redacted = false;
        for field in fields.split_ascii_whitespace() {
            if let Some(value) = field.strip_prefix("session=") {
                maybe_session = Some(value);
            } else if let Some(value) = field.strip_prefix("device_url=") {
                maybe_origin = Some(value);
            } else if field == "redacted=true" {
                redacted = true;
            }
        }
        let (Some(session), Some(origin)) = (maybe_session, maybe_origin) else {
            return None;
        };
        if !redacted || !is_lower_hex(session, 32) || StrictHttpClient::new(origin).is_err() {
            return None;
        }
        let observation = RuntimeOriginObservation {
            boot_session: session.to_owned(),
            origin: origin.trim_end_matches('/').to_owned(),
        };
        if maybe_observation
            .as_ref()
            .is_some_and(|existing| existing != &observation)
        {
            return None;
        }
        maybe_observation = Some(observation);
    }
    maybe_observation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_duplicate_current_markers() {
        // Arrange
        let marker = "runtime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 device_url=http://private-device redacted=true";
        let document = format!("prefix\n{marker}\nI (123) {marker}\n");

        // Act
        let observation = parse_runtime_origin_observation(document.as_bytes())
            .expect("one duplicated observation must be admitted");

        // Assert
        assert_eq!(observation.boot_session, "a".repeat(32));
        assert_eq!(observation.origin, "http://private-device");
    }

    #[test]
    fn rejects_conflicting_sessions_or_origins() {
        // Arrange
        let document = b"runtime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 device_url=http://first redacted=true\nruntime_origin session=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb boot_ordinal=8 device_url=http://second redacted=true\n";

        // Act
        let observation = parse_runtime_origin_observation(document);

        // Assert
        assert!(observation.is_none());
    }
}
