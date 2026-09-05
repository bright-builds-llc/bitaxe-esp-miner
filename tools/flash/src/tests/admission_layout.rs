use super::*;

#[test]
fn identity_admission_rejects_digest_rewritten_factory_app_tamper_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let factory_path = manifest
        .parent()
        .expect("manifest parent")
        .join(FACTORY_IMAGE_NAME);
    let mut factory = std::fs::read(factory_path.as_std_path()).expect("factory image");
    let tamper_offset = 0x10000 + 40;
    factory[tamper_offset] ^= 0x01;
    std::fs::write(factory_path.as_std_path(), &factory).expect("tampered factory image");
    rewrite_manifest_artifact_digest(&manifest, "factory_merged_image", &factory);
    let command = FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            port: None,
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    );

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = result.expect_err("factory application tamper").to_string();
    assert!(error.contains("identity_admission=blocked reason=ota_segment_checksum_mismatch"));
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn executable_admission_rejects_zero_load_address_in_parsed_dry_run_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let mut ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
    ota[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_esp_application(&mut ota);
    rewrite_manifest_application(&manifest, &ota);
    let cli = parse_cli([
        "bitaxe-flash".to_owned(),
        "flash".to_owned(),
        "--dry-run".to_owned(),
        "--port".to_owned(),
        "/dev/null".to_owned(),
        "--manifest".to_owned(),
        manifest.to_string(),
    ])
    .expect("parsed dry-run command");
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = result.expect_err("zero load address").to_string();
    assert!(error.contains("ota_segment_load_address_unsupported"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.created_snapshot_paths().is_empty());
}

#[test]
fn executable_admission_rejects_mapped_mismatch_in_parsed_non_dry_run_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let mut ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
    ota[24..28].copy_from_slice(&0x3c00_0024_u32.to_le_bytes());
    reseal_esp_application(&mut ota);
    rewrite_manifest_application(&manifest, &ota);
    let cli = parse_cli([
        "bitaxe-flash".to_owned(),
        "flash".to_owned(),
        "--manifest".to_owned(),
        manifest.to_string(),
        "--wifi-credentials".to_owned(),
        "/missing/credentials.json".to_owned(),
    ])
    .expect("parsed non-dry command");
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    );

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = result.expect_err("mapped mismatch").to_string();
    assert!(error.contains("ota_mapped_segment_misaligned"), "{error}");
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.created_snapshot_paths().is_empty());
}

#[test]
fn identity_admission_rejects_all_layout_classes_in_parsed_dry_run_before_effects() {
    for (fixture_kind, reason) in [
        (
            LayoutFixtureKind::DescriptorNotDrom,
            "app_descriptor_segment_not_drom",
        ),
        (
            LayoutFixtureKind::DestinationOverlap,
            "ota_segment_destination_overlap",
        ),
        (LayoutFixtureKind::AliasOverlap, "ota_segment_alias_overlap"),
    ] {
        assert_parsed_layout_rejected_before_effects(fixture_kind, reason, true);
    }
}

#[test]
fn identity_admission_rejects_all_layout_classes_in_parsed_non_dry_run_before_effects() {
    for (fixture_kind, reason) in [
        (
            LayoutFixtureKind::DescriptorNotDrom,
            "app_descriptor_segment_not_drom",
        ),
        (
            LayoutFixtureKind::DestinationOverlap,
            "ota_segment_destination_overlap",
        ),
        (LayoutFixtureKind::AliasOverlap, "ota_segment_alias_overlap"),
    ] {
        assert_parsed_layout_rejected_before_effects(fixture_kind, reason, false);
    }
}

#[test]
fn firmware_elf_app_sha_rejects_changed_elf_in_parsed_dry_run_before_later_reads() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    rewrite_manifest_elf_artifact_only(&manifest, b"changed firmware elf");
    let cli = parse_cli([
        "bitaxe-flash".to_owned(),
        "flash".to_owned(),
        "--dry-run".to_owned(),
        "--port".to_owned(),
        "/dev/null".to_owned(),
        "--manifest".to_owned(),
        manifest.to_string(),
    ])
    .expect("parsed dry-run command");
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    let ota_path = manifest
        .parent()
        .expect("manifest parent")
        .join("esp-miner.bin");
    std::fs::remove_file(ota_path.as_std_path()).expect("remove later OTA artifact");
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = result.expect_err("ELF relationship mismatch").to_string();
    assert!(error.contains("firmware_elf_app_sha_mismatch"));
    assert!(!error.contains("failed to read fake artifact"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.created_snapshot_paths().is_empty());
}

#[test]
fn firmware_elf_app_sha_rejects_changed_elf_in_parsed_non_dry_run_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    rewrite_manifest_elf_artifact_only(&manifest, b"changed firmware elf");
    let cli = parse_cli([
        "bitaxe-flash".to_owned(),
        "flash".to_owned(),
        "--manifest".to_owned(),
        manifest.to_string(),
        "--wifi-credentials".to_owned(),
        "/missing/credentials.json".to_owned(),
    ])
    .expect("parsed non-dry command");
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    );

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = result.expect_err("ELF relationship mismatch").to_string();
    assert!(error.contains("firmware_elf_app_sha_mismatch"));
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
    assert!(environment.executed_commands().is_empty());
    assert!(environment.created_snapshot_paths().is_empty());
}

#[test]
fn identity_admission_rejects_explicit_manifest_elf_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let image = manifest
        .parent()
        .expect("manifest parent")
        .join(DEFAULT_ELF_NAME);

    // Act
    let error = run_explicit_image_admission(&manifest, image)
        .expect_err("manifest ELF must not enter full-flash execution");

    // Assert
    assert!(format!("{error:#}")
        .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
}

#[test]
fn identity_admission_rejects_explicit_extra_artifact_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let image = add_manifest_artifact(&manifest, "extra", "extra.bin", b"extra image");

    // Act
    let error = run_explicit_image_admission(&manifest, image)
        .expect_err("extra artifact must not enter full-flash execution");

    // Assert
    assert!(format!("{error:#}")
        .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
}

#[test]
fn identity_admission_rejects_explicit_factory_path_alias_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let manifest_dir = manifest.parent().expect("manifest parent");
    let factory = manifest_dir.join(FACTORY_IMAGE_NAME);
    let factory_bytes = std::fs::read(factory.as_std_path()).expect("factory image");
    let alias = add_manifest_artifact(
        &manifest,
        "factory_alias",
        "factory-alias.bin",
        &factory_bytes,
    );

    // Act
    let error = run_explicit_image_admission(&manifest, alias)
        .expect_err("factory path alias must not enter full-flash execution");

    // Assert
    assert!(format!("{error:#}")
        .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
}

#[test]
fn identity_admission_rejects_explicit_factory_named_extra_before_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let image = add_manifest_artifact(
        &manifest,
        "factory_named_extra",
        "nested/bitaxe-ultra205-factory.bin",
        b"factory-named extra",
    );

    // Act
    let error = run_explicit_image_admission(&manifest, image)
        .expect_err("factory-like basename must not enter full-flash execution");

    // Assert
    assert!(format!("{error:#}")
        .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
}

#[test]
fn admitted_execution_uses_original_bytes_after_package_replacement() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let factory_path = manifest
        .parent()
        .expect("manifest parent")
        .join(FACTORY_IMAGE_NAME);
    let admitted_bytes = std::fs::read(factory_path.as_std_path()).expect("factory image");
    let command = FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default()
        .with_source_replacement(factory_path.clone(), b"replaced package bytes".to_vec());

    // Act
    run_flash(&command, &environment).expect("admitted flash");

    // Assert
    let observed = environment.observed_flashes();
    assert_eq!(observed.len(), 5);
    for segment in observed.iter() {
        assert_ne!(segment.path, factory_path);
        let start = segment.offset as usize;
        assert_eq!(
            segment.bytes,
            admitted_bytes[start..start + segment.bytes.len()]
        );
        #[cfg(unix)]
        assert_eq!(segment.unix_mode, Some(0o600));
        assert!(!segment.path.exists());
    }
}

#[test]
fn admitted_execution_child_failure_cleans_private_snapshot() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        factory_reset: false,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default().with_execute_failure();

    // Act
    let error = run_flash(&command, &environment).expect_err("child failure");

    // Assert
    let error = format!("{error:#}");
    assert!(error.contains("sentinel child failure"));
    assert!(environment.observed_flashes().is_empty());
    let snapshots = environment.created_snapshot_paths();
    assert_eq!(snapshots.len(), 5);
    for path in snapshots.iter() {
        assert!(!error.contains(path.as_str()));
        assert!(!path.exists());
    }
}

#[test]
fn admitted_execution_snapshot_write_failure_precedes_later_effects() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        factory_reset: true,
        common: CommonArgs {
            port: None,
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    )
    .with_snapshot_write_failure();

    // Act
    let error = run_flash(&command, &environment).expect_err("snapshot write failure");

    // Assert
    let error = format!("{error:#}");
    assert!(error.contains("execution_snapshot_write_failed"));
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn admitted_execution_later_preparation_failure_cleans_private_snapshot() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v4(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        factory_reset: true,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("preparation failure");

    // Assert
    assert!(format!("{error:#}").contains("Wi-Fi credential file"));
    let paths = environment.created_snapshot_paths();
    assert_eq!(paths.len(), 1);
    assert!(!paths[0].exists());
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn admitted_execution_command_construction_failure_cleans_private_snapshot() {
    // Arrange
    let snapshot =
        AdmittedExecutionSnapshot::materialize(b"admitted bytes").expect("private snapshot");
    let snapshot_path = snapshot.path().to_owned();
    let developer_image = AdmittedFlashImage::DeveloperDryRun {
        display_path: Utf8PathBuf::from("developer.elf"),
    };

    // Act
    let error = flash_command_for_admitted_image(
        "/dev/cu.usbmodem101",
        &developer_image,
        snapshot.path(),
        false,
    )
    .expect_err("non-dry-run developer command");
    drop(snapshot);

    // Assert
    assert!(format!("{error:#}").contains("developer_image_requires_dry_run"));
    assert!(!snapshot_path.exists());
}
