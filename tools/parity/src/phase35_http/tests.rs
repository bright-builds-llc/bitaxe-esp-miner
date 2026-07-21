use super::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn ready_metrics() -> Value {
    json!({
        "scheme_category": "http",
        "curl_exit_code": 0,
        "tcp_connect_millis": 5,
        "tls_handshake_millis": 0,
        "request_bytes": 128,
        "response_status": 200,
        "response_header_count": 4,
        "response_header_bytes": 192,
        "response_body_bytes": 27,
        "total_millis": 12,
        "first_byte_millis": 9,
        "tls_verification": "not_applicable"
    })
}

fn classify(
    metrics: Value,
    body: &[u8],
) -> Result<ClassifiedPhase35Http, Phase35HttpDiagnosticError> {
    classify_phase35_http(
        &serde_json::to_vec(&metrics).expect("metrics fixture should encode"),
        body,
    )
}

fn assert_category(mut metrics: Value, body: &[u8], expected: HttpTerminalCategory) {
    metrics["response_body_bytes"] = json!(body.len());

    // Act
    let result = classify(metrics, body).expect("observation should be structurally valid");

    // Assert
    assert_eq!(result.terminal_category, expected);
    assert_eq!(result.projection.terminal_category, expected);
}

#[test]
fn classifies_the_exact_ordered_terminal_matrix() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;
    let mut cases = Vec::new();

    let mut tcp = ready_metrics();
    tcp["tcp_connect_millis"] = json!(0);
    tcp["request_bytes"] = json!(0);
    tcp["response_status"] = json!(0);
    tcp["response_header_count"] = json!(0);
    tcp["response_header_bytes"] = json!(0);
    tcp["response_body_bytes"] = json!(0);
    tcp["first_byte_millis"] = json!(0);
    cases.push((tcp, Vec::new(), HttpTerminalCategory::TcpConnectionFailure));

    let mut tls = ready_metrics();
    tls["scheme_category"] = json!("https");
    tls["tls_verification"] = json!("failed");
    tls["request_bytes"] = json!(0);
    tls["response_status"] = json!(0);
    tls["response_header_count"] = json!(0);
    tls["response_header_bytes"] = json!(0);
    tls["response_body_bytes"] = json!(0);
    tls["first_byte_millis"] = json!(0);
    cases.push((tls, Vec::new(), HttpTerminalCategory::TlsHandshakeFailure));

    let mut request = ready_metrics();
    request["request_bytes"] = json!(0);
    request["response_status"] = json!(0);
    request["response_header_count"] = json!(0);
    request["response_header_bytes"] = json!(0);
    request["response_body_bytes"] = json!(0);
    request["first_byte_millis"] = json!(0);
    cases.push((
        request,
        Vec::new(),
        HttpTerminalCategory::RequestTransmissionIncomplete,
    ));

    let mut status = ready_metrics();
    status["response_status"] = json!(0);
    status["response_header_count"] = json!(0);
    status["response_header_bytes"] = json!(0);
    status["response_body_bytes"] = json!(0);
    status["first_byte_millis"] = json!(0);
    cases.push((
        status,
        Vec::new(),
        HttpTerminalCategory::ResponseStatusMissing,
    ));

    let mut headers = ready_metrics();
    headers["response_header_count"] = json!(0);
    headers["response_header_bytes"] = json!(0);
    headers["response_body_bytes"] = json!(0);
    cases.push((
        headers,
        Vec::new(),
        HttpTerminalCategory::ResponseHeadersMissing,
    ));

    let mut non_success = ready_metrics();
    non_success["response_status"] = json!(503);
    cases.push((
        non_success,
        body.to_vec(),
        HttpTerminalCategory::NonSuccessResponseStatus,
    ));

    let mut missing_body = ready_metrics();
    missing_body["response_body_bytes"] = json!(0);
    cases.push((
        missing_body,
        Vec::new(),
        HttpTerminalCategory::ResponseBodyMissing,
    ));

    let mut incomplete = ready_metrics();
    incomplete["curl_exit_code"] = json!(18);
    cases.push((
        incomplete,
        body.to_vec(),
        HttpTerminalCategory::ResponseBodyIncompleteOrOverLimit,
    ));

    cases.push((
        ready_metrics(),
        b"{".to_vec(),
        HttpTerminalCategory::InvalidJson,
    ));
    cases.push((
        ready_metrics(),
        br#"{"hostname":42}"#.to_vec(),
        HttpTerminalCategory::InvalidHostnameSchema,
    ));
    cases.push((ready_metrics(), body.to_vec(), HttpTerminalCategory::Ready));

    // Act / Assert
    for (mut metrics, case_body, expected) in cases {
        metrics["response_body_bytes"] = json!(case_body.len());
        let result = classify(metrics, &case_body).expect("matrix observation should be valid");
        assert_eq!(result.terminal_category, expected);
    }
}

#[test]
fn earliest_boundary_wins_when_later_facts_also_fail() {
    // Arrange
    let mut metrics = ready_metrics();
    metrics["response_status"] = json!(503);
    let body = b"{";

    // Act / Assert
    assert_category(
        metrics,
        body,
        HttpTerminalCategory::NonSuccessResponseStatus,
    );
}

#[test]
fn accepts_bounded_timeout_observation_overshoot() {
    // Arrange
    let mut metrics = ready_metrics();
    metrics["curl_exit_code"] = json!(28);
    metrics["request_bytes"] = json!(0);
    metrics["response_status"] = json!(0);
    metrics["response_header_count"] = json!(0);
    metrics["response_header_bytes"] = json!(0);
    metrics["response_body_bytes"] = json!(0);
    metrics["total_millis"] = json!(10_003);
    metrics["first_byte_millis"] = json!(0);

    // Act / Assert
    assert_category(
        metrics,
        b"",
        HttpTerminalCategory::RequestTransmissionIncomplete,
    );
}

#[test]
fn tls_is_not_applicable_to_http_and_required_for_https() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;
    let http = ready_metrics();
    let mut https = ready_metrics();
    https["scheme_category"] = json!("https");
    https["tls_handshake_millis"] = json!(7);
    https["tls_verification"] = json!("verified");

    // Act
    let http_result = classify(http, body).expect("HTTP observation should classify");
    let https_result = classify(https, body).expect("HTTPS observation should classify");

    // Assert
    assert!(!http_result.projection.tls_applicable);
    assert!(https_result.projection.tls_applicable);
    assert!(https_result.projection.tls_established);
    assert!(https_result.projection.tls_verified);
}

#[test]
fn status_classes_are_closed_and_redacted() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;
    let expected = [
        (101, "informational"),
        (204, "success"),
        (302, "redirection"),
        (404, "client_error"),
        (503, "server_error"),
    ];

    // Act / Assert
    for (status, class) in expected {
        let mut metrics = ready_metrics();
        metrics["response_status"] = json!(status);
        let result = classify(metrics, body).expect("status should classify");
        let projection = serde_json::to_value(result.projection).expect("projection should encode");
        assert_eq!(projection["response_status_class"], class);
    }
}

#[test]
fn rejects_every_missing_metrics_key() {
    // Arrange
    let metrics = ready_metrics();
    let keys = metrics
        .as_object()
        .expect("fixture should be an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    // Act / Assert
    for key in keys {
        let mut candidate = metrics.clone();
        candidate
            .as_object_mut()
            .expect("fixture should be an object")
            .remove(&key);
        assert!(
            classify(candidate, b"").is_err(),
            "missing key accepted: {key}"
        );
    }
}

#[test]
fn rejects_unknown_and_malformed_metrics_fields() {
    // Arrange
    let mut extra = ready_metrics();
    extra["origin"] = json!("forbidden");
    let mut malformed = ready_metrics();
    malformed["total_millis"] = json!("12");

    // Act
    let extra_result = classify(extra, b"");
    let malformed_result = classify(malformed, b"");

    // Assert
    assert!(extra_result.is_err());
    assert!(malformed_result.is_err());
}

#[test]
fn rejects_inconsistent_body_size_and_impossible_tls_state() {
    // Arrange
    let mut wrong_size = ready_metrics();
    wrong_size["response_body_bytes"] = json!(999);
    let mut impossible_tls = ready_metrics();
    impossible_tls["tls_verification"] = json!("verified");

    // Act
    let wrong_size_result = classify(wrong_size, br#"{"hostname":"fixture-host"}"#);
    let impossible_tls_result = classify(impossible_tls, br#"{"hostname":"fixture-host"}"#);

    // Assert
    assert!(wrong_size_result.is_err());
    assert!(impossible_tls_result.is_err());
}

#[test]
fn rejects_out_of_bound_counts_durations_and_body() {
    // Arrange
    let mut curl_exit = ready_metrics();
    curl_exit["curl_exit_code"] = json!(256);
    let mut tcp = ready_metrics();
    tcp["tcp_connect_millis"] = json!(5_001);
    let mut total = ready_metrics();
    total["total_millis"] = json!(11_001);
    let mut headers = ready_metrics();
    headers["response_header_count"] = json!(1_025);
    let over_limit_body = vec![b'x'; 65_537];
    let mut over_limit = ready_metrics();
    over_limit["response_body_bytes"] = json!(over_limit_body.len());

    // Act / Assert
    for candidate in [curl_exit, tcp, total, headers] {
        assert!(classify(candidate, b"").is_err());
    }
    assert!(classify(over_limit, &over_limit_body).is_err());
}

#[test]
fn rejects_timing_and_boundary_inconsistencies() {
    // Arrange
    let mut first_byte_after_total = ready_metrics();
    first_byte_after_total["first_byte_millis"] = json!(13);
    let mut tcp_after_total = ready_metrics();
    tcp_after_total["tcp_connect_millis"] = json!(13);
    let mut missing_tcp_with_response = ready_metrics();
    missing_tcp_with_response["tcp_connect_millis"] = json!(0);

    // Act / Assert
    for candidate in [
        first_byte_after_total,
        tcp_after_total,
        missing_tcp_with_response,
    ] {
        assert!(classify(candidate, br#"{"hostname":"fixture-host"}"#).is_err());
    }
}

#[test]
fn projection_has_the_exact_allowlisted_fields() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;

    // Act
    let result = classify(ready_metrics(), body).expect("ready observation should classify");
    let projection = serde_json::to_value(result.projection).expect("projection should encode");
    let keys = projection
        .as_object()
        .expect("projection should be an object")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    // Assert
    let expected = BTreeSet::from([
        "schema_version",
        "tcp_connected",
        "tls_applicable",
        "tls_established",
        "tls_verified",
        "request_transmission_complete",
        "response_status_received",
        "response_headers_received",
        "response_body_received",
        "response_body_complete",
        "json_parsed",
        "hostname_schema_valid",
        "curl_exit_code",
        "request_bytes",
        "response_header_count",
        "response_header_bytes",
        "response_body_bytes",
        "tcp_connect_millis",
        "tls_handshake_millis",
        "first_byte_millis",
        "total_millis",
        "response_status_class",
        "terminal_category",
    ]);
    assert_eq!(keys, expected);
    assert_eq!(projection["schema_version"], PHASE35_HTTP_SCHEMA);
}

#[test]
fn projection_forbids_raw_and_identifying_fields() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;
    let forbidden_fields = [
        "origin",
        "host",
        "ip",
        "port",
        "headers",
        "body",
        "hostname",
        "curl_error",
        "credentials",
        "device_identifier",
        "digest",
    ];
    let raw_canaries = [
        "raw-origin-canary",
        "raw-host-canary",
        "raw-body-canary",
        "raw-credential-canary",
        "raw-device-canary",
    ];

    // Act
    let result = classify(ready_metrics(), body).expect("ready observation should classify");
    let projection = serde_json::to_value(&result.projection).expect("projection should encode");
    let encoded = serde_json::to_string(&projection).expect("projection should encode");

    // Assert
    for field in forbidden_fields {
        assert!(
            projection.get(field).is_none(),
            "forbidden field present: {field}"
        );
    }
    for canary in raw_canaries {
        assert!(
            !encoded.contains(canary),
            "projection leaked canary: {canary}"
        );
    }
}

#[test]
fn ready_hostname_remains_private_from_the_projection() {
    // Arrange
    let body = br#"{"hostname":"fixture-host"}"#;

    // Act
    let result = classify(ready_metrics(), body).expect("ready observation should classify");
    let encoded = serde_json::to_string(&result.projection).expect("projection should encode");

    // Assert
    assert_eq!(result.maybe_hostname.as_deref(), Some("fixture-host"));
    assert!(!encoded.contains("fixture-host"));
}
