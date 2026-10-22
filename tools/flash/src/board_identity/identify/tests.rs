use super::*;

/// An `nvs_tool --dump minimal --format json` document, followed by a diagnostic line.
fn dump(entries: &[(&str, &str, &str)]) -> Vec<u8> {
    let entries: Vec<_> = entries
        .iter()
        .map(|(namespace, key, value)| {
            serde_json::json!({"namespace": namespace, "key": key, "encoding": "string", "data": value,
                "state": "Written", "is_empty": null})
        })
        .collect();
    let mut bytes = serde_json::to_vec(&entries).expect("json");
    bytes.extend_from_slice(b"\nFound unused namespace\n");
    bytes
}

fn stock_601() -> Vec<u8> {
    dump(&[
        ("main", "wifissid", "home-network"),
        ("main", "wifipass", "secret"),
        ("main", "boardversion", "601"),
        ("main", "asicmodel", "BM1370"),
        ("main", "devicemodel", "gamma"),
    ])
}

fn fields(version: &str, asic: &str, model: &str) -> BoardFields {
    BoardFields {
        boardversion: Some(version.to_owned()),
        asicmodel: Some(asic.to_owned()),
        devicemodel: Some(model.to_owned()),
    }
}

#[test]
fn a_stock_601_s_records_are_found_despite_a_diagnostic() {
    // Arrange / Act
    let found = board_fields(&stock_601()).expect("fields");
    // Assert
    assert_eq!(found, expected_fields(RegisteredBoard::Gamma601));
}

#[test]
fn only_the_main_namespace_counts() {
    // Arrange
    let other = dump(&[("other", "boardversion", "601")]);
    // Act
    let found = board_fields(&other).expect("fields");
    // Assert
    assert_eq!(found.boardversion, None);
}

#[test]
fn no_other_key_survives_the_extraction() {
    // Arrange / Act
    let found = format!("{:?}", board_fields(&stock_601()).expect("fields"));
    // Assert
    assert!(
        !found.contains("secret") && !found.contains("home-network"),
        "{found}"
    );
}

#[test]
fn a_dump_that_is_not_json_is_refused() {
    // Arrange / Act
    let error = board_fields(b"not json").expect_err("invalid");
    // Assert
    assert!(error.to_string().contains("nvs_dump_invalid"), "{error}");
}

#[test]
fn missing_records_stop_as_missing() {
    // Arrange
    let found = BoardFields {
        boardversion: Some("601".to_owned()),
        ..BoardFields::default()
    };
    // Act
    let error = decide(
        RegisteredBoard::Gamma601,
        &found,
        Admission::Unregistered,
        true,
    )
    .expect_err("missing");
    // Assert
    assert!(
        error.to_string().contains("reason=records_missing"),
        "{error}"
    );
}

#[test]
fn a_205_s_records_stop_identification_as_a_601() {
    // Arrange
    let found = fields("205", "BM1366", "ultra");
    // Act
    let error = decide(
        RegisteredBoard::Gamma601,
        &found,
        Admission::Unregistered,
        true,
    )
    .expect_err("other board");
    // Assert
    assert!(
        error
            .to_string()
            .contains("reason=records_name_another_board"),
        "{error}"
    );
}

#[test]
fn a_match_without_register_registers_nothing() {
    // Arrange
    let found = fields("601", "BM1370", "gamma");
    // Act
    let outcome = decide(
        RegisteredBoard::Gamma601,
        &found,
        Admission::Unregistered,
        false,
    )
    .expect("decided");
    // Assert
    assert_eq!(outcome, "not_requested");
}

#[test]
fn a_matching_unregistered_board_is_registered() {
    // Arrange
    let found = fields("601", "BM1370", "gamma");
    // Act
    let outcome = decide(
        RegisteredBoard::Gamma601,
        &found,
        Admission::Unregistered,
        true,
    )
    .expect("decided");
    // Assert
    assert_eq!(outcome, "register");
}

#[test]
fn a_matching_registered_board_is_left_as_is() {
    // Arrange
    let found = fields("601", "BM1370", "gamma");
    // Act
    let outcome =
        decide(RegisteredBoard::Gamma601, &found, Admission::Admitted, true).expect("decided");
    // Assert
    assert_eq!(outcome, "already_registered");
}

#[test]
fn the_contract_enables_identification_in_the_active_block() {
    // Arrange
    let tasks =
        format!("## Active\n\n### {TASK} | 2026-10-21 | Bring up\n\n{ENABLE_LINE}\n\n## Future\n");
    // Act / Assert
    assert!(contract_enabled(&tasks));
}

#[test]
fn the_contract_under_future_does_not_enable_identification() {
    // Arrange
    let tasks =
        format!("## Active\n\n## Future\n\n### {TASK} | 2026-10-21 | Bring up\n\n{ENABLE_LINE}\n");
    // Act / Assert
    assert!(!contract_enabled(&tasks));
}

#[test]
fn an_enable_line_in_another_task_does_not_count() {
    // Arrange
    let tasks = format!("## Active\n\n### task-other | 2026-10-21 | Other\n\n{ENABLE_LINE}\n");
    // Act / Assert
    assert!(!contract_enabled(&tasks));
}

#[test]
fn the_flash_size_comes_from_board_info() {
    // Arrange
    let output = b"Chip type:         esp32s3 (revision v0.2)\nFlash size:        16MB\n";
    // Act / Assert
    assert_eq!(flash_size(output), "16MB");
}

#[test]
fn board_info_without_a_flash_size_reports_missing() {
    // Arrange / Act / Assert
    assert_eq!(flash_size(b"Chip type: esp32s3\n"), "missing");
}

#[test]
fn unknown_field_values_are_reported_as_a_category() {
    // Arrange
    let custom = Some("my-board".to_owned());
    // Act / Assert
    assert_eq!(field_label(custom.as_ref()), "other");
}

#[test]
fn identification_writes_nothing_and_stores_no_image() {
    // Arrange
    let source = include_str!("../identify.rs");
    let forbidden = [
        "execute_esptool_write_flash",
        "execute_core_dump_erase",
        "execute_with_output",
        "environment.execute(",
        "execute_flash_read(&",
    ];
    // Act
    let used: Vec<_> = forbidden
        .iter()
        .filter(|effect| source.contains(**effect))
        .collect();
    // Assert: one NVS read into a pipe, hard-resetting back to the installed application.
    assert!(used.is_empty(), "{used:?}");
    assert!(source.contains("execute_flash_read_to_pipe(&read)"));
    assert!(source.contains("FlashReadDisposition::ReturnToApplication"));
}

#[test]
fn the_registry_is_validated_before_the_reset_into_rom() {
    // Arrange
    let source = include_str!("../identify.rs");
    let body = &source[source.find("pub(crate) fn run_identify").expect("run")..];
    // Act
    let registry = body.find("load_registry(").expect("registry");
    let session = body.find("begin_usb_session(").expect("session");
    // Assert
    assert!(registry < session);
}

/// Real `nvs_tool.py --integrity-check --dump minimal --format json` output (ESP-IDF v5.5.4),
/// read through a FIFO, for an image `nvs_partition_gen.py` built from a synthetic CSV holding
/// the stock 601 board fields and one non-secret setting.
const REAL_NVS_TOOL_DUMP: &str = r#"[
    {
        "namespace": "main",
        "key": "boardversion",
        "encoding": "string",
        "data": "601",
        "state": "Written",
        "is_empty": false
    },
    {
        "namespace": "main",
        "key": "asicmodel",
        "encoding": "string",
        "data": "BM1370",
        "state": "Written",
        "is_empty": false
    },
    {
        "namespace": "main",
        "key": "devicemodel",
        "encoding": "string",
        "data": "gamma",
        "state": "Written",
        "is_empty": false
    },
    {
        "namespace": "main",
        "key": "rotation",
        "encoding": "uint16_t",
        "data": 0,
        "state": "Written",
        "is_empty": false
    }
]

Page no. 0 	CRC32: OK
Page Empty
Page Empty
Page Empty
Page Empty
Page Empty

"#;

#[test]
fn real_nvs_tool_output_yields_the_601_fields() {
    // Arrange / Act
    let found = board_fields(REAL_NVS_TOOL_DUMP.as_bytes()).expect("fields");
    // Assert
    assert_eq!(found, expected_fields(RegisteredBoard::Gamma601));
}
