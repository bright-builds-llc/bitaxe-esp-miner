fn common_args() -> CommonArgs {
    CommonArgs {
        board: BoardId::Ultra205,
        port: Some("/dev/cu.usbmodem101".to_owned()),
        dry_run: true,
        redact_evidence: false,
        evidence_mode: None,
        evidence_dir: None,
    }
}

fn trusted_monitor_log() -> String {
    [
        "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        "ota_boot_validation=not_pending state=factory",
        "spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518",
        "axeos_api_route_shell=started registered_routes=15",
        "reset_reason=11",
        "firmware_commit=0123456789ab",
        "reference_commit=abcdef012345",
        "esp_idf_version=v5.5.4",
    ]
    .join("\n")
}

fn runtime_attestation_log() -> String {
    [10_000_u64, 20_000]
        .into_iter()
        .map(|uptime_ms| {
            format!(
                "runtime_boot_attestation schema_version=1 \
                 session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=7 \
                 reset_reason=other uptime_ms={uptime_ms} board=205 asic=BM1366 \
                 mining=disabled work_submission=disabled hardware_control=disabled \
                 firmware_commit={SOURCE_COMMIT} reference_commit={REFERENCE_COMMIT} \
                 app_elf_sha256={APP_ELF_SHA256} esp_idf_version=v5.5.4 \
                 ota_boot_validation=complete spiffs_mount=available \
                 api_route_shell=started redacted=true"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn flash_monitor_fixture(dir: &TempDir, evidence_dir: Utf8PathBuf) -> FlashMonitorCommand {
    let manifest = write_manifest_v3(dir, DEFAULT_ELF_NAME);
    FlashMonitorCommand {
        common: CommonArgs {
            evidence_dir: Some(evidence_dir),
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
        network_reconnect_probe: false,
        thermal_fault_stimulus_intent: None,
        self_test_intent: None,
        capture_timeout_seconds: DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS,
    }
}

fn dir_path(dir: &TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
}

fn write_wifi_credentials(dir: &TempDir, ssid: &str, wifi_pass: &str) -> Utf8PathBuf {
    let path = dir_path(dir).join("wifi.json");
    std::fs::write(
        path.as_std_path(),
        serde_json::json!({
            "ssid": ssid,
            "wifiPass": wifi_pass,
        })
        .to_string(),
    )
    .expect("write wifi credentials");
    path
}

fn write_manifest(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
    let dir_path = dir_path(dir);
    write_manifest_at(
        &dir_path,
        PACKAGE_MANIFEST_RELATIVE_PATH,
        default_flash_image,
    )
}

fn write_manifest_at(
    workspace_dir: &Utf8Path,
    manifest_relative_path: &str,
    default_flash_image: &str,
) -> Utf8PathBuf {
    let manifest = workspace_dir.join(manifest_relative_path);
    let manifest_dir = manifest.parent().expect("parent");
    std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
    write_manifest_v3_contents(&manifest, default_flash_image, FACTORY_IMAGE_NAME);
    manifest
}

fn write_manifest_v3(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
    write_manifest_v3_with_factory_artifact(dir, default_flash_image, FACTORY_IMAGE_NAME)
}

fn write_manifest_v3_with_factory_artifact(
    dir: &TempDir,
    default_flash_image: &str,
    factory_artifact_path: &str,
) -> Utf8PathBuf {
    let dir_path = dir_path(dir);
    let manifest = dir_path.join(PACKAGE_MANIFEST_RELATIVE_PATH);
    write_manifest_v3_contents(&manifest, default_flash_image, factory_artifact_path);
    manifest
}

fn write_manifest_v3_contents(
    manifest: &Utf8Path,
    default_flash_image: &str,
    factory_artifact_path: &str,
) {
    let manifest_dir = manifest.parent().expect("parent");
    std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
    let elf = b"synthetic firmware elf".to_vec();
    let ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
    let partition_table = factory_partition_table_fixture();
    let factory = factory_image_fixture(&partition_table, &ota);
    let www = b"synthetic www".to_vec();
    let otadata = b"synthetic otadata".to_vec();
    let artifacts = [
        ("firmware_elf", DEFAULT_ELF_NAME, elf.as_slice()),
        ("firmware_ota_image", "esp-miner.bin", ota.as_slice()),
        (
            "factory_merged_image",
            factory_artifact_path,
            factory.as_slice(),
        ),
        ("www_spiffs_image", "www.bin", www.as_slice()),
        (
            "partition_table",
            "partition-table.bin",
            partition_table.as_slice(),
        ),
        ("otadata_initial", "otadata-initial.bin", otadata.as_slice()),
    ];
    let mut artifact_values = Vec::new();
    for (kind, path, bytes) in artifacts {
        std::fs::write(manifest_dir.join(path).as_std_path(), bytes).expect("write artifact");
        artifact_values.push(serde_json::json!({
            "kind": kind,
            "path": path,
            "offset": "Unavailable",
            "sha256": sha256_bytes(bytes),
        }));
    }
    let value = serde_json::json!({
        "schema_version": 3,
        "release_name": "bitaxe-ultra205",
        "semantic_version": "0.1.0",
        "source_commit": SOURCE_COMMIT,
        "reference_commit": REFERENCE_COMMIT,
        "app_elf_sha256": APP_ELF_SHA256,
        "build_identity": {
            "label": BUILD_LABEL,
            "channel": "dev",
            "source_dirty": false,
            "release_tag": null
        },
        "default_flash_image": default_flash_image,
        "artifacts": artifact_values,
    });
    std::fs::write(
        manifest.as_std_path(),
        serde_json::to_string_pretty(&value).expect("manifest json"),
    )
    .expect("write manifest");
}

fn rewrite_manifest_provenance(manifest: &Utf8Path, provenance: &BuildProvenance) {
    let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
    let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
    let identity = provenance.build_identity();
    value["semantic_version"] = serde_json::json!(provenance.semantic_version());
    value["source_commit"] = serde_json::json!(identity.source_commit());
    value["reference_commit"] = serde_json::json!(provenance.reference_commit());
    value["build_identity"] = serde_json::json!({
        "label": identity.build_label(),
        "channel": identity.build_channel().as_str(),
        "source_dirty": identity.source_dirty(),
        "release_tag": identity.maybe_release_tag(),
    });

    let ota = esp_application_fixture(identity.source_commit(), identity.build_label());
    let ota_path = manifest
        .parent()
        .expect("manifest parent")
        .join("esp-miner.bin");
    std::fs::write(ota_path.as_std_path(), &ota).expect("rewrite ota");
    let partition_table = factory_partition_table_fixture();
    let factory = factory_image_fixture(&partition_table, &ota);
    let factory_path = manifest
        .parent()
        .expect("manifest parent")
        .join(FACTORY_IMAGE_NAME);
    std::fs::write(factory_path.as_std_path(), &factory).expect("rewrite factory");
    let artifacts = value["artifacts"].as_array_mut().expect("artifacts array");
    let ota_artifact = artifacts
        .iter_mut()
        .find(|artifact| artifact["kind"] == "firmware_ota_image")
        .expect("ota artifact");
    ota_artifact["sha256"] = serde_json::json!(sha256_bytes(&ota));
    let factory_artifact = artifacts
        .iter_mut()
        .find(|artifact| artifact["kind"] == "factory_merged_image")
        .expect("factory artifact");
    factory_artifact["sha256"] = serde_json::json!(sha256_bytes(&factory));

    std::fs::write(
        manifest.as_std_path(),
        serde_json::to_string_pretty(&value).expect("manifest json"),
    )
    .expect("rewrite manifest");
}

fn duplicate_manifest_artifact(manifest: &Utf8Path, kind: &str) {
    let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
    let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
    let artifacts = value["artifacts"].as_array_mut().expect("artifacts array");
    let duplicate = artifacts
        .iter()
        .find(|artifact| artifact["kind"] == kind)
        .expect("artifact kind")
        .clone();
    artifacts.push(duplicate);
    std::fs::write(
        manifest.as_std_path(),
        serde_json::to_string_pretty(&value).expect("manifest json"),
    )
    .expect("rewrite manifest");
}

fn add_manifest_artifact(
    manifest: &Utf8Path,
    kind: &str,
    relative_path: &str,
    bytes: &[u8],
) -> Utf8PathBuf {
    let manifest_dir = manifest.parent().expect("manifest parent");
    let path = manifest_dir.join(relative_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent.as_std_path()).expect("create artifact parent");
    }
    std::fs::write(path.as_std_path(), bytes).expect("write extra artifact");
    let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
    let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
    value["artifacts"]
        .as_array_mut()
        .expect("artifacts array")
        .push(serde_json::json!({
            "kind": kind,
            "path": relative_path,
            "offset": "Unavailable",
            "sha256": sha256_bytes(bytes),
        }));
    std::fs::write(
        manifest.as_std_path(),
        serde_json::to_string_pretty(&value).expect("manifest json"),
    )
    .expect("rewrite manifest");
    path
}

fn run_explicit_image_admission(
    manifest: &Utf8Path,
    image: Utf8PathBuf,
) -> Result<FlashOutcome> {
    let command = FlashCommand {
        common: CommonArgs {
            port: None,
            dry_run: false,
            ..common_args()
        },
        image: Some(image),
        manifest: Some(manifest.to_owned()),
        wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    );
    run_flash(&command, &environment)
}

fn rewrite_manifest_artifact_digest(manifest: &Utf8Path, kind: &str, bytes: &[u8]) {
    let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
    let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
    let artifact = value["artifacts"]
        .as_array_mut()
        .expect("artifacts array")
        .iter_mut()
        .find(|artifact| artifact["kind"] == kind)
        .expect("artifact kind");
    artifact["sha256"] = serde_json::json!(sha256_bytes(bytes));
    std::fs::write(
        manifest.as_std_path(),
        serde_json::to_string_pretty(&value).expect("manifest json"),
    )
    .expect("rewrite manifest");
}

fn rewrite_manifest_application(manifest: &Utf8Path, ota: &[u8]) {
    let manifest_dir = manifest.parent().expect("manifest parent");
    let ota_path = manifest_dir.join("esp-miner.bin");
    std::fs::write(ota_path.as_std_path(), ota).expect("rewrite OTA image");
    let partition_table = factory_partition_table_fixture();
    let factory = factory_image_fixture(&partition_table, ota);
    let factory_path = manifest_dir.join(FACTORY_IMAGE_NAME);
    std::fs::write(factory_path.as_std_path(), &factory).expect("rewrite factory image");
    rewrite_manifest_artifact_digest(manifest, "firmware_ota_image", ota);
    rewrite_manifest_artifact_digest(manifest, "factory_merged_image", &factory);
}

fn rewrite_manifest_elf_artifact_only(manifest: &Utf8Path, elf: &[u8]) {
    let elf_path = manifest
        .parent()
        .expect("manifest parent")
        .join(DEFAULT_ELF_NAME);
    std::fs::write(elf_path.as_std_path(), elf).expect("rewrite firmware ELF");
    rewrite_manifest_artifact_digest(manifest, "firmware_elf", elf);
}

#[derive(Clone, Copy)]
enum LayoutFixtureKind {
    DescriptorNotDrom,
    DestinationOverlap,
    AliasOverlap,
}

fn assert_parsed_layout_rejected_before_effects(
    fixture_kind: LayoutFixtureKind,
    expected_reason: &str,
    dry_run: bool,
) {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let ota = layout_fixture(fixture_kind);
    rewrite_manifest_application(&manifest, &ota);
    let mut args = vec![
        "bitaxe-flash".to_owned(),
        "flash".to_owned(),
        "--board".to_owned(),
        "205".to_owned(),
        "--manifest".to_owned(),
        manifest.to_string(),
    ];
    let environment = if dry_run {
        args.extend([
            "--port".to_owned(),
            "/dev/null".to_owned(),
            "--dry-run".to_owned(),
        ]);
        FakeFlashEnvironment::default()
    } else {
        args.extend([
            "--wifi-credentials".to_owned(),
            "/missing/credentials.json".to_owned(),
        ]);
        FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        )
    };
    let cli = parse_cli(args).expect("parsed flash command");
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };

    // Act
    let error = run_flash(&command, &environment)
        .expect_err("layout admission")
        .to_string();

    // Assert
    assert_eq!(
        error,
        format!("identity_admission=blocked reason={expected_reason}")
    );
    assert_eq!(environment.list_ports_calls(), 0);
    assert!(!environment
        .read_string_paths()
        .iter()
        .any(|path| path.as_str().contains("credentials")));
    assert!(environment.generated_nvs_partitions().is_empty());
    assert!(environment.created_snapshot_paths().is_empty());
    assert!(environment.captured_commands().is_empty());
    assert!(environment.executed_commands().is_empty());
    assert!(environment.observed_flashes().is_empty());
}

fn layout_fixture(fixture_kind: LayoutFixtureKind) -> Vec<u8> {
    let mut image = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
    match fixture_kind {
        LayoutFixtureKind::DescriptorNotDrom => {
            image[24..28].copy_from_slice(&0x3fc8_8000_u32.to_le_bytes());
        }
        LayoutFixtureKind::DestinationOverlap => {
            append_esp_segment(&mut image, 0x4037_4000, &[0; 4]);
        }
        LayoutFixtureKind::AliasOverlap => {
            image[4..8].copy_from_slice(&0x4037_8000_u32.to_le_bytes());
            let executable_header = second_esp_segment_header(&image);
            image[executable_header..executable_header + 4]
                .copy_from_slice(&0x4037_8000_u32.to_le_bytes());
            append_esp_segment(&mut image, 0x3fc8_8000, &[0; 4]);
        }
    }
    reseal_esp_application(&mut image);
    image
}

fn append_esp_segment(image: &mut Vec<u8>, load_address: u32, payload: &[u8]) {
    let data_end = esp_segment_data_end(image);
    image.truncate(data_end);
    image[1] = image[1].checked_add(1).expect("fixture segment count");
    image.extend_from_slice(&load_address.to_le_bytes());
    image.extend_from_slice(
        &u32::try_from(payload.len())
            .expect("fixture payload length")
            .to_le_bytes(),
    );
    image.extend_from_slice(payload);
}

fn esp_segment_data_end(image: &[u8]) -> usize {
    const IMAGE_HEADER_LEN: usize = 24;
    const SEGMENT_HEADER_LEN: usize = 8;

    let mut cursor = IMAGE_HEADER_LEN;
    for _ in 0..usize::from(image[1]) {
        let payload_len = usize::try_from(u32::from_le_bytes(
            image[cursor + 4..cursor + 8]
                .try_into()
                .expect("fixture segment length"),
        ))
        .expect("fixture payload length");
        cursor += SEGMENT_HEADER_LEN + payload_len;
    }
    cursor
}

fn second_esp_segment_header(image: &[u8]) -> usize {
    const IMAGE_HEADER_LEN: usize = 24;
    const SEGMENT_HEADER_LEN: usize = 8;

    let first_payload_len = usize::try_from(u32::from_le_bytes(
        image[IMAGE_HEADER_LEN + 4..IMAGE_HEADER_LEN + 8]
            .try_into()
            .expect("fixture segment length"),
    ))
    .expect("fixture payload length");
    IMAGE_HEADER_LEN + SEGMENT_HEADER_LEN + first_payload_len
}

fn esp_application_fixture(source_commit: &str, build_label: &str) -> Vec<u8> {
    const IMAGE_HEADER_LEN: usize = 24;
    const APP_DESCRIPTOR_LEN: usize = 256;
    const VERSION_OFFSET: usize = 16;
    const VERSION_LEN: usize = 32;
    const ELF_SHA_OFFSET: usize = 144;
    const MMU_PAGE_SIZE_OFFSET: usize = 180;

    let mut descriptor = vec![0_u8; APP_DESCRIPTOR_LEN];
    descriptor[..4].copy_from_slice(&0xABCD_5432_u32.to_le_bytes());
    descriptor[VERSION_OFFSET..VERSION_OFFSET + build_label.len()]
        .copy_from_slice(build_label.as_bytes());
    descriptor[ELF_SHA_OFFSET..ELF_SHA_OFFSET + 32]
        .copy_from_slice(&decode_lower_hex(APP_ELF_SHA256).expect("valid app hash"));
    descriptor[MMU_PAGE_SIZE_OFFSET] = 16;
    assert!(build_label.len() < VERSION_LEN);

    let mut payload = descriptor;
    payload.extend_from_slice(source_commit.as_bytes());
    let mut image = vec![0_u8; IMAGE_HEADER_LEN];
    image[0] = 0xe9;
    image[1] = 2;
    image[2] = 2;
    image[3] = 0x4f;
    image[4..8].copy_from_slice(&0x4037_4000_u32.to_le_bytes());
    image[8] = 0xee;
    image[12..14].copy_from_slice(&9_u16.to_le_bytes());
    image[15..17].copy_from_slice(&0_u16.to_le_bytes());
    image[17..19].copy_from_slice(&99_u16.to_le_bytes());
    image[23] = 1;
    image.extend_from_slice(&0x3c00_0020_u32.to_le_bytes());
    image.extend_from_slice(
        &u32::try_from(payload.len())
            .expect("fixture payload length")
            .to_le_bytes(),
    );
    image.extend_from_slice(&payload);
    image.extend_from_slice(&0x4037_4000_u32.to_le_bytes());
    image.extend_from_slice(&4_u32.to_le_bytes());
    image.extend_from_slice(&[0x13, 0, 0, 0]);
    reseal_esp_application(&mut image);
    image
}

fn reseal_esp_application(image: &mut Vec<u8>) {
    const IMAGE_HEADER_LEN: usize = 24;
    const SEGMENT_HEADER_LEN: usize = 8;

    let mut cursor = IMAGE_HEADER_LEN;
    let mut checksum = 0xef_u8;
    for _ in 0..usize::from(image[1]) {
        let payload_start = cursor + SEGMENT_HEADER_LEN;
        let payload_len = usize::try_from(u32::from_le_bytes([
            image[cursor + 4],
            image[cursor + 5],
            image[cursor + 6],
            image[cursor + 7],
        ]))
        .expect("fixture payload length");
        let payload_end = payload_start + payload_len;
        checksum = image[payload_start..payload_end]
            .iter()
            .fold(checksum, |value, byte| value ^ byte);
        cursor = payload_end;
    }
    let padding_len = (15 - (cursor % 16)) % 16;
    image.truncate(cursor);
    image.resize(cursor + padding_len, 0);
    image.push(checksum);
    let digest = Sha256::digest(&*image);
    image.extend_from_slice(&digest);
}

fn factory_partition_table_fixture() -> Vec<u8> {
    let mut record = [0_u8; 32];
    record[..2].copy_from_slice(&[0xaa, 0x50]);
    record[2] = 0x00;
    record[3] = 0x00;
    record[4..8].copy_from_slice(&0x10000_u32.to_le_bytes());
    record[8..12].copy_from_slice(&0x400000_u32.to_le_bytes());
    record[12..19].copy_from_slice(b"factory");
    let mut table = record.to_vec();
    table.extend_from_slice(&[0xff; 32]);
    table
}

fn factory_image_fixture(partition_table: &[u8], ota: &[u8]) -> Vec<u8> {
    const PARTITION_TABLE_OFFSET: usize = 0x8000;
    const FACTORY_APP_OFFSET: usize = 0x10000;

    let mut factory = vec![0xff; FACTORY_APP_OFFSET + ota.len()];
    factory[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + partition_table.len()]
        .copy_from_slice(partition_table);
    factory[FACTORY_APP_OFFSET..FACTORY_APP_OFFSET + ota.len()].copy_from_slice(ota);
    factory
}
