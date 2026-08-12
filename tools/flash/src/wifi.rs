use crate::*;

#[derive(Debug)]
pub(crate) struct NvsSeedOutcome {
    pub(crate) image: Utf8PathBuf,
    pub(crate) command: CommandSpec,
    pub(crate) _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum WifiNvsSeedMode {
    Ordinary,
    NetworkReconnectProbe,
}

pub(crate) fn prepare_wifi_nvs_seed(
    port: &str,
    credentials_path: &Utf8Path,
    mode: WifiNvsSeedMode,
    environment: &impl FlashEnvironment,
) -> Result<NvsSeedOutcome> {
    let credentials_path = environment.workspace_path(credentials_path);
    let credentials = read_wifi_credentials(&credentials_path, environment)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("bitaxe-wifi-nvs-")
        .tempdir()
        .context("failed to create temporary Wi-Fi NVS directory")?;
    let temp_dir_path =
        Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).map_err(|path| {
            anyhow::anyhow!("temporary Wi-Fi NVS directory is not valid UTF-8: {path:?}")
        })?;
    let csv_path = temp_dir_path.join("wifi-nvs.csv");
    let image_path = temp_dir_path.join("wifi-nvs.bin");
    environment.write_file(&csv_path, &wifi_nvs_csv_for_mode(&credentials, mode))?;
    environment.generate_nvs_partition(&csv_path, &image_path, NVS_PARTITION_SIZE)?;

    Ok(NvsSeedOutcome {
        command: nvs_seed_command_for_image(port, &image_path),
        image: image_path,
        _temp_dir: temp_dir,
    })
}

pub(crate) fn nvs_seed_command_for_image(port: &str, nvs_image: &Utf8Path) -> CommandSpec {
    CommandSpec::new(
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
            NVS_PARTITION_OFFSET,
            nvs_image.as_str(),
        ],
    )
}

#[derive(Debug, Deserialize)]
pub(crate) struct WifiCredentialsFile {
    pub(crate) ssid: String,
    #[serde(rename = "wifiPass")]
    pub(crate) wifi_pass: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct WifiCredentials {
    pub(crate) ssid: String,
    pub(crate) wifi_pass: String,
}

pub(crate) fn read_wifi_credentials(
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<WifiCredentials> {
    let contents = environment
        .read_to_string(path)
        .with_context(|| format!("failed to read Wi-Fi credential file {path}"))?;
    let file: WifiCredentialsFile = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse Wi-Fi credential file JSON {path}"))?;
    validate_wifi_credentials(file)
}

pub(crate) fn validate_wifi_credentials(file: WifiCredentialsFile) -> Result<WifiCredentials> {
    let patch = SettingsPatch::from_pairs([
        ("ssid", RawSettingValue::String(file.ssid)),
        ("wifiPass", RawSettingValue::String(file.wifi_pass)),
    ]);

    match apply_settings_patch(&patch) {
        SettingsUpdateDecision::Accepted { writes } => Ok(WifiCredentials {
            ssid: string_write_value(&writes, "wifissid")?,
            wifi_pass: string_write_value(&writes, "wifipass")?,
        }),
        SettingsUpdateDecision::Rejected { errors } => {
            bail!(
                "invalid Wi-Fi credentials: {}",
                validation_error_summaries(&errors)
            );
        }
    }
}

pub(crate) fn string_write_value(writes: &[NvsWrite], key_name: &str) -> Result<String> {
    writes
        .iter()
        .find_map(|write| match write {
            NvsWrite::String { key, value } if key.as_str() == key_name => Some(value.clone()),
            _ => None,
        })
        .with_context(|| format!("validated Wi-Fi patch did not produce {key_name} NVS write"))
}

pub(crate) fn validation_error_summaries(errors: &[ConfigValidationError]) -> String {
    errors
        .iter()
        .map(validation_error_summary)
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn validation_error_summary(error: &ConfigValidationError) -> String {
    match error {
        ConfigValidationError::InvalidLength {
            field,
            min,
            max,
            actual,
        } => format!("{field} length {actual} is outside {min}..={max}"),
        ConfigValidationError::OutOfRange {
            field,
            min,
            max,
            actual,
        } => format!("{field} value {actual} is outside {min}..={max}"),
        ConfigValidationError::InvalidEnum { field, .. } => {
            format!("{field} has an invalid value")
        }
        ConfigValidationError::InvalidBoardScope { .. } => {
            "board version is not active hardware-verified scope".to_owned()
        }
        ConfigValidationError::InvalidNvsKeyName { max_bytes, .. } => {
            format!("NVS key name is invalid; maximum length is {max_bytes} bytes")
        }
    }
}

#[cfg(test)]
pub(crate) fn wifi_nvs_csv(credentials: &WifiCredentials) -> String {
    wifi_nvs_csv_for_mode(credentials, WifiNvsSeedMode::Ordinary)
}

pub(crate) fn wifi_nvs_csv_for_mode(
    credentials: &WifiCredentials,
    mode: WifiNvsSeedMode,
) -> String {
    let mut rows = vec![
        "key,type,encoding,value".to_owned(),
        format!("{NVS_NAMESPACE},namespace,,"),
    ];
    rows.extend(
        ultra_205_default_seed_values()
            .iter()
            .map(private_nvs_csv_row),
    );
    rows.extend([
        private_nvs_csv_row(&StoredValue::string("wifissid", &credentials.ssid)),
        private_nvs_csv_row(&StoredValue::string("wifipass", &credentials.wifi_pass)),
        private_nvs_csv_row(&StoredValue::u16("mineonboot", 0)),
    ]);
    if mode == WifiNvsSeedMode::NetworkReconnectProbe {
        rows.push(private_nvs_csv_row(&StoredValue::u16("netreconprobe", 1)));
    }
    rows.join("\n") + "\n"
}

fn private_nvs_csv_row(value: &StoredValue) -> String {
    match &value.value {
        StoredValueKind::String(contents) => {
            format!("{},data,string,{}", value.key.as_str(), csv_cell(contents))
        }
        StoredValueKind::U16(contents) => {
            format!("{},data,u16,{contents}", value.key.as_str())
        }
        StoredValueKind::I32(contents) => {
            format!("{},data,i32,{contents}", value.key.as_str())
        }
        StoredValueKind::U64(contents) => {
            format!("{},data,u64,{contents}", value.key.as_str())
        }
    }
}

pub(crate) fn csv_cell(value: &str) -> String {
    if !value
        .chars()
        .any(|character| matches!(character, ',' | '"' | '\n' | '\r'))
    {
        return value.to_owned();
    }

    format!("\"{}\"", value.replace('"', "\"\""))
}
