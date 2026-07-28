use crate::*;

#[derive(Debug)]
pub(crate) struct FlashOutcome {
    pub(crate) manifest: Option<Utf8PathBuf>,
    pub(crate) flash_image: Utf8PathBuf,
    pub(crate) runtime_identity: Option<ExpectedRuntimeAttestationIdentity>,
    pub(crate) command: CommandSpec,
    pub(crate) nvs_seed: Option<NvsSeedOutcome>,
}

pub(crate) struct PreparedFlash {
    pub(crate) outcome: FlashOutcome,
    pub(crate) execution_command: CommandSpec,
    pub(crate) _execution_snapshot: Option<AdmittedExecutionSnapshot>,
}

pub(crate) fn prepare_flash(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<PreparedFlash> {
    ensure_ultra_205(command.common.board)?;
    let admitted_image = resolve_flash_image(command, environment)?;
    let maybe_execution_snapshot = if command.common.dry_run {
        None
    } else {
        let Some(factory_bytes) = admitted_image.maybe_factory_bytes() else {
            bail!("identity_admission=blocked reason=developer_image_requires_dry_run");
        };
        Some(environment.create_admitted_execution_snapshot(factory_bytes)?)
    };
    let execution_path = maybe_execution_snapshot
        .as_ref()
        .map(AdmittedExecutionSnapshot::path)
        .unwrap_or_else(|| admitted_image.display_path());
    let port = resolve_port(command.common.port.as_deref(), environment)?;
    let execution_command = flash_command_for_admitted_image(
        &port,
        &admitted_image,
        execution_path,
        command.common.dry_run,
    )?;
    let display_command = if maybe_execution_snapshot.is_some() {
        flash_command_for_admitted_image(
            &port,
            &admitted_image,
            Utf8Path::new("<admitted-factory-snapshot>"),
            command.common.dry_run,
        )?
    } else {
        execution_command.clone()
    };
    let nvs_seed = match &command.wifi_credentials {
        Some(path) => Some(prepare_wifi_nvs_seed(&port, path, environment)?),
        None => None,
    };

    Ok(PreparedFlash {
        outcome: FlashOutcome {
            manifest: admitted_image.maybe_manifest().map(Utf8Path::to_owned),
            flash_image: admitted_image.display_path().to_owned(),
            runtime_identity: admitted_image.maybe_runtime_identity().cloned(),
            command: display_command,
            nvs_seed,
        },
        execution_command,
        _execution_snapshot: maybe_execution_snapshot,
    })
}

pub(crate) fn flash_command_for_admitted_image(
    port: &str,
    admitted_image: &AdmittedFlashImage,
    execution_path: &Utf8Path,
    dry_run: bool,
) -> Result<CommandSpec> {
    match admitted_image {
        AdmittedFlashImage::Factory(_) => Ok(CommandSpec::new(
            "espflash",
            [
                "write-bin",
                "--chip",
                "esp32s3",
                "--port",
                port,
                "--non-interactive",
                "--before",
                "usb-reset",
                "--after",
                "hard-reset",
                "--skip-update-check",
                "0x0",
                execution_path.as_str(),
            ],
        )),
        AdmittedFlashImage::DeveloperDryRun { .. } if dry_run => Ok(CommandSpec::new(
            "espflash",
            [
                "flash",
                "--chip",
                "esp32s3",
                "--port",
                port,
                execution_path.as_str(),
            ],
        )),
        AdmittedFlashImage::DeveloperDryRun { .. } => {
            bail!("identity_admission=blocked reason=developer_image_requires_dry_run")
        }
    }
}

pub(crate) fn resolve_flash_image(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<AdmittedFlashImage> {
    if command.common.dry_run && command.manifest.is_none() {
        let Some(image) = &command.image else {
            bail!("identity_admission=blocked reason=dry_run_requires_image_or_v3_manifest");
        };
        return Ok(AdmittedFlashImage::DeveloperDryRun {
            display_path: environment.workspace_path(image),
        });
    }

    if command.image.is_some() && command.manifest.is_none() {
        bail!("identity_admission=blocked reason=explicit_image_requires_v3_manifest");
    }

    if command.manifest.is_none() {
        environment.build_package()?;
    }
    let manifest = match &command.manifest {
        Some(path) => environment.workspace_path(path),
        None => environment
            .bazel_bin()?
            .join(PACKAGE_MANIFEST_RELATIVE_PATH),
    };
    let manifest_contents = environment.read_to_string(&manifest)?;
    let package_manifest: PackageManifest = serde_json::from_str(&manifest_contents)
        .with_context(|| format!("failed to parse package manifest {manifest}"))?;
    let current_provenance = environment.current_provenance()?;
    let admitted_factory = validate_identity_admission(
        &manifest,
        &package_manifest,
        &current_provenance,
        environment,
    )?;
    if let Some(image) = &command.image {
        let explicit_image = environment.workspace_path(image);
        if explicit_image != admitted_factory.display_path {
            bail!("identity_admission=blocked reason=explicit_image_not_admitted_factory");
        }
    }

    Ok(AdmittedFlashImage::Factory(admitted_factory))
}

pub(crate) fn validate_identity_admission(
    manifest_path: &Utf8Path,
    manifest: &PackageManifest,
    current_provenance: &BuildProvenance,
    environment: &impl FlashEnvironment,
) -> Result<AdmittedFactoryImage> {
    if manifest.schema_version != 3 {
        bail!("identity_admission=blocked reason=manifest_schema_not_v3");
    }
    validate_required_artifact_kinds(manifest)?;
    let manifest_provenance = BuildProvenance::new(
        &manifest.semantic_version,
        &manifest.source_commit,
        manifest.build_identity.source_dirty,
        manifest.build_identity.release_tag.as_deref(),
        &manifest.reference_commit,
    )
    .context("identity_admission=blocked reason=manifest_provenance_invalid")?;
    let identity = manifest_provenance.build_identity();
    if manifest.build_identity.label != identity.build_label()
        || manifest.build_identity.channel != identity.build_channel().as_str()
    {
        bail!("identity_admission=blocked reason=manifest_identity_contradictory");
    }
    if identity.source_dirty() {
        bail!("identity_admission=blocked reason=package_source_dirty");
    }
    if current_provenance.build_identity().source_dirty() {
        bail!("identity_admission=blocked reason=current_workspace_dirty");
    }
    if &manifest_provenance != current_provenance {
        bail!("identity_admission=blocked reason=package_workspace_identity_mismatch");
    }
    validate_lower_hex("app_elf_sha256", &manifest.app_elf_sha256, true)?;
    let _ = resolve_manifest_default(manifest_path, Utf8Path::new(&manifest.default_flash_image))?;

    let elf_artifact = require_artifact(manifest, "firmware_elf")?;
    let elf_path = resolve_manifest_sibling(manifest_path, Utf8Path::new(&elf_artifact.path))?;
    let elf_bytes = read_validated_artifact(elf_artifact, &elf_path, environment)?;
    if sha256_bytes(&elf_bytes) != manifest.app_elf_sha256 {
        bail!("identity_admission=blocked reason=firmware_elf_app_sha_mismatch");
    }

    let ota_artifact = require_artifact(manifest, "firmware_ota_image")?;
    let ota_path = resolve_manifest_sibling(manifest_path, Utf8Path::new(&ota_artifact.path))?;
    let ota_bytes = read_validated_artifact(ota_artifact, &ota_path, environment)?;
    let app_elf_sha256 = decode_lower_hex(&manifest.app_elf_sha256)?;
    let factory_artifact = require_artifact(manifest, "factory_merged_image")?;
    let factory_path =
        resolve_manifest_factory_artifact(manifest_path, Utf8Path::new(&factory_artifact.path))?;
    let factory_bytes = read_validated_artifact(factory_artifact, &factory_path, environment)?;
    package_admission::validate_factory_ota_identity(
        &factory_bytes,
        &ota_bytes,
        package_admission::ExpectedApplicationIdentity {
            build_label: &manifest.build_identity.label,
            source_commit: &manifest.source_commit,
            app_elf_sha256: &app_elf_sha256,
        },
    )?;

    Ok(AdmittedFactoryImage {
        manifest: manifest_path.to_owned(),
        display_path: factory_path,
        bytes: factory_bytes,
        runtime_identity: ExpectedRuntimeAttestationIdentity {
            firmware_commit: manifest.source_commit.clone(),
            reference_commit: manifest.reference_commit.clone(),
            app_elf_sha256: manifest.app_elf_sha256.clone(),
        },
    })
}

pub(crate) fn require_artifact<'a>(
    manifest: &'a PackageManifest,
    kind: &str,
) -> Result<&'a PackageArtifact> {
    let mut matches = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind);
    let Some(artifact) = matches.next() else {
        bail!("identity_admission=blocked reason=missing_{kind}_artifact");
    };
    if matches.next().is_some() {
        bail!("identity_admission=blocked reason=duplicate_{kind}_artifact");
    }

    Ok(artifact)
}

pub(crate) fn validate_required_artifact_kinds(manifest: &PackageManifest) -> Result<()> {
    for kind in [
        "firmware_elf",
        "firmware_ota_image",
        "www_spiffs_image",
        "factory_merged_image",
        "partition_table",
        "otadata_initial",
    ] {
        require_artifact(manifest, kind)?;
    }

    Ok(())
}

pub(crate) fn read_validated_artifact(
    artifact: &PackageArtifact,
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<Vec<u8>> {
    validate_lower_hex("artifact_sha256", &artifact.sha256, false)?;
    let bytes = environment.read_bytes(path)?;
    if sha256_bytes(&bytes) != artifact.sha256 {
        bail!("identity_admission=blocked reason=package_artifact_digest_mismatch");
    }
    Ok(bytes)
}

pub(crate) fn validate_lower_hex(label: &str, value: &str, reject_zero: bool) -> Result<()> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid && (!reject_zero || value.bytes().any(|byte| byte != b'0')) {
        return Ok(());
    }

    bail!("identity_admission=blocked reason=invalid_{label}")
}

pub(crate) fn decode_lower_hex(value: &str) -> Result<Vec<u8>> {
    validate_lower_hex("app_elf_sha256", value, true)?;
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

pub(crate) fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => bail!("identity_admission=blocked reason=invalid_hex_nibble"),
    }
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub(crate) fn resolve_manifest_default(
    manifest: &Utf8Path,
    default_flash_image: &Utf8Path,
) -> Result<Utf8PathBuf> {
    let Some(file_name) = default_flash_image.file_name() else {
        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}");
    };

    if file_name != DEFAULT_ELF_NAME {
        if file_name == FACTORY_IMAGE_NAME {
            bail!(
                "default_flash_image must resolve to {DEFAULT_ELF_NAME}; {FACTORY_IMAGE_NAME} is only an additional artifact"
            );
        }

        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}, not {file_name}");
    }

    resolve_manifest_sibling(manifest, default_flash_image)
}

pub(crate) fn resolve_manifest_factory_artifact(
    manifest: &Utf8Path,
    factory_image: &Utf8Path,
) -> Result<Utf8PathBuf> {
    let Some(file_name) = factory_image.file_name() else {
        bail!("factory_merged_image artifact must resolve to {FACTORY_IMAGE_NAME}");
    };

    if file_name != FACTORY_IMAGE_NAME {
        bail!(
            "factory_merged_image artifact must resolve to {FACTORY_IMAGE_NAME}, not {file_name}"
        );
    }

    resolve_manifest_sibling(manifest, factory_image)
}

pub(crate) fn resolve_manifest_sibling(
    manifest: &Utf8Path,
    image: &Utf8Path,
) -> Result<Utf8PathBuf> {
    if image.is_absolute() {
        return Ok(image.to_owned());
    }

    let Some(manifest_dir) = manifest.parent() else {
        bail!("manifest path has no parent directory: {manifest}");
    };

    Ok(manifest_dir.join(image))
}
