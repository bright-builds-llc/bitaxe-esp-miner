use crate::*;

pub(crate) fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| UNAVAILABLE.to_owned())
}

pub(crate) fn parse_board(value: &str) -> std::result::Result<BoardId, String> {
    value.parse()
}

pub(crate) fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    Ok(Utf8PathBuf::from(value))
}

pub(crate) fn parse_sha256(value: &str) -> std::result::Result<String, String> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(value.to_ascii_lowercase());
    }
    Err("expected a 64-character SHA-256 digest".to_owned())
}

pub(crate) fn command_output_to_string(
    output: std::process::Output,
    description: &str,
) -> Result<String> {
    if !output.status.success() {
        bail!(
            "{description} failed: {}",
            command_stderr_or_status(&output)
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{description} output was not valid UTF-8"))?;
    Ok(stdout.trim().to_owned())
}

pub(crate) fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed_stderr = stderr.trim();
    if !trimmed_stderr.is_empty() {
        return trimmed_stderr.to_owned();
    }

    format!("exit status {}", output.status)
}

pub(crate) fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .context("failed to detect workspace directory with git rev-parse --show-toplevel")?;

    command_output_to_string(output, "git rev-parse --show-toplevel").map(Utf8PathBuf::from)
}

pub(crate) fn resolve_espflash_executable() -> Result<Utf8PathBuf> {
    let requested = env::var("ESPFLASH_BIN").unwrap_or_else(|_| "espflash".to_owned());
    let requested_path = Utf8Path::new(&requested);
    let candidate = if requested_path.components().count() > 1 || requested_path.is_absolute() {
        requested_path.to_owned()
    } else {
        env::split_paths(&env::var_os("PATH").unwrap_or_default())
            .map(|directory| directory.join(&requested))
            .find(|path| path.is_file())
            .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
            .context("espflash executable not found")?
    };
    let canonical = fs::canonicalize(candidate.as_std_path())
        .context("failed to canonicalize espflash executable")?;
    let canonical = Utf8PathBuf::from_path_buf(canonical)
        .map_err(|_| anyhow::anyhow!("espflash executable path is not UTF-8"))?;
    let metadata = fs::metadata(canonical.as_std_path())?;
    if !metadata.is_file() {
        bail!("espflash executable is not a regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("espflash executable is not executable");
        }
    }
    Ok(canonical)
}

pub(crate) fn flash_log_connected(log: &str) -> bool {
    ["Connected to device", "Chip type:", "Chip:"]
        .iter()
        .any(|marker| log.contains(marker))
}

pub(crate) fn flash_log_device_info_complete(log: &str) -> bool {
    ["Flash size:", "Crystal frequency:", "MAC address:"]
        .iter()
        .any(|marker| log.contains(marker))
}

pub(crate) fn phase35_probe_checksum_observed(log: &str) -> bool {
    log.lines()
        .filter(|line| {
            let Some(checksum) = line.trim().strip_prefix("0x") else {
                return false;
            };
            !checksum.is_empty()
                && checksum.len() <= 32
                && checksum
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .count()
        == 1
}

pub(crate) fn phase35_probe_command(espflash_bin: &Utf8Path, port: &str) -> CommandSpec {
    CommandSpec::new(
        espflash_bin.as_str(),
        [
            "checksum-md5",
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
            "4096",
        ],
    )
}

pub(crate) fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn set_private_directory_mode(path: &Utf8Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub(crate) fn set_private_file_mode(path: &Utf8Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub(crate) fn write_private_new_bytes(path: &Utf8Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path.as_std_path())?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(crate) fn maybe_git_output<const N: usize>(
    workspace_dir: &Utf8Path,
    args: [&str; N],
) -> Option<String> {
    let output = Command::new("git")
        .current_dir(workspace_dir.as_std_path())
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_owned())
}
