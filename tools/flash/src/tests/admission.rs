use super::*;

#[test]
fn dry_run_flash_resolves_admitted_factory_artifact() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        common: common_args(),
        image: None,
        manifest: Some(manifest.clone()),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
    assert_eq!(
        outcome.flash_image,
        manifest.parent().expect("parent").join(FACTORY_IMAGE_NAME)
    );
    assert_eq!(
        outcome.command.args,
        vec![
            "write-bin",
            "--no-stub",
            "--chip",
            "esp32s3",
            "--port",
            "/dev/cu.usbmodem101",
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "no-reset",
            "--skip-update-check",
            "0x0",
            outcome.flash_image.as_str(),
        ]
    );
}

#[test]
fn relative_image_argument_resolves_under_workspace_dir() {
    // Arrange
    let workspace = tempdir().expect("workspace");
    let workspace_dir = dir_path(&workspace);
    let command = FlashCommand {
        common: common_args(),
        image: Some(Utf8PathBuf::from("docs/evidence/bitaxe-ultra205.elf")),
        manifest: None,
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    assert_eq!(
        outcome.flash_image,
        workspace_dir.join("docs/evidence/bitaxe-ultra205.elf")
    );
}

#[test]
fn relative_manifest_argument_resolves_under_workspace_dir() {
    // Arrange
    let workspace = tempdir().expect("workspace");
    let workspace_dir = dir_path(&workspace);
    let manifest = write_manifest_at(
        &workspace_dir,
        "docs/evidence/package/bitaxe-ultra205-package.json",
        DEFAULT_ELF_NAME,
    );
    let command = FlashCommand {
        common: common_args(),
        image: None,
        manifest: Some(Utf8PathBuf::from(
            "docs/evidence/package/bitaxe-ultra205-package.json",
        )),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
    assert_eq!(
        outcome.flash_image,
        workspace_dir
            .join("docs/evidence/package")
            .join(FACTORY_IMAGE_NAME)
    );
}

#[test]
fn rejects_manifest_default_factory_bin() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, FACTORY_IMAGE_NAME);
    let command = FlashCommand {
        common: common_args(),
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    assert!(format!("{result:#?}").contains(DEFAULT_ELF_NAME));
}

#[test]
fn manifest_v3_uses_factory_artifact_for_full_flash() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let command = FlashCommand {
        common: common_args(),
        image: None,
        manifest: Some(manifest.clone()),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment).expect("flash");

    // Assert
    assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
    assert_eq!(
        outcome.flash_image,
        manifest.parent().expect("parent").join(FACTORY_IMAGE_NAME)
    );
    assert_eq!(
        outcome.command.args,
        vec![
            "write-bin",
            "--no-stub",
            "--chip",
            "esp32s3",
            "--port",
            "/dev/cu.usbmodem101",
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "no-reset",
            "--skip-update-check",
            "0x0",
            outcome.flash_image.as_str(),
        ]
    );
}

#[test]
fn identity_admission_accepts_clean_dev_and_release_builds() {
    // Arrange
    let cases = [
        BuildProvenance::new(
            "0.1.0",
            SOURCE_COMMIT,
            false,
            None::<&str>,
            REFERENCE_COMMIT,
        )
        .expect("dev provenance"),
        BuildProvenance::new(
            "1.2.0",
            SOURCE_COMMIT,
            false,
            Some("v1.2"),
            REFERENCE_COMMIT,
        )
        .expect("release provenance"),
    ];

    for provenance in cases {
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        rewrite_manifest_provenance(&manifest, &provenance);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment =
            FakeFlashEnvironment::default().with_current_provenance(provenance.clone());

        // Act
        let outcome = run_flash(&command, &environment);

        // Assert
        assert!(outcome.is_ok(), "{outcome:#?}");
    }
}

#[test]
fn identity_admission_rejects_dirty_package_before_port_or_credentials() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let dirty_provenance =
        BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
            .expect("dirty provenance");
    rewrite_manifest_provenance(&manifest, &dirty_provenance);
    let command = FlashCommand {
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
    let error = format!("{result:#?}");
    assert!(error.contains("identity_admission=blocked reason=package_source_dirty"));
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
}

#[test]
fn identity_admission_rejects_dirty_current_workspace_before_port() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let dirty_provenance =
        BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
            .expect("dirty provenance");
    let command = FlashCommand {
        common: CommonArgs {
            port: None,
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    )
    .with_current_provenance(dirty_provenance);

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("identity_admission=blocked reason=current_workspace_dirty"));
    assert!(!error.contains("Ambiguous serial ports"));
}

#[test]
fn identity_admission_rejects_unmanifested_explicit_image_before_port() {
    // Arrange
    let command = FlashCommand {
        common: CommonArgs {
            port: None,
            dry_run: false,
            ..common_args()
        },
        image: Some(Utf8PathBuf::from("/tmp/firmware.bin")),
        manifest: None,
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::with_ports(
        "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
    );

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    let error = format!("{result:#?}");
    assert!(error.contains("identity_admission=blocked reason=explicit_image_requires_v3_manifest"));
    assert!(!error.contains("Ambiguous serial ports"));
}

#[test]
fn identity_admission_rejects_package_digest_mismatch() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    let ota = manifest
        .parent()
        .expect("manifest parent")
        .join("esp-miner.bin");
    std::fs::write(ota.as_std_path(), b"tampered ota").expect("tamper ota");
    let command = FlashCommand {
        common: common_args(),
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
    };
    let environment = FakeFlashEnvironment::default();

    // Act
    let result = run_flash(&command, &environment);

    // Assert
    assert!(format!("{result:#?}")
        .contains("identity_admission=blocked reason=package_artifact_digest_mismatch"));
}

#[test]
fn identity_admission_rejects_duplicate_ota_before_port_or_credentials() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    duplicate_manifest_artifact(&manifest, "firmware_ota_image");
    let command = FlashCommand {
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
    let error = format!("{result:#?}");
    assert!(
        error.contains("identity_admission=blocked reason=duplicate_firmware_ota_image_artifact")
    );
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
}

#[test]
fn identity_admission_rejects_duplicate_factory_before_port_or_credentials() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
    duplicate_manifest_artifact(&manifest, "factory_merged_image");
    let command = FlashCommand {
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
    let error = format!("{result:#?}");
    assert!(
        error.contains("identity_admission=blocked reason=duplicate_factory_merged_image_artifact")
    );
    assert!(!error.contains("Ambiguous serial ports"));
    assert!(!error.contains("credentials"));
}
