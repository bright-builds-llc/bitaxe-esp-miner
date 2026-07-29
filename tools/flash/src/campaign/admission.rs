use super::*;

#[derive(Debug)]
pub(super) struct CampaignNvsSeedOutcome {
    pub(super) command: CommandSpec,
    pub(super) _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PoolCredentialsFile {
    #[serde(rename = "poolURL")]
    pool_url: String,
    #[serde(rename = "poolPort")]
    pool_port: u16,
    #[serde(rename = "poolUser")]
    pool_user: String,
    #[serde(rename = "poolPassword")]
    pool_password: String,
}

pub(super) struct PoolCredentials {
    pool_url: String,
    pool_port: u16,
    pool_user: String,
    pool_password: String,
}

pub(super) fn admit_campaign(
    command: &MiningCampaignCommand,
    environment: &impl FlashEnvironment,
) -> std::result::Result<CampaignAdmission, CampaignFailure> {
    if command.board != BoardId::Ultra205 || !command.redact_evidence {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::AdmissionFailed,
        ));
    }
    let expected_duration = match command.stage {
        MiningCampaignStage::Observation => OBSERVATION_DURATION_SECONDS,
        MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => MINING_DURATION_SECONDS,
    };
    if command.duration_seconds != expected_duration {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::AdmissionFailed,
        ));
    }
    let stage_shape_valid = match command.stage {
        MiningCampaignStage::Observation => {
            command.profile.is_none() && command.pool_credentials.is_none()
        }
        MiningCampaignStage::LiveShare => {
            command.profile == Some(MiningCampaignProfile::Conservative)
                && command.pool_credentials.is_some()
        }
        MiningCampaignStage::Soak => {
            command.profile == Some(MiningCampaignProfile::UpstreamDefault)
                && command.pool_credentials.is_some()
        }
    };
    if !stage_shape_valid {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::AdmissionFailed,
        ));
    }
    let maybe_lease_id = (command.stage != MiningCampaignStage::Observation)
        .then(|| environment.campaign_lease_id())
        .filter(|lease_id| *lease_id != 0);
    if command.stage != MiningCampaignStage::Observation && maybe_lease_id.is_none() {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::AdmissionFailed,
        ));
    }
    Ok(CampaignAdmission {
        stage: command.stage,
        maybe_profile: command.profile,
        duration_seconds: command.duration_seconds,
        maybe_lease_id,
    })
}

pub(super) fn prepare_campaign_nvs_seed(
    command: &MiningCampaignCommand,
    admission: CampaignAdmission,
    port: &str,
    environment: &impl FlashEnvironment,
) -> Result<CampaignNvsSeedOutcome> {
    let wifi_path = environment.workspace_path(&command.wifi_credentials);
    let wifi = read_wifi_credentials(&wifi_path, environment)
        .map_err(|_| anyhow::anyhow!("campaign credential admission failed"))?;
    let maybe_pool = match command.pool_credentials.as_deref() {
        Some(path) => {
            let pool_path = environment.workspace_path(path);
            Some(
                read_pool_credentials(&pool_path, environment)
                    .map_err(|_| anyhow::anyhow!("campaign credential admission failed"))?,
            )
        }
        None => None,
    };
    let temp_dir = tempfile::Builder::new()
        .prefix("bitaxe-campaign-nvs-")
        .tempdir()
        .context("failed to create private campaign NVS directory")?;
    let temp_root = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .map_err(|_| anyhow::anyhow!("campaign credential admission failed"))?;
    set_private_directory_mode(&temp_root)?;
    let csv_path = temp_root.join("campaign-nvs.csv");
    let image_path = temp_root.join("campaign-nvs.bin");
    let csv = campaign_nvs_csv(&wifi, maybe_pool.as_ref(), admission)?;
    environment.write_file(&csv_path, &csv)?;
    set_private_file_mode(&csv_path)?;
    environment.generate_nvs_partition(&csv_path, &image_path, NVS_PARTITION_SIZE)?;
    set_private_file_mode(&image_path)?;
    Ok(CampaignNvsSeedOutcome {
        command: nvs_seed_command_for_image(port, &image_path),
        _temp_dir: temp_dir,
    })
}

fn read_pool_credentials(
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<PoolCredentials> {
    let contents = environment
        .read_to_string(path)
        .map_err(|_| anyhow::anyhow!("pool credential input unavailable"))?;
    let file: PoolCredentialsFile = serde_json::from_str(&contents)
        .map_err(|_| anyhow::anyhow!("pool credential input invalid"))?;
    validate_pool_credentials(file)
}

fn validate_pool_credentials(file: PoolCredentialsFile) -> Result<PoolCredentials> {
    let patch = SettingsPatch::from_pairs([
        ("stratumProtocol", RawSettingValue::String("SV1".to_owned())),
        ("stratumURL", RawSettingValue::String(file.pool_url)),
        (
            "stratumPort",
            RawSettingValue::Number(i64::from(file.pool_port)),
        ),
        ("stratumUser", RawSettingValue::String(file.pool_user)),
        (
            "stratumPassword",
            RawSettingValue::String(file.pool_password),
        ),
        ("stratumTLS", RawSettingValue::Number(0)),
    ]);
    let writes = match apply_settings_patch(&patch) {
        SettingsUpdateDecision::Accepted { writes } => writes,
        SettingsUpdateDecision::Rejected { .. } => {
            bail!("pool credential input failed schema admission")
        }
    };
    let pool_url = string_write(&writes, "stratumurl")?;
    let pool_port = u16_write(&writes, "stratumport")?;
    let pool_user = string_write(&writes, "stratumuser")?;
    let pool_password = string_write(&writes, "stratumpass")?;
    if pool_url.is_empty() || pool_port == 0 || pool_user.is_empty() {
        bail!("pool credential input failed closed admission");
    }
    Ok(PoolCredentials {
        pool_url,
        pool_port,
        pool_user,
        pool_password,
    })
}

fn string_write(writes: &[NvsWrite], key_name: &str) -> Result<String> {
    writes
        .iter()
        .find_map(|write| match write {
            NvsWrite::String { key, value } if key.as_str() == key_name => Some(value.clone()),
            _ => None,
        })
        .context("pool credential schema write missing")
}

fn u16_write(writes: &[NvsWrite], key_name: &str) -> Result<u16> {
    writes
        .iter()
        .find_map(|write| match write {
            NvsWrite::U16 { key, value } if key.as_str() == key_name => Some(*value),
            _ => None,
        })
        .context("pool credential schema write missing")
}

pub(super) fn campaign_nvs_csv(
    wifi: &WifiCredentials,
    maybe_pool: Option<&PoolCredentials>,
    admission: CampaignAdmission,
) -> Result<String> {
    let mut rows = vec![
        "key,type,encoding,value".to_owned(),
        format!("{NVS_NAMESPACE},namespace,,"),
        format!("wifissid,data,string,{}", csv_cell(&wifi.ssid)),
        format!("wifipass,data,string,{}", csv_cell(&wifi.wifi_pass)),
        "mineonboot,data,u16,0".to_owned(),
        format!(
            "campstage,data,string,{}",
            csv_cell(admission.stage.as_str())
        ),
    ];
    match (admission.stage, maybe_pool) {
        (MiningCampaignStage::Observation, None) => {}
        (MiningCampaignStage::Observation, Some(_)) => {
            bail!("observation campaign cannot seed pool credentials")
        }
        (_, Some(pool)) => {
            let profile = admission
                .maybe_profile
                .context("mining campaign profile missing")?;
            let lease_id = admission
                .maybe_lease_id
                .context("mining campaign lease missing")?;
            rows.extend([
                format!("campprofile,data,string,{}", profile.as_str()),
                format!("camplease,data,u64,{lease_id}"),
                format!(
                    "campdurms,data,u64,{}",
                    admission.duration_seconds.saturating_mul(1_000)
                ),
                "stratumprot,data,string,SV1".to_owned(),
                format!("stratumurl,data,string,{}", csv_cell(&pool.pool_url)),
                format!("stratumport,data,u16,{}", pool.pool_port),
                format!("stratumuser,data,string,{}", csv_cell(&pool.pool_user)),
                format!("stratumpass,data,string,{}", csv_cell(&pool.pool_password)),
                "stratumtls,data,u16,0".to_owned(),
                "usefbstartum,data,u16,0".to_owned(),
            ]);
        }
        (_, None) => bail!("mining campaign pool credentials missing"),
    }
    Ok(rows.join("\n") + "\n")
}
