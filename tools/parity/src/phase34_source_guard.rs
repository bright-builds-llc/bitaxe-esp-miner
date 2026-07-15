const BUILD_SCRIPT_SOURCE: &str = include_str!("../../../firmware/bitaxe/build.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");
const CORE_SOURCE: &str = include_str!("../../../crates/bitaxe-core/src/lib.rs");
const API_WIRE_SOURCE: &str = include_str!("../../../crates/bitaxe-api/src/wire.rs");
const BUILD_IDENTITY_SOURCE: &str =
    include_str!("../../../crates/bitaxe-api/src/build_identity.rs");
const XTASK_SOURCE: &str = include_str!("../../xtask/src/main.rs");
const PACKAGE_MANIFEST_SOURCE: &str = include_str!("../../xtask/src/package_manifest.rs");
const FLASH_SOURCE: &str = include_str!("../../flash/src/main.rs");
const PACKAGE_SCRIPT_SOURCE: &str = include_str!("../../../scripts/package-firmware.sh");

#[test]
fn phase34_identity_runtime_source_guard() {
    // Arrange
    let lcd_identity = source_between(CORE_SOURCE, "fn startup_debug_build_label", "#[cfg(test)]");
    let retained_identity =
        source_between(MAIN_SOURCE, "fn retain_build_identity", "fn info_retained");
    let platform_identity = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "fn collect_platform_snapshot",
        "fn heap_free",
    );

    // Act / Assert
    assert!(BUILD_SCRIPT_SOURCE.contains("required_build_provenance"));
    assert!(!BUILD_SCRIPT_SOURCE.contains("Command::new"));
    assert!(!BUILD_SCRIPT_SOURCE.contains("git describe"));

    assert!(lcd_identity.contains("build_label.to_owned()"));
    assert!(!lcd_identity.contains(".take("));
    assert!(!lcd_identity.contains("source_commit"));

    for marker in [
        "firmware_commit={}",
        "reference_commit={}",
        "app_elf_sha256={}",
        "BITAXE_RUNTIME_BUILD_IDENTITY",
    ] {
        assert!(retained_identity.contains(marker), "missing {marker}");
    }
    assert!(BUILD_IDENTITY_SOURCE.contains(
        "runtime_build_identity semantic_version={} label={} channel={} source_dirty={} release_tag={} redacted=true"
    ));

    for assignment in [
        "platform.version = crate::build_label()",
        "platform.semantic_version = crate::semantic_version()",
        "platform.source_commit = crate::firmware_commit()",
        "platform.reference_commit = crate::reference_commit()",
        "platform.app_elf_sha256 = crate::app_elf_sha256()",
        "platform.build_channel = crate::build_channel()",
        "platform.source_dirty = crate::source_dirty()",
        "platform.maybe_release_tag = crate::maybe_release_tag()",
    ] {
        assert!(
            platform_identity.contains(assignment),
            "missing {assignment}"
        );
    }

    for field in [
        "semanticVersion",
        "sourceCommit",
        "referenceCommit",
        "appElfSha256",
        "buildChannel",
        "sourceDirty",
        "releaseTag",
    ] {
        assert!(API_WIRE_SOURCE.contains(field), "missing API field {field}");
    }
}

#[test]
fn phase34_package_and_hardware_admission_source_guard() {
    // Arrange
    let flash_preparation = source_between(
        FLASH_SOURCE,
        "fn prepare_flash",
        "fn flash_command_for_image",
    );
    let manifest_builder = source_between(
        PACKAGE_MANIFEST_SOURCE,
        "pub(crate) fn build_manifest",
        "pub(crate) fn validate_default_flash_image",
    );

    // Act / Assert
    assert!(manifest_builder.contains("BuildProvenance::parse_stamp"));
    assert!(manifest_builder.contains("schema_version: 3"));
    assert!(manifest_builder.contains("app_elf_sha256"));
    assert!(!manifest_builder.contains("Command::new"));
    assert!(!manifest_builder.contains("git describe"));
    assert!(!XTASK_SOURCE.contains("fn firmware_commit"));
    assert!(!XTASK_SOURCE.contains("fn reference_commit"));

    let image_resolution = flash_preparation
        .find("resolve_flash_image")
        .expect("identity admission must resolve the image");
    let port_resolution = flash_preparation
        .find("resolve_port")
        .expect("port resolution must remain explicit");
    assert!(image_resolution < port_resolution);
    for marker in [
        "package_source_dirty",
        "current_workspace_dirty",
        "package_workspace_identity_mismatch",
        "embedded_source_commit_mismatch",
        "app_descriptor_sha_mismatch",
    ] {
        assert!(
            FLASH_SOURCE.contains(marker),
            "missing admission gate {marker}"
        );
    }
    assert!(PACKAGE_SCRIPT_SOURCE.contains("esptool\" image_info --version 2"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--elf-sha256-offset"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("generated_partition_table"));
    assert!(!PACKAGE_SCRIPT_SOURCE.contains("espflash\n\tsave-image"));
    assert!(PACKAGE_SCRIPT_SOURCE.contains("--build-provenance-stamp"));
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source.find(start).expect("start marker should exist");
    let tail = &source[start_index..];
    let end_index = tail.find(end).expect("end marker should exist");
    &tail[..end_index]
}
