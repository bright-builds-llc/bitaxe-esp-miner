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
    let result = Phase36PreEffectResult {
        schema_version: PHASE36_EFFECT_SCHEMA,
        operation: &operation,
        status: "failed_no_device_effect",
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
    let normalized = normalize_args(args);
    let cli = Cli::try_parse_from(normalized).map_err(anyhow::Error::new)?;
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

pub(crate) fn normalize_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for arg in args {
        let arg = arg.into();
        if arg.starts_with("--") {
            normalized.push(arg);
            continue;
        }

        let Some((key, value)) = arg.split_once('=') else {
            normalized.push(arg);
            continue;
        };

        match key {
            "board" => push_flag_value(&mut normalized, "--board", value),
            "port" => push_flag_value(&mut normalized, "--port", value),
            "image" => push_flag_value(&mut normalized, "--image", value),
            "manifest" => push_flag_value(&mut normalized, "--manifest", value),
            "stage" => push_flag_value(&mut normalized, "--stage", value),
            "profile" => push_flag_value(&mut normalized, "--profile", value),
            "wifi-credentials" | "wifi_credentials" => {
                push_flag_value(&mut normalized, "--wifi-credentials", value)
            }
            "pool-credentials" | "pool_credentials" => {
                push_flag_value(&mut normalized, "--pool-credentials", value)
            }
            "evidence-dir" | "evidence_dir" => {
                push_flag_value(&mut normalized, "--evidence-dir", value)
            }
            "evidence-mode" | "evidence_mode" => {
                push_flag_value(&mut normalized, "--evidence-mode", value)
            }
            "expected-private-sha256" | "expected_private_sha256" => {
                push_flag_value(&mut normalized, "--expected-private-sha256", value)
            }
            "capture-timeout-seconds" | "capture_timeout_seconds" => {
                push_flag_value(&mut normalized, "--capture-timeout-seconds", value)
            }
            "duration-seconds" | "duration_seconds" => {
                push_flag_value(&mut normalized, "--duration-seconds", value)
            }
            "stage-root" | "stage_root" => push_flag_value(&mut normalized, "--stage-root", value),
            "timeout-seconds" | "timeout_seconds" => {
                push_flag_value(&mut normalized, "--timeout-seconds", value)
            }
            "redact-evidence" | "redact_evidence" => {
                if parse_bool_alias(value) {
                    normalized.push("--redact-evidence".to_owned());
                }
            }
            "dry-run" | "dry_run" => {
                if parse_bool_alias(value) {
                    normalized.push("--dry-run".to_owned());
                }
            }
            _ => normalized.push(arg),
        }
    }

    normalized
}

pub(crate) fn push_flag_value(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}

pub(crate) fn parse_bool_alias(value: &str) -> bool {
    matches!(value, "true" | "1" | "yes" | "on")
}
