use super::*;

#[derive(Debug)]
pub(super) struct CampaignNvsSeedOutcome {
    pub(super) command: CommandSpec,
    pub(super) _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PoolCredentialsFile {
    #[serde(rename = "poolURL")]
    pub(crate) pool_url: String,
    #[serde(rename = "poolPort")]
    pub(crate) pool_port: u16,
    #[serde(rename = "poolUser")]
    pub(crate) pool_user: String,
    #[serde(rename = "poolPassword")]
    pub(crate) pool_password: String,
    #[serde(rename = "stratumProtocol", default)]
    pub(crate) stratum_protocol: Option<String>,
    #[serde(rename = "stratumV2ChannelType", default)]
    pub(crate) stratum_v2_channel_type: Option<String>,
    #[serde(rename = "stratumV2AuthorityPubkey", default)]
    pub(crate) stratum_v2_authority_pubkey: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PoolCredentials {
    pub(crate) pool_url: String,
    pub(crate) pool_port: u16,
    pub(crate) pool_user: String,
    pub(crate) pool_password: String,
    pub(crate) stratum_protocol: String,
    pub(crate) stratum_v2_channel_type: Option<String>,
    pub(crate) stratum_v2_authority_pubkey: Option<String>,
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
        MiningCampaignStage::JobTransition => JOB_TRANSITION_DURATION_SECONDS,
        MiningCampaignStage::CommandEffects => COMMAND_EFFECTS_DURATION_SECONDS,
        MiningCampaignStage::StratumV2 => STRATUM_V2_DURATION_SECONDS,
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
        MiningCampaignStage::JobTransition => {
            command.profile == Some(MiningCampaignProfile::Conservative)
                && command.pool_credentials.is_some()
        }
        MiningCampaignStage::CommandEffects => {
            command.profile == Some(MiningCampaignProfile::Conservative)
                && command.pool_credentials.is_some()
        }
        MiningCampaignStage::StratumV2 => {
            command.profile == Some(MiningCampaignProfile::Conservative)
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

pub(crate) fn read_pool_credentials(
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

pub(crate) fn validate_pool_credentials(file: PoolCredentialsFile) -> Result<PoolCredentials> {
    let protocol = file.stratum_protocol.unwrap_or_else(|| "SV1".to_owned());
    if protocol == "SV2" {
        if file.stratum_v2_channel_type.as_deref() != Some("standard") {
            bail!("Stratum V2 campaign requires a standard channel");
        }
        let authority = file
            .stratum_v2_authority_pubkey
            .as_deref()
            .context("Stratum V2 campaign authority key missing")?;
        if bitaxe_stratum::v2::authority::parse_authority_public_key(authority)?.is_none() {
            bail!("Stratum V2 campaign authority key missing");
        }
    }
    let mut pairs = vec![
        ("stratumProtocol", RawSettingValue::String(protocol.clone())),
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
    ];
    if let Some(channel_type) = file.stratum_v2_channel_type {
        pairs.push((
            "stratumV2ChannelType",
            RawSettingValue::String(channel_type),
        ));
    }
    if let Some(authority) = file.stratum_v2_authority_pubkey {
        pairs.push((
            "stratumV2AuthorityPubkey",
            RawSettingValue::String(authority),
        ));
    }
    let patch = SettingsPatch::from_pairs(pairs);
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
    let stratum_protocol = string_write(&writes, "stratumprot")?;
    let stratum_v2_channel_type = writes.iter().find_map(|write| match write {
        NvsWrite::String { key, value } if key.as_str() == "sv2chantype" => Some(value.clone()),
        _ => None,
    });
    let stratum_v2_authority_pubkey = writes.iter().find_map(|write| match write {
        NvsWrite::String { key, value } if key.as_str() == "sv2authpubkey" => Some(value.clone()),
        _ => None,
    });
    if pool_url.is_empty() || pool_port == 0 || pool_user.is_empty() {
        bail!("pool credential input failed closed admission");
    }
    Ok(PoolCredentials {
        pool_url,
        pool_port,
        pool_user,
        pool_password,
        stratum_protocol,
        stratum_v2_channel_type,
        stratum_v2_authority_pubkey,
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
            let expected_protocol = if admission.stage == MiningCampaignStage::StratumV2 {
                "SV2"
            } else {
                "SV1"
            };
            if pool.stratum_protocol != expected_protocol {
                bail!("campaign pool protocol does not match stage");
            }
            if admission.stage == MiningCampaignStage::StratumV2
                && (pool.stratum_v2_channel_type.as_deref() != Some("standard")
                    || pool.stratum_v2_authority_pubkey.is_none())
            {
                bail!("Stratum V2 campaign requires standard channel and authority key");
            }
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
                format!("stratumprot,data,string,{}", pool.stratum_protocol),
                format!("stratumurl,data,string,{}", csv_cell(&pool.pool_url)),
                format!("stratumport,data,u16,{}", pool.pool_port),
                format!("stratumuser,data,string,{}", csv_cell(&pool.pool_user)),
                format!("stratumpass,data,string,{}", csv_cell(&pool.pool_password)),
                "stratumtls,data,u16,0".to_owned(),
                "usefbstartum,data,u16,0".to_owned(),
            ]);
            if let Some(channel_type) = &pool.stratum_v2_channel_type {
                rows.push(format!(
                    "sv2chantype,data,string,{}",
                    csv_cell(channel_type)
                ));
            }
            if let Some(authority) = &pool.stratum_v2_authority_pubkey {
                rows.push(format!("sv2authpubkey,data,string,{}", csv_cell(authority)));
            }
        }
        (_, None) => bail!("mining campaign pool credentials missing"),
    }
    Ok(rows.join("\n") + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stratum_v2_credentials_seed_only_standard_noise_campaign_keys() {
        // Arrange
        let pool = validate_pool_credentials(PoolCredentialsFile {
            pool_url: "private-host-canary".to_owned(),
            pool_port: 1234,
            pool_user: "private-user-canary".to_owned(),
            pool_password: String::new(),
            stratum_protocol: Some("SV2".to_owned()),
            stratum_v2_channel_type: Some("standard".to_owned()),
            stratum_v2_authority_pubkey: Some(
                bitaxe_stratum::v2::authority::encode_authority_public_key([0x22; 32]),
            ),
        })
        .expect("V2 credentials");
        let wifi = WifiCredentials {
            ssid: "private-wifi-canary".to_owned(),
            wifi_pass: "private-pass-canary".to_owned(),
        };
        let admission = CampaignAdmission {
            stage: MiningCampaignStage::StratumV2,
            maybe_profile: Some(MiningCampaignProfile::Conservative),
            duration_seconds: 180,
            maybe_lease_id: Some(7),
        };

        // Act
        let csv = campaign_nvs_csv(&wifi, Some(&pool), admission).expect("campaign NVS");

        // Assert
        assert!(csv.contains("campstage,data,string,stratum-v2"));
        assert!(csv.contains("stratumprot,data,string,SV2"));
        assert!(csv.contains("sv2chantype,data,string,standard"));
        assert!(csv.contains("sv2authpubkey,data,string,"));
        assert!(!csv.contains("stratumprot,data,string,SV1"));
    }

    #[test]
    fn stratum_v2_campaign_rejects_v1_or_unauthenticated_pool_shape() {
        // Arrange
        let wifi = WifiCredentials {
            ssid: "wifi".to_owned(),
            wifi_pass: "password".to_owned(),
        };
        let admission = CampaignAdmission {
            stage: MiningCampaignStage::StratumV2,
            maybe_profile: Some(MiningCampaignProfile::Conservative),
            duration_seconds: 180,
            maybe_lease_id: Some(7),
        };
        let v1 = PoolCredentials {
            pool_url: "pool".to_owned(),
            pool_port: 1,
            pool_user: "user".to_owned(),
            pool_password: String::new(),
            stratum_protocol: "SV1".to_owned(),
            stratum_v2_channel_type: None,
            stratum_v2_authority_pubkey: None,
        };
        let unauthenticated_v2 = PoolCredentials {
            stratum_protocol: "SV2".to_owned(),
            stratum_v2_channel_type: Some("standard".to_owned()),
            ..v1.clone()
        };

        // Act
        let v1_result = campaign_nvs_csv(&wifi, Some(&v1), admission);
        let unauthenticated_result = campaign_nvs_csv(&wifi, Some(&unauthenticated_v2), admission);

        // Assert
        assert!(v1_result.is_err());
        assert!(unauthenticated_result.is_err());
    }
}
