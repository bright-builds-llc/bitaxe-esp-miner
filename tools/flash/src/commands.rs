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
    environment.execute(&command_spec)?;
    Ok(())
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
