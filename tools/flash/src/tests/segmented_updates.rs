use super::*;

fn update_command(dir: &TempDir) -> FlashCommand {
    FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(write_manifest_v4(dir, DEFAULT_ELF_NAME)),
        wifi_credentials: None,
    }
}

#[test]
fn ordinary_update_never_writes_nvs_or_unrelated_data_partitions() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = update_command(&dir);
    let environment = FakeFlashEnvironment::default();
    // Act
    let outcome = run_flash(&command, &environment).expect("state-preserving update");
    // Assert
    assert!(outcome.nvs_seed.is_none());
    let observed = environment.observed_flashes();
    assert_eq!(
        observed
            .iter()
            .map(|segment| segment.offset)
            .collect::<Vec<_>>(),
        [0, 0x8000, 0x10000, 0x410000, 0xf10000]
    );
    for segment in observed.iter() {
        let erase_end = segment.offset + ((segment.bytes.len() as u32 + 0xfff) & !0xfff);
        assert!(erase_end <= 0x9000 || segment.offset >= 0xf000);
        assert!(erase_end <= 0x710000 || segment.offset == 0xf10000);
    }
    assert!(environment.generated_nvs_partitions().is_empty());
    assert_eq!(*environment.application_exit_write_counts.borrow(), [1]);
}

#[test]
fn failed_segment_stops_before_later_segments_and_application_exit() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = update_command(&dir);
    let environment = FakeFlashEnvironment::default().with_execute_failure_offset("0x10000");
    // Act
    let result = run_flash(&command, &environment);
    // Assert
    assert!(result.is_err());
    assert_eq!(
        environment
            .observed_flashes()
            .iter()
            .map(|segment| segment.offset)
            .collect::<Vec<_>>(),
        [0, 0x8000]
    );
    assert!(environment
        .application_exit_write_counts
        .borrow()
        .is_empty());
    assert!(environment
        .created_snapshot_paths()
        .iter()
        .all(|path| !path.exists()));
}

#[test]
fn rehashed_segment_must_still_match_the_admitted_factory_container() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = update_command(&dir);
    let manifest = command.manifest.as_ref().expect("manifest");
    let replacement = b"different www";
    std::fs::write(
        manifest.parent().expect("parent").join("www.bin"),
        replacement,
    )
    .expect("replace web artifact");
    rewrite_manifest_artifact_digest(manifest, "www_spiffs_image", replacement);
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&command, &environment).expect_err("independent artifact drift");
    // Assert
    assert!(error
        .to_string()
        .contains("update_factory_artifact_mismatch"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn mutually_rehashed_partition_artifacts_cannot_change_persistent_layout() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = update_command(&dir);
    let manifest = command.manifest.as_ref().expect("manifest");
    let root = manifest.parent().expect("parent");
    let table = esp_idf_part::PartitionTable::try_from_str(
        CANONICAL_PARTITIONS.replace("0x9000, 0x6000", "0x9000, 0x5000"),
    )
    .expect("altered valid layout")
    .to_bin()
    .expect("binary table");
    let mut factory = std::fs::read(root.join(FACTORY_IMAGE_NAME)).expect("factory");
    factory[0x8000..0x8000 + table.len()].copy_from_slice(&table);
    std::fs::write(root.join("partition-table.bin"), &table).expect("replace table");
    std::fs::write(root.join(FACTORY_IMAGE_NAME), &factory).expect("replace factory");
    rewrite_manifest_artifact_digest(manifest, "partition_table_binary", &table);
    rewrite_manifest_artifact_digest(manifest, "factory_merged_image", &factory);
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&command, &environment).expect_err("NVS geometry must stay canonical");
    // Assert
    assert!(error
        .to_string()
        .contains("update_partition_layout_mismatch"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn historical_v3_packages_are_not_ordinary_update_authority() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let command = update_command(&dir);
    let manifest = command.manifest.as_ref().expect("manifest");
    let mut document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest).expect("manifest"))
            .expect("manifest JSON");
    document["schema_version"] = 3.into();
    std::fs::write(manifest, document.to_string()).expect("historical manifest");
    let environment = FakeFlashEnvironment::default();
    // Act
    let error =
        run_flash(&command, &environment).expect_err("V3 must not reset identity implicitly");
    // Assert
    assert!(error
        .to_string()
        .contains("manifest_update_segments_required"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn ordinary_wifi_seed_is_rejected_before_credential_access() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let mut command = update_command(&dir);
    command.wifi_credentials = Some(Utf8PathBuf::from("/missing/private-wifi.json"));
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&command, &environment).expect_err("explicit factory reset required");
    // Assert
    assert!(error
        .to_string()
        .contains("ordinary_update_preserves_nvs_use_explicit_factory_reset"));
    assert!(!environment
        .read_string_paths()
        .iter()
        .any(|path| path.as_str().contains("private-wifi")));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn factory_reset_is_an_explicit_cli_choice() {
    // Arrange / Act
    let CliCommand::Flash(ordinary) =
        parse_cli(["bitaxe-flash", "flash", "--manifest", "package.json"])
            .expect("ordinary CLI")
            .command
    else {
        panic!("flash variant");
    };
    let CliCommand::Flash(factory) = parse_cli([
        "bitaxe-flash",
        "flash",
        "--manifest",
        "package.json",
        "--factory-reset",
    ])
    .expect("explicit factory CLI")
    .command
    else {
        panic!("flash variant");
    };
    // Assert
    assert!(!ordinary.factory_reset);
    assert!(factory.factory_reset);
}
