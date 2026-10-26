//! The SELF-001 records the single Serial/JTAG writer forwards (ADR-0021 amendment, ADR-0041
//! item 14): the self-test's stages, checkpoint, metrics, terminal and receipt, the PSRAM fact,
//! and the connected origin its campaign reads to back up and restore settings. Each must match
//! its closed shape; no other text passes.

/// The longest self-test record forwarded; the metrics line is the longest, about 330 bytes.
const MAXIMUM_LINE_BYTES: usize = 512;

/// Whether `line` is one of the self-test or origin records, in its exact closed shape.
pub(super) fn is_self_test_line(line: &str) -> bool {
    if line.len() > MAXIMUM_LINE_BYTES {
        return false;
    }
    // The campaign's PSRAM fact (upstream's first self-test check).
    if matches!(line, "psram_status=available" | "psram_status=unavailable") {
        return true;
    }
    if let Some(rest) = line.strip_prefix("self_test_receipt ") {
        return valid_receipt(rest);
    }
    if let Some(rest) = line.strip_prefix("runtime_origin ") {
        return valid_origin(rest);
    }
    if let Some(rest) = line.strip_prefix("self_test_authority=granted ") {
        return valid_authority(rest);
    }
    JSON_RECORDS.iter().any(|(name, schema, keys)| {
        line.strip_prefix(name).is_some_and(|body| {
            body.starts_with(schema)
                && body.ends_with('}')
                && closed_json_body(body)
                && keys_within(body, keys)
        })
    })
}

/// Each JSON record's name, schema prefix and the only field names it may carry.
type JsonRecord = (&'static str, &'static str, &'static [&'static str]);
const JSON_RECORDS: [JsonRecord; 5] = [
    (
        "self_test_stage=",
        r#"{"schema":"self-test-runtime-v1","#,
        &["schema", "stage", "case", "lease_present"],
    ),
    (
        "self_test_checkpoint=",
        r#"{"schema":"self-test-runtime-v1","#,
        &[
            "schema",
            "checkpoint",
            "safe_state",
            "response_required",
            "failure",
        ],
    ),
    (
        "self_test_terminal=",
        r#"{"schema":"self-test-runtime-v1","#,
        &["schema", "outcome", "safe_stop", "restart_delay_ms"],
    ),
    (
        "self_test_metrics=",
        r#"{"schema":"self-test-metrics-v1","#,
        &[
            "schema",
            "measured_ms",
            "total_ghs",
            "domain_ghs",
            "domain_samples",
            "domain_rejected",
            "input_volts",
            "core_mv",
            "power_watts",
            "fan_rpm",
            "maximum_temperature_c",
        ],
    ),
    (
        "self_test_regulator_metrics=",
        r#"{"schema":"self-test-regulator-metrics-v1","#,
        &[
            "schema",
            "kind",
            "maximum_vr_temperature_c",
            "regulator_fault",
        ],
    ),
];

/// Every field name in `body` is one of `keys`, so a field added later must be reviewed and
/// listed here before it can reach USB. The closed character set has no escapes, so the text
/// between quotes alternates outside/inside, and an inside string followed by `:` is a key.
fn keys_within(body: &str, keys: &[&str]) -> bool {
    let parts: Vec<&str> = body.split('"').collect();
    (1..parts.len()).step_by(2).all(|index| {
        let is_key = parts
            .get(index + 1)
            .is_some_and(|next| next.starts_with(':'));
        !is_key || keys.contains(&parts[index])
    })
}

/// Only lowercase tokens, digits, signs and JSON punctuation: no spaces, no free text, no
/// address or path characters such as `/`, `@` or `:` outside the JSON key separator.
fn closed_json_body(body: &str) -> bool {
    body.bytes().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(
                byte,
                b'_' | b'-' | b'.' | b'"' | b':' | b',' | b'{' | b'}' | b'[' | b']'
            )
    })
}

/// `outcome=passed|cancelled lease=<16 lowercase hex>`.
fn valid_receipt(rest: &str) -> bool {
    let mut fields = rest.split(' ');
    matches!(fields.next(), Some("outcome=passed" | "outcome=cancelled"))
        && fields
            .next()
            .and_then(|field| field.strip_prefix("lease="))
            .is_some_and(|lease| lowercase_hex(lease, 16))
        && fields.next().is_none()
}

/// `generation=<n> bound_ms=<n>`.
fn valid_authority(rest: &str) -> bool {
    let mut fields = rest.split(' ');
    fields
        .next()
        .and_then(|field| field.strip_prefix("generation="))
        .is_some_and(|value| super::canonical_unsigned(value, u64::from(u32::MAX)))
        && fields
            .next()
            .and_then(|field| field.strip_prefix("bound_ms="))
            .is_some_and(|value| super::canonical_unsigned(value, u64::MAX))
        && fields.next().is_none()
}

/// `session=<32 hex> boot_ordinal=<n> device_url=http://<dotted IPv4> redacted=true`.
fn valid_origin(rest: &str) -> bool {
    let mut fields = rest.split(' ');
    fields
        .next()
        .and_then(|field| field.strip_prefix("session="))
        .is_some_and(|session| lowercase_hex(session, 32))
        && fields
            .next()
            .and_then(|field| field.strip_prefix("boot_ordinal="))
            .is_some_and(|value| super::canonical_unsigned(value, u64::MAX))
        && fields
            .next()
            .and_then(|field| field.strip_prefix("device_url=http://"))
            .is_some_and(dotted_ipv4)
        && fields.next() == Some("redacted=true")
        && fields.next().is_none()
}

fn lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn dotted_ipv4(value: &str) -> bool {
    let octets: Vec<&str> = value.split('.').collect();
    octets.len() == 4
        && octets
            .iter()
            .all(|octet| super::canonical_unsigned(octet, 255))
}

#[cfg(test)]
mod tests {
    use super::is_self_test_line;

    #[test]
    fn the_runtime_stage_checkpoint_and_terminal_records_pass() {
        // Arrange
        let lines = [
            r#"self_test_stage={"schema":"self-test-runtime-v1","stage":"measuring","case":"pass","lease_present":true}"#,
            r#"self_test_checkpoint={"schema":"self-test-runtime-v1","checkpoint":"cancel_ready","safe_state":true,"response_required":false,"failure":"planned_evaluation_failure"}"#,
            r#"self_test_terminal={"schema":"self-test-runtime-v1","outcome":"passed","safe_stop":true,"restart_delay_ms":10000}"#,
        ];
        // Act
        let passed = lines.map(is_self_test_line);
        // Assert
        assert_eq!(passed, [true; 3]);
    }

    #[test]
    fn the_metrics_records_pass() {
        // Arrange
        let lines = [
            r#"self_test_metrics={"schema":"self-test-metrics-v1","measured_ms":30012,"total_ghs":1043.512,"domain_ghs":[260.1,259.8,262.0,261.6],"domain_samples":[29,29,29,29],"domain_rejected":[0,0,0,0],"input_volts":5.141,"core_mv":1149,"power_watts":20.315,"fan_rpm":7952,"maximum_temperature_c":64.25}"#,
            r#"self_test_regulator_metrics={"schema":"self-test-regulator-metrics-v1","kind":"gamma601_full_v1","maximum_vr_temperature_c":71.50,"regulator_fault":false}"#,
        ];
        // Act
        let passed = lines.map(is_self_test_line);
        // Assert
        assert_eq!(passed, [true; 2]);
    }

    #[test]
    fn the_receipt_authority_and_origin_records_pass() {
        // Arrange
        let lines = [
            "psram_status=available",
            "self_test_receipt outcome=cancelled lease=0123456789abcdef",
            "self_test_authority=granted generation=4096 bound_ms=240000",
            "runtime_origin session=0123456789abcdef0123456789abcdef boot_ordinal=3 device_url=http://192.168.1.20 redacted=true",
        ];
        // Act
        let passed = lines.map(is_self_test_line);
        // Assert
        assert_eq!(passed, [true; 4]);
    }

    #[test]
    fn free_text_and_other_shapes_never_pass() {
        // Arrange
        let lines = [
            r#"self_test_stage={"schema":"self-test-runtime-v1","stage":"measuring","note":"Hello World"}"#,
            r#"self_test_stage={"schema":"other-v1","stage":"measuring"}"#,
            r#"self_test_metrics={"schema":"self-test-metrics-v1","pool":"stratum+tcp://pool"}"#,
            "self_test_receipt outcome=passed lease=XYZ",
            "self_test_receipt outcome=passed lease=0123456789abcdef extra",
            "runtime_origin session=0123456789abcdef0123456789abcdef boot_ordinal=3 device_url=http://bitaxe.local redacted=true",
            "runtime_origin session=0123456789abcdef0123456789abcdef boot_ordinal=3 device_url=http://192.168.1.300 redacted=true",
            "self_test_owner=started board=601 kind=gamma601_full_v1",
            "psram_status=available extra",
            r#"self_test_stage={"schema":"self-test-runtime-v1","stage":"measuring","worker":"bc1qowner"}"#,
        ];
        // Act
        let passed = lines.map(is_self_test_line);
        // Assert
        assert_eq!(passed, [false; 10]);
    }

    #[test]
    fn an_oversized_record_never_passes() {
        // Arrange
        let line = format!(
            r#"self_test_metrics={{"schema":"self-test-metrics-v1","pad":"{}"}}"#,
            "a".repeat(600)
        );
        // Act
        let passed = is_self_test_line(&line);
        // Assert
        assert!(!passed);
    }
}
