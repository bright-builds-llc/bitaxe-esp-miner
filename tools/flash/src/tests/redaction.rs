use super::*;

#[test]
fn evidence_sanitizer_developer_raw_preserves_network_fields_and_redacts_secrets() {
    // Arrange
    let text = r#"{"ssid":"lab-net","wifiPass":"super-secret","ipv4":"192.168.1.24","mac":"aa:bb:cc:dd:ee:ff","device_url":"http://192.168.1.24","token":"api-secret"}"#;

    // Act
    let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::DeveloperRaw);

    // Assert
    assert!(sanitized.contains(r#""ssid":"lab-net""#));
    assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
    assert!(sanitized.contains(r#""ipv4":"192.168.1.24""#));
    assert!(sanitized.contains(r#""mac":"aa:bb:cc:dd:ee:ff""#));
    assert!(sanitized.contains(r#""device_url":"http://192.168.1.24""#));
    assert!(sanitized.contains(r#""token":"[redacted]""#));
    assert!(!sanitized.contains("super-secret"));
    assert!(!sanitized.contains("api-secret"));
}

#[test]
fn evidence_sanitizer_redacts_numeric_never_persist_json_scalars() {
    // Arrange
    let text = r#"{"poolPort":3333,"poolUser":"owner.worker","wifiPass":"super-secret"}"#;

    // Act
    let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::DeveloperRaw);

    // Assert
    assert!(sanitized.contains(r#""poolPort":"[redacted]""#));
    assert!(sanitized.contains(r#""poolUser":"[redacted]""#));
    assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
    assert!(!sanitized.contains("3333"));
    assert!(!sanitized.contains("owner.worker"));
    assert!(!sanitized.contains("super-secret"));
}

#[test]
fn evidence_sanitizer_commit_redacted_redacts_json_wifi_fields_network_urls_ips_and_macs() {
    // Arrange
    let text = concat!(
        r#"{"ssid":"lab-net","wifiPass":"super-secret","ipv4":"192.168.1.24","#,
        r#""mac":"aa:bb:cc:dd:ee:ff","device_url":"http://192.168.1.24","#,
        r#""hostname":"miner.local","poolUser":"owner.worker"}"#,
        "\npath=/Users/operator/private.log port=/dev/cu.usbmodem101 pid=123 pgid=456\n",
        "GET /api/system/info HTTP/1.1\nHost: miner.local\n",
    );

    // Act
    let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::CommitRedacted);

    // Assert
    assert!(sanitized.contains(r#""ssid":"[redacted]""#));
    assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
    assert!(sanitized.contains(r#""ipv4":"[redacted-ip]""#));
    assert!(sanitized.contains(r#""mac":"[redacted-mac]""#));
    assert!(sanitized.contains(r#""device_url":"[redacted-url]""#));
    assert!(!sanitized.contains("lab-net"));
    assert!(!sanitized.contains("super-secret"));
    assert!(!sanitized.contains("192.168.1.24"));
    assert!(!sanitized.contains("aa:bb:cc:dd:ee:ff"));
    assert!(!sanitized.contains("http://192.168.1.24"));
    assert!(!sanitized.contains("miner.local"));
    assert!(!sanitized.contains("owner.worker"));
    assert!(!sanitized.contains("/Users/operator"));
    assert!(!sanitized.contains("/dev/cu.usbmodem101"));
    assert!(!sanitized.contains("pid=123"));
    assert!(!sanitized.contains("pgid=456"));
    assert!(!sanitized.contains("HTTP/1.1"));
    assert!(sanitized.contains("[redacted-path]"));
    assert!(sanitized.contains("[redacted-http]"));
}
