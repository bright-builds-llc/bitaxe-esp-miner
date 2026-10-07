use super::*;

fn dry_run_command(manifest: Utf8PathBuf, maybe_web_ui: Option<WebUiVariant>) -> FlashCommand {
    let mut common = common_args();
    common.dry_run = true;
    FlashCommand {
        factory_reset: false,
        common,
        image: None,
        manifest: Some(manifest),
        wifi_credentials: None,
        maybe_web_ui,
    }
}

fn rewrite_web_ui_variant(manifest: &Utf8Path, maybe_variant: Option<&str>) {
    let mut document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest).expect("manifest"))
            .expect("manifest JSON");
    let object = document.as_object_mut().expect("manifest object");
    match maybe_variant {
        Some(variant) => object.insert("web_ui_variant".to_owned(), variant.into()),
        None => object.remove("web_ui_variant"),
    };
    std::fs::write(manifest, document.to_string()).expect("rewrite manifest");
}

#[test]
fn requested_web_ui_variant_must_match_the_manifest() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    let command = dry_run_command(manifest, Some(WebUiVariant::Solid));
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("variant mismatch must block");

    // Assert
    assert!(error.to_string().contains("web_ui_variant_mismatch"));
    assert!(environment.executed_commands().is_empty());
}

#[test]
fn matching_web_ui_variant_is_admitted() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    rewrite_web_ui_variant(&manifest, Some("solid"));
    let command = dry_run_command(manifest, Some(WebUiVariant::Solid));
    let environment = FakeFlashEnvironment::default();

    // Act
    let outcome = run_flash(&command, &environment);

    // Assert
    assert!(outcome.is_ok(), "{outcome:?}");
}

#[test]
fn manifest_without_a_web_ui_variant_is_blocked() {
    // Arrange
    let dir = tempdir().expect("tempdir");
    let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
    rewrite_web_ui_variant(&manifest, None);
    let command = dry_run_command(manifest, None);
    let environment = FakeFlashEnvironment::default();

    // Act
    let error = run_flash(&command, &environment).expect_err("missing variant must block");

    // Assert
    assert!(error
        .to_string()
        .contains("manifest_web_ui_variant_missing"));
}

#[test]
fn self_built_packages_default_to_the_current_variant() {
    // Arrange
    let requests = [None, Some(WebUiVariant::Solid)];

    // Act
    let built: Vec<_> = requests
        .iter()
        .map(|request| expected_web_ui_variant(*request, true))
        .collect();
    let explicit = expected_web_ui_variant(None, false);

    // Assert
    assert_eq!(
        built,
        [Some(WebUiVariant::Current), Some(WebUiVariant::Solid)]
    );
    assert_eq!(explicit, None);
    assert_eq!(
        WebUiVariant::Solid.bazel_flag(),
        "--//firmware/bitaxe:web_ui=solid"
    );
}

#[test]
fn flash_cli_parses_the_web_ui_variant() {
    // Arrange
    let argv = [
        "bitaxe-flash",
        "flash",
        "--board",
        "205",
        "--web-ui",
        "solid",
    ];

    // Act
    let cli = parse_cli(argv.iter().map(|value| value.to_string())).expect("parse flash");

    // Assert
    let CliCommand::Flash(command) = cli.command else {
        panic!("expected flash command");
    };
    assert_eq!(command.maybe_web_ui, Some(WebUiVariant::Solid));
}
