use super::*;

#[test]
fn virtual_package_rejected_before_capture_discovery_or_credentials() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("virtual.json");
    std::fs::write(
        &path,
        r#"{"execution_profile":"virtual-ultra205","hardware_eligible":false}"#,
    )
    .expect("manifest");
    let command = FlashCommand {
        factory_reset: true,
        common: CommonArgs {
            dry_run: false,
            ..common_args()
        },
        image: None,
        manifest: Some(Utf8PathBuf::from_path_buf(path).expect("UTF8 path")),
        wifi_credentials: Some(Utf8PathBuf::from("/missing-sensitive-input")),
    };
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash(&command, &environment).expect_err("virtual rejected");
    // Assert
    assert!(error.to_string().contains("virtual_image_not_physical"));
    assert_eq!(environment.list_ports_calls.get(), 0);
    assert!(environment.capture_lifecycle.borrow().is_empty());
    assert!(environment.executed_commands.borrow().is_empty());
    assert_eq!(environment.read_string_paths.borrow().len(), 1);
}

#[test]
fn virtual_marker_rejected_even_when_manifest_claims_physical() {
    // Arrange
    let elf = b"ELF bytes BITAXE_EXECUTION_PROFILE=virtual-ultra205 end";
    // Act
    let result = crate::execution_profile::reject_virtual_bytes(elf);
    // Assert
    assert!(result.is_err());
}

#[test]
fn virtual_flash_monitor_rejected_before_reader_preparation() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let mut command = flash_monitor_fixture(&dir, dir_path(&dir).join("evidence"));
    let path = dir_path(&dir).join("virtual.json");
    std::fs::write(&path, r#"{"execution_profile":"virtual-ultra205"}"#).expect("manifest");
    command.manifest = Some(path);
    let environment = FakeFlashEnvironment::default();
    // Act
    let error = run_flash_monitor(&command, &environment).expect_err("virtual rejected");
    // Assert
    assert!(error.to_string().contains("virtual_image_not_physical"));
    assert!(environment.capture_lifecycle.borrow().is_empty());
    assert_eq!(environment.list_ports_calls.get(), 0);
}
