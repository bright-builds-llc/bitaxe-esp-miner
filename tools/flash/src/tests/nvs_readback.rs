use super::*;

#[test]
fn nvs_readback_owns_one_exact_read_only_range() {
    let output = Utf8Path::new("scratch/nvs.private.bin");
    let args = nvs_read_flash_args("admitted", output);
    assert!(nvs_read_args_are_exact(&args, output));
    assert_eq!(
        args[8..],
        ["read_flash", "0x9000", "0x6000", output.as_str()]
    );
    for forbidden in ["write_flash", "erase_flash", "erase_region"] {
        assert!(!args.iter().any(|argument| argument == forbidden));
    }
}

fn entry(namespace: &str, key: &str, encoding: &str, data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "namespace": namespace,
        "key": key,
        "encoding": encoding,
        "data": data,
        "state": "Written",
        "is_empty": false
    })
}

fn parse_entries(entries: Vec<serde_json::Value>) -> Vec<NvsSemanticEntry> {
    parse_nvs_entries(&serde_json::to_vec(&entries).expect("serialize synthetic NVS entries"))
        .expect("parse synthetic NVS entries")
}

#[test]
fn nvs_comparison_accepts_exact_expected_subset_with_runtime_extras() {
    // Arrange
    let expected = parse_entries(vec![
        entry("main", "mineonboot", "u16", serde_json::json!(0)),
        entry("main", "wifissid", "string", serde_json::json!("fixture")),
    ]);
    let installed = parse_entries(vec![
        entry("main", "mineonboot", "u16", serde_json::json!(0)),
        entry("main", "wifissid", "string", serde_json::json!("fixture")),
        entry("main", "runtimeonly", "u64", serde_json::json!(42)),
    ]);

    // Act
    let comparison = compare_expected_nvs(&installed, &expected);

    // Assert
    assert_eq!(
        comparison,
        NvsComparison {
            namespace_match: true,
            key_set_match: true,
            encoding_match: true,
            value_digest_match: true,
            state_match: true,
            nvs_match: true,
        }
    );
}

#[test]
fn nvs_comparison_rejects_missing_or_duplicate_expected_keys() {
    // Arrange
    let expected = parse_entries(vec![entry(
        "main",
        "mineonboot",
        "u16",
        serde_json::json!(0),
    )]);
    let missing = Vec::new();
    let duplicate = parse_entries(vec![
        entry("main", "mineonboot", "u16", serde_json::json!(0)),
        entry("main", "mineonboot", "u16", serde_json::json!(0)),
    ]);

    // Act / Assert
    assert!(!compare_expected_nvs(&missing, &expected).nvs_match);
    assert!(!compare_expected_nvs(&duplicate, &expected).nvs_match);
}

#[test]
fn nvs_comparison_classifies_namespace_encoding_value_and_state_drift() {
    // Arrange
    let expected = parse_entries(vec![entry(
        "main",
        "mineonboot",
        "u16",
        serde_json::json!(0),
    )]);
    let wrong_namespace = parse_entries(vec![entry(
        "other",
        "mineonboot",
        "u16",
        serde_json::json!(0),
    )]);
    let wrong_encoding = parse_entries(vec![entry(
        "main",
        "mineonboot",
        "i32",
        serde_json::json!(0),
    )]);
    let wrong_value = parse_entries(vec![entry(
        "main",
        "mineonboot",
        "u16",
        serde_json::json!(1),
    )]);
    let mut wrong_state_json = entry("main", "mineonboot", "u16", serde_json::json!(0));
    wrong_state_json["state"] = serde_json::json!("Erased");
    let wrong_state = parse_entries(vec![wrong_state_json]);

    // Act / Assert
    assert!(!compare_expected_nvs(&wrong_namespace, &expected).key_set_match);
    assert!(!compare_expected_nvs(&wrong_encoding, &expected).encoding_match);
    assert!(!compare_expected_nvs(&wrong_value, &expected).value_digest_match);
    assert!(!compare_expected_nvs(&wrong_state, &expected).state_match);
}

#[test]
fn nvs_json_parser_rejects_corrupt_and_oversized_inputs() {
    // Arrange
    let corrupt = b"not-json";
    let oversized = vec![b' '; MAX_NVS_JSON_BYTES + 1];

    // Act / Assert
    assert!(parse_nvs_entries(corrupt).is_err());
    assert!(parse_nvs_entries(&oversized).is_err());
}

#[test]
fn nvs_json_parser_accepts_clean_integrity_output_and_rejects_failed_integrity() {
    // Arrange
    let json = serde_json::to_string(&vec![entry(
        "main",
        "mineonboot",
        "u16",
        serde_json::json!(0),
    )])
    .expect("serialize NVS fixture");
    let clean = format!("{json}\n\nPage no. 0 CRC32: OK\n");
    let failed = format!("{json}\n\nEntry mineonboot has wrong CRC32!\n");

    // Act / Assert
    assert_eq!(
        parse_nvs_entries(clean.as_bytes())
            .expect("clean output")
            .len(),
        1
    );
    assert!(parse_nvs_entries(failed.as_bytes()).is_err());
}

#[test]
fn nvs_readback_cli_requires_the_sealed_stage_one_inputs() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "nvs-readback",
        "--board",
        "205",
        "--port",
        "/dev/cu.usbmodem1101",
        "--wifi-credentials",
        "wifi-credentials.json",
        "--private-root",
        NVS_READ_ROOT,
        "--plan",
        NVS_FIRST_PLAN,
        "--redact-evidence",
    ];

    // Act
    let cli = parse_cli(args).expect("NVS readback CLI");

    // Assert
    let CliCommand::NvsReadback(command) = cli.command else {
        panic!("expected NVS readback command");
    };
    assert_eq!(command.board, BoardId::Ultra205);
    assert_eq!(
        command.wifi_credentials,
        Utf8Path::new("wifi-credentials.json")
    );
    assert_eq!(command.private_root, Utf8Path::new(NVS_READ_ROOT));
    assert_eq!(command.plan, Utf8Path::new(NVS_FIRST_PLAN));
    assert!(command.redact_evidence);
    assert!(!command.admission_only);
}

#[test]
fn nvs_readback_cli_exposes_a_no_effect_admission_checkpoint() {
    // Arrange
    let args = [
        "bitaxe-flash",
        "nvs-readback",
        "--port",
        "/dev/cu.usbmodem1101",
        "--wifi-credentials",
        "wifi-credentials.json",
        "--private-root",
        NVS_READ_ROOT,
        "--plan",
        NVS_FIRST_PLAN,
        "--redact-evidence",
        "--admission-only",
    ];

    // Act
    let cli = parse_cli(args).expect("NVS admission-only CLI");

    // Assert
    let CliCommand::NvsReadback(command) = cli.command else {
        panic!("expected NVS readback command");
    };
    assert!(command.admission_only);
}
