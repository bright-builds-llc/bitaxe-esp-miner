use crate::*;

mod finalize;
mod flash;
mod phase35_probe;

pub(crate) use finalize::*;
pub(crate) use flash::*;
pub(crate) use phase35_probe::*;

pub(crate) fn combine_operation_and_cleanup(
    operation_result: Result<()>,
    cleanup_result: Result<()>,
) -> Result<()> {
    match (operation_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(cleanup_error)) => Err(cleanup_error),
        (Err(operation_error), Ok(())) => Err(operation_error),
        (Err(operation_error), Err(_cleanup_error)) => {
            Err(operation_error.context("cleanup_failure=secondary"))
        }
    }
}

pub(crate) fn maybe_write_phase36_pre_effect_result(failure: &'static str) -> Result<()> {
    maybe_write_phase36_effect_result("failed_no_device_effect", Some(failure))
}

pub(crate) fn maybe_write_phase36_operation_result(
    operation_succeeded: bool,
    device_effect_state: UsbDeviceEffectState,
) -> Result<()> {
    let (status, failure) =
        classify_phase36_operation_result(operation_succeeded, device_effect_state);
    maybe_write_phase36_effect_result(status, failure)
}

fn classify_phase36_operation_result(
    operation_succeeded: bool,
    device_effect_state: UsbDeviceEffectState,
) -> (&'static str, Option<&'static str>) {
    match (operation_succeeded, device_effect_state) {
        (true, UsbDeviceEffectState::Completed) => ("completed", None),
        (false, UsbDeviceEffectState::Completed) => {
            ("failed_after_completed_device_effect", Some("flash_failed"))
        }
        (_, UsbDeviceEffectState::ConfirmedPartial) => (
            "failed_confirmed_partial_device_effect",
            Some("flash_failed"),
        ),
        (_, UsbDeviceEffectState::None) => ("failed_no_device_effect", Some("flash_failed")),
    }
}

fn maybe_write_phase36_effect_result(
    status: &'static str,
    failure: Option<&'static str>,
) -> Result<()> {
    let maybe_path = env::var_os("PHASE36_EFFECT_RESULT_PATH");
    let maybe_operation = env::var_os("PHASE36_EFFECT_OPERATION");
    let maybe_package_digest = env::var_os("PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST");
    let maybe_factory_digest = env::var_os("PHASE36_EFFECT_FACTORY_IMAGE_DIGEST");
    if maybe_path.is_none()
        && maybe_operation.is_none()
        && maybe_package_digest.is_none()
        && maybe_factory_digest.is_none()
    {
        return Ok(());
    }
    let (Some(path), Some(operation), Some(package_digest), Some(factory_digest)) = (
        maybe_path,
        maybe_operation,
        maybe_package_digest,
        maybe_factory_digest,
    ) else {
        bail!("phase36_effect_result=failed reason=incomplete_contract");
    };
    let path = Utf8PathBuf::from_path_buf(path.into())
        .map_err(|_| anyhow::anyhow!("phase36_effect_result=failed reason=path_invalid"))?;
    let operation = operation
        .into_string()
        .map_err(|_| anyhow::anyhow!("phase36_effect_result=failed reason=operation_invalid"))?;
    let package_digest = package_digest
        .into_string()
        .map_err(|_| anyhow::anyhow!("phase36_effect_result=failed reason=identity_invalid"))?;
    let factory_digest = factory_digest
        .into_string()
        .map_err(|_| anyhow::anyhow!("phase36_effect_result=failed reason=identity_invalid"))?;
    if !path.is_absolute()
        || !matches!(operation.as_str(), "exact_package_flash" | "typed_recovery")
        || !is_lower_hex_digest(&package_digest)
        || !is_lower_hex_digest(&factory_digest)
    {
        bail!("phase36_effect_result=failed reason=contract_invalid");
    }
    let Some(parent) = path.parent() else {
        bail!("phase36_effect_result=failed reason=path_invalid");
    };
    let metadata = fs::symlink_metadata(parent.as_std_path())
        .context("phase36_effect_result=failed reason=parent_invalid")?;
    #[cfg(unix)]
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        bail!("phase36_effect_result=failed reason=parent_invalid");
    }
    let result = Phase36EffectResult {
        schema_version: PHASE36_EFFECT_SCHEMA,
        operation: &operation,
        status,
        failure,
        package_identity_digest: &package_digest,
        factory_image_digest: &factory_digest,
    };
    let mut bytes = serde_json::to_vec(&result)?;
    bytes.push(b'\n');
    write_private_new_bytes(&path, &bytes)
}

pub(crate) fn parse_cli<I, S>(args: I) -> Result<Cli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let canonical_args: Vec<String> = args.into_iter().map(Into::into).collect();
    let cli = Cli::try_parse_from(canonical_args).map_err(anyhow::Error::new)?;
    match &cli.command {
        CliCommand::Flash(command) if command.common.evidence_mode.is_some() => {
            bail!("--evidence-mode dual is supported only by flash-monitor");
        }
        CliCommand::Monitor(command) if command.common.evidence_mode.is_some() => {
            bail!("--evidence-mode dual is supported only by flash-monitor");
        }
        _ => {}
    }
    Ok(cli)
}

pub(crate) fn run_detect(
    command: &DetectCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    let port = resolve_port(command.port.as_deref(), environment)?;
    environment.begin_usb_session(UsbOperation::Detect, &port)?;
    let command_spec = CommandSpec::new(
        "espflash",
        [
            "board-info",
            "--chip",
            "esp32s3",
            "--port",
            port.as_str(),
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
        ],
    );
    let output = environment.execute_with_output(&command_spec)?;
    let candidate = configuration_candidate_from_board_info(&output)?;
    emit_line("configuration_candidate", &candidate)?;
    Ok(())
}

fn configuration_candidate_from_board_info(output: &[u8]) -> Result<String> {
    let document = std::str::from_utf8(output).context("board-info output was not valid UTF-8")?;
    let identities = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix("MAC address:").map(str::trim))
        .collect::<Vec<_>>();
    let [identity] = identities.as_slice() else {
        bail!("board-info output did not contain exactly one base MAC identity");
    };
    let octets = identity
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .collect::<Result<Vec<_>, _>>()
        .context("board-info base MAC identity was malformed")?;
    let [_, _, _, _, penultimate, final_octet] = octets.as_slice() else {
        bail!("board-info base MAC identity was malformed");
    };
    let soft_ap_final = final_octet
        .checked_add(1)
        .context("board-info base MAC identity cannot derive SoftAP identity")?;
    Ok(format!("Bitaxe_{penultimate:02X}{soft_ap_final:02X}"))
}

#[cfg(test)]
mod configuration_candidate_tests {
    use super::configuration_candidate_from_board_info;

    #[test]
    fn derives_soft_ap_candidate_from_the_single_base_mac() {
        // Arrange
        let output = b"Chip type: ESP32-S3\nMAC address: 02:00:00:00:A1:B1\n";

        // Act
        let candidate = configuration_candidate_from_board_info(output)
            .expect("single base MAC should derive a candidate");

        // Assert
        assert_eq!(candidate, "Bitaxe_A1B2");
    }

    #[test]
    fn rejects_missing_duplicate_malformed_and_overflow_identity() {
        for output in [
            "Chip type: ESP32-S3\n",
            "MAC address: invalid\n",
            "MAC address: 02:00:00:00:A1:FF\n",
            "MAC address: 02:00:00:00:A1:B1\nMAC address: 02:00:00:00:C3:D3\n",
        ] {
            // Act / Assert
            assert!(configuration_candidate_from_board_info(output.as_bytes()).is_err());
        }
    }
}

#[cfg(test)]
mod phase36_result_tests {
    use super::*;

    #[test]
    fn completed_device_effect_and_success_emit_completed() {
        // Arrange
        let state = UsbDeviceEffectState::Completed;

        // Act
        let result = classify_phase36_operation_result(true, state);

        // Assert
        assert_eq!(result, ("completed", None));
    }

    #[test]
    fn completed_device_effect_and_failure_preserve_completed_effect() {
        // Arrange
        let state = UsbDeviceEffectState::Completed;

        // Act
        let result = classify_phase36_operation_result(false, state);

        // Assert
        assert_eq!(
            result,
            ("failed_after_completed_device_effect", Some("flash_failed"))
        );
    }

    #[test]
    fn confirmed_partial_device_effect_remains_partial() {
        // Arrange
        let state = UsbDeviceEffectState::ConfirmedPartial;

        // Act
        let result = classify_phase36_operation_result(false, state);

        // Assert
        assert_eq!(
            result,
            (
                "failed_confirmed_partial_device_effect",
                Some("flash_failed")
            )
        );
    }

    #[test]
    fn absent_device_effect_remains_no_effect() {
        // Arrange
        let state = UsbDeviceEffectState::None;

        // Act
        let result = classify_phase36_operation_result(false, state);

        // Assert
        assert_eq!(result, ("failed_no_device_effect", Some("flash_failed")));
    }
}
