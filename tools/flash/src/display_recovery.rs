use std::fs;
use std::net::Ipv4Addr;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use bitaxe_http_transport::{
    strict_http_evaluator_sha256, ExchangeObservation, RequestProgress, StrictHttpClient,
};
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::*;

pub(crate) const DISPLAY_RECOVERY_PLAN: &str =
    "docs/parity/work-plans/20260830T161148Z-NATIVE-USB-DISPLAY-RECOVERY/PLAN.md";
pub(crate) const DISPLAY_RECOVERY_PLAN_SHA256: &str =
    "cba106c78f7a12105d64f185a5989ac445afe64f3479a917ac7cc95285196427";
pub(crate) const DISPLAY_RECOVERY_TASK: &str = "task-native-usb-display-recovery-205";
pub(crate) const DISPLAY_RECOVERY_ROOT: &str = "scratch/native-usb-display-recovery/attempt-001";
pub(crate) const DISPLAY_RECOVERY_BUNDLE: &str =
    "scratch/str005-installed-package-recovery/recovery-006/restore-bundle.private.json";
pub(crate) const DISPLAY_RECOVERY_BACKUP: &str =
    "scratch/str005-stratum-v2/attempt-004/settings-backup.private.json";
pub(crate) const DISPLAY_RECOVERY_MANIFEST: &str =
    "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json";

const RESTORABLE_KEYS: [&str; 38] = [
    "hostname",
    "stratumProtocol",
    "stratumURL",
    "stratumPort",
    "stratumUser",
    "stratumSuggestedDifficulty",
    "stratumExtranonceSubscribe",
    "stratumTLS",
    "stratumCert",
    "stratumV2ChannelType",
    "stratumV2AuthorityPubkey",
    "stratumDecodeCoinbase",
    "fallbackStratumProtocol",
    "fallbackStratumURL",
    "fallbackStratumPort",
    "fallbackStratumUser",
    "fallbackStratumSuggestedDifficulty",
    "fallbackStratumExtranonceSubscribe",
    "fallbackStratumTLS",
    "fallbackStratumCert",
    "fallbackStratumV2ChannelType",
    "fallbackStratumV2AuthorityPubkey",
    "fallbackStratumDecodeCoinbase",
    "useFallbackStratum",
    "frequency",
    "coreVoltage",
    "overclockEnabled",
    "display",
    "rotation",
    "invertscreen",
    "displayOffset",
    "displayTimeout",
    "autofanspeed",
    "manualFanSpeed",
    "minFanSpeed",
    "temptarget",
    "overheat_mode",
    "statsFrequency",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DisplayIpv4Origin(Ipv4Addr);

impl DisplayIpv4Origin {
    pub(crate) fn parse(candidate: &str) -> Result<Self> {
        let address: Ipv4Addr = candidate
            .parse()
            .map_err(|_| anyhow::anyhow!("display_origin=blocked reason=invalid_ipv4"))?;
        let octets = address.octets();
        if candidate != address.to_string() || !address.is_private() || matches!(octets[3], 0 | 255)
        {
            bail!("display_origin=blocked reason=non_private_ipv4");
        }
        Ok(Self(address))
    }

    pub(crate) fn origin(self) -> String {
        format!("http://{}", self.0)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct DisplayRestorationTransaction {
    pub(crate) settings: Value,
    pub(crate) theme: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayOriginCapture {
    schema_version: String,
    generation: u8,
    status: String,
    ipv4: String,
}

#[derive(Debug, Serialize)]
struct DisplayRecoveryMachineResult {
    schema_version: &'static str,
    source_commit: String,
    reference_commit: String,
    plan_sha256: &'static str,
    evaluator_sha256: String,
    package_manifest_sha256: String,
    restore_bundle_sha256: String,
    capture_sha256: String,
    usb_receipt_sha256: String,
    display_origin_supplied: bool,
    private_ipv4: bool,
    usb_mac_bound: bool,
    recovery_identity_exact: bool,
    settings_exact: bool,
    theme_exact: bool,
    mineonboot_disabled: bool,
    mining_inactive: bool,
    zero_work: bool,
    stable_physical_identity: bool,
    cleanup_complete: bool,
    settings_request_count: u8,
    theme_request_count: u8,
    reconciliation_read_count: u8,
    terminal_category: &'static str,
    redaction_status: &'static str,
}

pub(crate) fn run_display_recovery_start(
    command: &DisplayRecoveryStartCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    validate_display_recovery_invocation(command, environment)?;
    let root = environment.workspace_path(&command.private_root);
    require_display_mode(&root, 0o700, true)?;
    let capture_path = environment.workspace_path(&command.capture_input);
    require_display_mode(&capture_path, 0o600, false)?;
    let capture_document = environment.read_bytes(&capture_path)?;
    let capture: DisplayOriginCapture = serde_json::from_slice(&capture_document)?;
    let expected_capture = format!(
        "{DISPLAY_RECOVERY_ROOT}/display-origin-capture-00{}.private.json",
        capture.generation
    );
    if capture.schema_version != "bitaxe-native-usb-display-origin-capture-v1"
        || capture.status != "accepted"
        || !matches!(capture.generation, 1 | 2)
        || command.capture_input != Utf8Path::new(&expected_capture)
    {
        bail!("display_recovery=blocked reason=capture_contract");
    }
    let origin = DisplayIpv4Origin::parse(&capture.ipv4)?;
    let manifest_document =
        environment.read_bytes(&environment.workspace_path(&command.package_manifest))?;
    let manifest: Value = serde_json::from_slice(&manifest_document)?;
    let source_commit = environment.firmware_commit();
    let reference_commit = environment.reference_commit();
    if environment.pushed_firmware_commit() != source_commit
        || manifest.get("source_commit").and_then(Value::as_str) != Some(&source_commit)
        || manifest.get("reference_commit").and_then(Value::as_str) != Some(&reference_commit)
    {
        bail!("display_recovery=blocked reason=source_identity");
    }
    let bundle_document =
        environment.read_bytes(&environment.workspace_path(&command.restore_bundle))?;
    let bundle: Value = serde_json::from_slice(&bundle_document)?;
    let expected_identity = bundle
        .get("installed_identity")
        .and_then(Value::as_object)
        .context("display_recovery=blocked reason=bundle_identity")?;
    let backup: Value = serde_json::from_slice(
        &environment.read_bytes(&environment.workspace_path(&command.settings_backup))?,
    )?;
    let wifi: Value = serde_json::from_slice(
        &environment.read_bytes(&environment.workspace_path(&command.wifi_credentials))?,
    )?;
    let pool: Value = serde_json::from_slice(
        &environment.read_bytes(&environment.workspace_path(&command.pool_credentials))?,
    )?;
    let transaction = plan_display_restoration(&backup, &wifi, &pool)?;

    environment.begin_usb_session(UsbOperation::Detect, &command.port)?;
    let physical_identity = environment.usb_physical_identity_digest()?;
    let stable_physical_identity = physical_identity.len() == 64;
    let board_info = environment.execute_with_output(&CommandSpec::new(
        "espflash",
        [
            "board-info",
            "--chip",
            "esp32s3",
            "--port",
            command.port.as_str(),
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
        ],
    ))?;
    let usb_mac_sha256 = board_info_mac_sha256(&board_info)?;
    let usb_receipt = serde_json::json!({
        "schema_version": "bitaxe-native-usb-display-recovery-usb-admission-v1",
        "board": 205,
        "base_mac_sha256": usb_mac_sha256,
        "physical_identity_sha256": physical_identity,
        "board_info_admitted": true,
    });
    let usb_receipt_bytes = json_line(&usb_receipt)?;
    write_private_new_bytes(&root.join("usb-admission.private.json"), &usb_receipt_bytes)?;
    let client = StrictHttpClient::new(&origin.origin())?;
    let baseline = read_json_until(&client, DisplayRead::System, Duration::from_secs(60));
    let Ok((baseline, _baseline_read_count)) = baseline else {
        let marker = serde_json::json!({
            "schema_version": "bitaxe-native-usb-display-origin-retry-v1",
            "generation": capture.generation,
            "eligible": capture.generation == 1,
            "settings_request_count": 0,
        });
        write_private_new_bytes(
            &root.join("origin-unreachable.private.json"),
            &json_line(&marker)?,
        )?;
        bail!("display_recovery=blocked reason=origin_unreachable");
    };
    let api_mac = baseline
        .get("macAddr")
        .and_then(Value::as_str)
        .context("display_recovery=blocked reason=api_mac_missing")?;
    if display_mac_sha256(api_mac)? != usb_mac_sha256 {
        bail!("display_recovery=blocked reason=usb_mac_mismatch");
    }
    if !display_recovery_identity_matches(&baseline, expected_identity)
        || baseline.get("startMiningOnBoot") != Some(&Value::Bool(false))
    {
        bail!("display_recovery=blocked reason=recovery_identity_mismatch");
    }

    let settings_bytes = serde_json::to_vec(&transaction.settings)?;
    let theme_bytes = serde_json::to_vec(&transaction.theme)?;
    let settings_observation = client
        .patch_system_settings_once(&settings_bytes, Instant::now() + Duration::from_secs(10))?;
    if !matches!(
        settings_observation.request_progress(),
        RequestProgress::Complete { .. }
    ) {
        bail!("display_recovery=blocked reason=settings_request_failed");
    }
    let theme_observation =
        client.post_theme_once(&theme_bytes, Instant::now() + Duration::from_secs(10))?;
    if !matches!(
        theme_observation.request_progress(),
        RequestProgress::Complete { .. }
    ) {
        bail!("display_recovery=blocked reason=theme_request_failed");
    }
    let (confirmed, system_read_count) =
        read_json_until(&client, DisplayRead::System, Duration::from_secs(30))
            .context("display_recovery=blocked reason=restoration_uncertain")?;
    let (confirmed_theme, theme_read_count) =
        read_json_until(&client, DisplayRead::Theme, Duration::from_secs(30))
            .context("display_recovery=blocked reason=restoration_uncertain")?;
    if display_mac_sha256(
        confirmed
            .get("macAddr")
            .and_then(Value::as_str)
            .context("display_recovery=blocked reason=api_mac_missing")?,
    )? != usb_mac_sha256
        || !display_recovery_identity_matches(&confirmed, expected_identity)
    {
        bail!("display_recovery=blocked reason=physical_identity_drift");
    }
    if !display_restoration_is_exact(&confirmed, &confirmed_theme, &transaction, &wifi, &pool) {
        bail!("display_recovery=blocked reason=restoration_uncertain");
    }
    let mining_inactive = matches!(
        confirmed.get("miningActivity").and_then(Value::as_str),
        Some("paused" | "safe_blocked")
    );
    let zero_work = ["hashRate", "sharesAccepted", "sharesRejected"]
        .into_iter()
        .all(|field| confirmed.get(field).and_then(Value::as_f64) == Some(0.0));
    if !mining_inactive || !zero_work {
        bail!("display_recovery=blocked reason=runtime_state_mismatch");
    }
    if environment.current_usb_physical_identity_digest(&command.port)? != physical_identity {
        bail!("display_recovery=blocked reason=physical_identity_drift");
    }
    environment.finish_usb_session()?;
    let result = DisplayRecoveryMachineResult {
        schema_version: "bitaxe-native-usb-display-recovery-machine-v1",
        source_commit,
        reference_commit,
        plan_sha256: DISPLAY_RECOVERY_PLAN_SHA256,
        evaluator_sha256: display_recovery_evaluator_sha256(),
        package_manifest_sha256: sha256_bytes(&manifest_document),
        restore_bundle_sha256: sha256_bytes(&bundle_document),
        capture_sha256: sha256_bytes(&capture_document),
        usb_receipt_sha256: sha256_bytes(&usb_receipt_bytes),
        display_origin_supplied: true,
        private_ipv4: true,
        usb_mac_bound: true,
        recovery_identity_exact: true,
        settings_exact: true,
        theme_exact: true,
        mineonboot_disabled: true,
        mining_inactive,
        zero_work,
        stable_physical_identity,
        cleanup_complete: true,
        settings_request_count: 1,
        theme_request_count: 1,
        reconciliation_read_count: system_read_count.saturating_add(theme_read_count),
        terminal_category: "complete",
        redaction_status: "passed",
    };
    write_private_new_bytes(
        &root.join("machine-result.private.json"),
        &json_line(&result)?,
    )?;
    emit_line("display_recovery", "complete")
}

pub(crate) fn display_mac_sha256(candidate: &str) -> Result<String> {
    let octets = candidate.split(':').collect::<Vec<_>>();
    if octets.len() != 6
        || octets
            .iter()
            .any(|octet| octet.len() != 2 || u8::from_str_radix(octet, 16).is_err())
    {
        bail!("display_recovery=blocked reason=mac_invalid");
    }
    let canonical = octets.join(":").to_ascii_lowercase();
    let mut digest = Sha256::new();
    digest.update(canonical.as_bytes());
    Ok(hex_lower(&digest.finalize()))
}

pub(crate) fn plan_display_restoration(
    backup: &Value,
    wifi: &Value,
    pool: &Value,
) -> Result<DisplayRestorationTransaction> {
    let settings = required_object(backup, "settings")?;
    let theme = backup
        .get("theme")
        .filter(|value| value.is_object())
        .context("display_recovery=blocked reason=theme_input")?
        .clone();
    let wifi = wifi
        .as_object()
        .context("display_recovery=blocked reason=wifi_input")?;
    let pool = pool
        .as_object()
        .context("display_recovery=blocked reason=pool_input")?;
    let mut patch = Map::new();
    for key in RESTORABLE_KEYS {
        if let Some(value) = settings.get(key) {
            patch.insert(key.to_owned(), value.clone());
        }
    }
    for (target, source, object) in [
        ("ssid", "ssid", wifi),
        ("wifiPass", "wifiPass", wifi),
        ("stratumURL", "poolURL", pool),
        ("stratumPort", "poolPort", pool),
        ("stratumUser", "poolUser", pool),
        ("stratumPassword", "poolPassword", pool),
    ] {
        let value = object
            .get(source)
            .context("display_recovery=blocked reason=restoration_input")?;
        patch.insert(target.to_owned(), value.clone());
    }
    patch.insert("startMiningOnBoot".to_owned(), Value::Bool(false));
    patch.insert("useFallbackStratum".to_owned(), Value::Bool(false));
    patch.insert(
        "fallbackStratumURL".to_owned(),
        Value::String(String::new()),
    );
    Ok(DisplayRestorationTransaction {
        settings: Value::Object(patch),
        theme,
    })
}

fn validate_display_recovery_invocation(
    command: &DisplayRecoveryStartCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.package_manifest != Utf8Path::new(DISPLAY_RECOVERY_MANIFEST)
        || command.restore_bundle != Utf8Path::new(DISPLAY_RECOVERY_BUNDLE)
        || command.settings_backup != Utf8Path::new(DISPLAY_RECOVERY_BACKUP)
        || command.wifi_credentials != Utf8Path::new("wifi-credentials.json")
        || command.private_root != Utf8Path::new(DISPLAY_RECOVERY_ROOT)
        || command.plan != Utf8Path::new(DISPLAY_RECOVERY_PLAN)
        || !command.redact_evidence
    {
        bail!("display_recovery=blocked reason=invocation");
    }
    let pool_name = command
        .pool_credentials
        .file_name()
        .context("display_recovery=blocked reason=pool_input")?;
    if command.pool_credentials.parent() != Some(Utf8Path::new(""))
        || !pool_name.starts_with("pool-credentials")
        || !pool_name.ends_with(".json")
    {
        bail!("display_recovery=blocked reason=pool_input");
    }
    let plan = environment.read_bytes(&environment.workspace_path(&command.plan))?;
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if sha256_bytes(&plan) != DISPLAY_RECOVERY_PLAN_SHA256
        || !tasks.contains(&format!("### {DISPLAY_RECOVERY_TASK}"))
    {
        bail!("display_recovery=blocked reason=plan_identity");
    }
    environment.approve_private_evidence_root(&command.private_root)?;
    for relative in [
        &command.restore_bundle,
        &command.settings_backup,
        &command.wifi_credentials,
        &command.pool_credentials,
        &command.capture_input,
    ] {
        require_display_mode(&environment.workspace_path(relative), 0o600, false)?;
    }
    let machine = environment
        .workspace_path(&command.private_root)
        .join("machine-result.private.json");
    if fs::symlink_metadata(machine.as_std_path()).is_ok() {
        bail!("display_recovery=blocked reason=result_exists");
    }
    Ok(())
}

fn require_display_mode(candidate: &Utf8Path, mode: u32, directory: bool) -> Result<()> {
    let metadata = fs::symlink_metadata(candidate.as_std_path())?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        bail!("display_recovery=blocked reason=protected_mode");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != mode {
        bail!("display_recovery=blocked reason=protected_mode");
    }
    Ok(())
}

fn board_info_mac_sha256(output: &[u8]) -> Result<String> {
    let text = std::str::from_utf8(output)?;
    let candidates = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("MAC address:").map(str::trim))
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        bail!("display_recovery=blocked reason=board_info_mac");
    };
    display_mac_sha256(candidate)
}

fn read_json_response(observation: &ExchangeObservation) -> Option<Value> {
    let response = observation
        .maybe_http_response()
        .filter(|response| matches!(response.status(), 200..=299))?;
    serde_json::from_slice(response.body()).ok()
}

#[derive(Debug, Clone, Copy)]
enum DisplayRead {
    System,
    Theme,
}

fn read_json_until(
    client: &StrictHttpClient,
    kind: DisplayRead,
    timeout: Duration,
) -> Result<(Value, u8)> {
    let deadline = Instant::now() + timeout;
    let mut count = 0_u8;
    while Instant::now() < deadline {
        count = count.saturating_add(1);
        let request_deadline = (Instant::now() + Duration::from_secs(10)).min(deadline);
        let observation = match kind {
            DisplayRead::System => client.get_system_info(request_deadline)?,
            DisplayRead::Theme => client.get_theme(request_deadline)?,
        };
        if let Some(value) = read_json_response(&observation) {
            return Ok((value, count));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    bail!("display_recovery=blocked reason=origin_unreachable")
}

pub(crate) fn display_recovery_identity_matches(
    system: &Value,
    expected: &Map<String, Value>,
) -> bool {
    [
        ("sourceCommit", "source_commit"),
        ("referenceCommit", "reference_commit"),
        ("appElfSha256", "app_elf_sha256"),
        ("buildTimestampUtc", "build_timestamp_utc"),
        ("version", "build_label"),
        ("runningPartition", "running_partition"),
    ]
    .into_iter()
    .all(|(runtime, bundle)| system.get(runtime) == expected.get(bundle))
}

pub(crate) fn display_restoration_is_exact(
    system: &Value,
    theme: &Value,
    transaction: &DisplayRestorationTransaction,
    wifi: &Value,
    pool: &Value,
) -> bool {
    let Some(expected) = transaction.settings.as_object() else {
        return false;
    };
    let fields_match = RESTORABLE_KEYS
        .into_iter()
        .filter(|key| expected.contains_key(*key))
        .all(|key| system.get(key) == expected.get(key));
    fields_match
        && theme == &transaction.theme
        && system.get("startMiningOnBoot") == Some(&Value::Bool(false))
        && system.get("useFallbackStratum") == Some(&Value::Bool(false))
        && system.get("fallbackStratumURL").and_then(Value::as_str) == Some("")
        && system.get("ssid") == wifi.get("ssid")
        && system.get("stratumURL") == pool.get("poolURL")
        && system.get("stratumPort") == pool.get("poolPort")
        && system.get("stratumUser") == pool.get("poolUser")
}

fn display_recovery_evaluator_sha256() -> String {
    let mut digest = Sha256::new();
    digest.update(strict_http_evaluator_sha256().as_bytes());
    digest.update(include_str!("display_recovery.rs").as_bytes());
    digest.update(include_str!("cli.rs").as_bytes());
    hex_lower(&digest.finalize())
}

fn json_line(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn required_object<'value>(value: &'value Value, key: &str) -> Result<&'value Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .context("display_recovery=blocked reason=backup_input")
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
