use crate::*;

const MANAGED_NVS_PYTHON: &str = ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python";
const MANAGED_NVS_TOOL: &str =
    ".embuild/espressif/esp-idf/v5.5.4/components/nvs_flash/nvs_partition_tool/nvs_tool.py";
const WIFI_CREDENTIALS: &str = "wifi-credentials.json";
pub(crate) const MAX_NVS_JSON_BYTES: usize = 1_048_576;

pub(crate) const NVS_READ_ADDRESS: &str = "0x9000";
pub(crate) const NVS_READ_SIZE: &str = "0x6000";
pub(crate) const NVS_READ_ROOT: &str = "scratch/native-usb-config-ap-recovery/attempt-001";
pub(crate) const NVS_FIRST_PLAN: &str =
    "docs/parity/work-plans/20260831T033840Z-NATIVE-USB-CONFIG-AP-RECOVERY-NVS-FIRST/PLAN.md";
const NVS_FIRST_PLAN_SHA256: &str =
    "44f35fcef288199baab06da036adff88e815a49583aa3b230ea7fa565ff05bf6";

pub(crate) struct ManagedEsptoolReadFlash {
    program: Utf8PathBuf,
    args: Vec<String>,
    output: Utf8PathBuf,
}

impl ManagedEsptoolReadFlash {
    pub(crate) fn program(&self) -> &Utf8Path {
        &self.program
    }
    pub(crate) fn args(&self) -> &[String] {
        &self.args
    }
    pub(crate) fn output(&self) -> &Utf8Path {
        &self.output
    }
}

pub(crate) fn nvs_read_flash_args(port: &str, output: &Utf8Path) -> Vec<String> {
    [
        "--chip",
        "esp32s3",
        "--port",
        port,
        "--before",
        "usb_reset",
        "--after",
        "hard_reset",
        "read_flash",
        NVS_READ_ADDRESS,
        NVS_READ_SIZE,
        output.as_str(),
    ]
    .map(str::to_owned)
    .to_vec()
}

#[cfg(test)]
pub(crate) fn nvs_read_args_are_exact(args: &[String], output: &Utf8Path) -> bool {
    args == nvs_read_flash_args("admitted", output)
}

pub(crate) fn run_nvs_readback(
    command: &NvsReadbackCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.private_root != Utf8Path::new(NVS_READ_ROOT)
        || command.plan != Utf8Path::new(NVS_FIRST_PLAN)
        || command.wifi_credentials != Utf8Path::new(WIFI_CREDENTIALS)
        || !command.redact_evidence
    {
        bail!("nvs_readback=blocked reason=invocation");
    }
    let plan = environment.read_bytes(&environment.workspace_path(&command.plan))?;
    let tasks =
        environment.read_to_string(&environment.workspace_path(Utf8Path::new("TASKS.md")))?;
    if sha256_bytes(&plan) != NVS_FIRST_PLAN_SHA256
        || !tasks.contains("### task-native-usb-config-ap-recovery-205")
    {
        bail!("nvs_readback=blocked reason=plan_identity");
    }
    validate_private_wifi_credentials(
        &environment.workspace_path(&command.wifi_credentials),
        environment,
    )?;
    let expected_seed = prepare_wifi_nvs_seed(
        &command.port,
        &command.wifi_credentials,
        WifiNvsSeedMode::Ordinary,
        environment,
    )?;
    let python = validate_managed_tool(environment, MANAGED_NVS_PYTHON)?;
    let nvs_tool = validate_managed_tool(environment, MANAGED_NVS_TOOL)?;
    let root = environment.workspace_path(&command.private_root);
    if fs::symlink_metadata(root.as_std_path()).is_ok() {
        bail!("nvs_readback=blocked reason=root_exists");
    }
    environment.approve_private_evidence_root(&command.private_root)?;
    fs::create_dir_all(
        root.parent()
            .context("nvs_readback=blocked reason=root_parent")?,
    )?;
    set_private_directory_mode(
        root.parent()
            .context("nvs_readback=blocked reason=root_parent")?,
    )?;
    fs::create_dir(root.as_std_path())?;
    set_private_directory_mode(&root)?;
    let program = [
        ".embuild/espressif/python_env/idf5.5_py3.14_env/bin/esptool.py",
        ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/esptool.py",
    ]
    .into_iter()
    .map(Utf8Path::new)
    .map(|path| environment.workspace_path(path))
    .find(|path| {
        fs::symlink_metadata(path.as_std_path())
            .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
    })
    .context("nvs_readback=blocked reason=esptool_missing")?;
    let output = root.join("installed-nvs.private.bin");
    let read = ManagedEsptoolReadFlash {
        program,
        args: nvs_read_flash_args(&command.port, &output),
        output: output.clone(),
    };
    environment.begin_usb_session(UsbOperation::Recover, &command.port)?;
    environment.execute_esptool_read_flash(&read)?;
    environment.finish_usb_session()?;

    let installed_json = run_nvs_dump(&python, &nvs_tool, &output)?;
    let expected_json = run_nvs_dump(&python, &nvs_tool, &expected_seed.image)?;
    write_private_new_bytes(
        &root.join("installed-nvs-tool.private.log"),
        &installed_json,
    )?;
    write_private_new_bytes(&root.join("expected-nvs-tool.private.log"), &expected_json)?;
    let installed = parse_nvs_entries(&installed_json)?;
    let expected = parse_nvs_entries(&expected_json)?;
    let comparison = compare_expected_nvs(&installed, &expected);
    let stage = if comparison.nvs_match {
        "nvs_match"
    } else {
        "nvs_mismatch"
    };
    let state = serde_json::json!({
        "schema_version": "bitaxe-native-usb-config-ap-recovery-state-v1",
        "stage": stage,
        "nvs_sha256": sha256_bytes(&environment.read_bytes(&output)?),
        "expected_nvs_sha256": sha256_bytes(&environment.read_bytes(&expected_seed.image)?),
        "nvs_bytes": 0x6000,
        "installed_entry_count": installed.len(),
        "expected_entry_count": expected.len(),
        "integrity_check": true,
        "namespace_match": comparison.namespace_match,
        "key_set_match": comparison.key_set_match,
        "encoding_match": comparison.encoding_match,
        "value_digest_match": comparison.value_digest_match,
        "state_match": comparison.state_match,
        "nvs_match": comparison.nvs_match,
        "device_write_observed": false,
        "cleanup_complete": true,
    });
    write_private_new_bytes(&root.join("state.private.json"), &json_line(&state)?)?;
    emit_line("nvs_readback", stage)?;
    if !comparison.nvs_match {
        bail!("nvs_readback=terminal reason=nvs_mismatch");
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NvsSemanticEntry {
    namespace: String,
    key: String,
    encoding: String,
    data: serde_json::Value,
    state: String,
    is_empty: Option<bool>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct NvsComparison {
    pub(crate) namespace_match: bool,
    pub(crate) key_set_match: bool,
    pub(crate) encoding_match: bool,
    pub(crate) value_digest_match: bool,
    pub(crate) state_match: bool,
    pub(crate) nvs_match: bool,
}

pub(crate) fn parse_nvs_entries(bytes: &[u8]) -> Result<Vec<NvsSemanticEntry>> {
    if bytes.is_empty() || bytes.len() > MAX_NVS_JSON_BYTES {
        bail!("nvs_readback=terminal reason=nvs_json_size");
    }
    let mut stream =
        serde_json::Deserializer::from_slice(bytes).into_iter::<Vec<NvsSemanticEntry>>();
    let entries = stream
        .next()
        .transpose()
        .context("nvs_readback=terminal reason=nvs_json_invalid")?
        .context("nvs_readback=terminal reason=nvs_json_missing")?;
    let diagnostics = String::from_utf8_lossy(&bytes[stream.byte_offset()..]);
    let integrity_failures = [
        "wrong CRC32",
        "possibly truncated",
        "No free (empty) page",
        "Found duplicate entries",
        "Undefined namespace index",
        "Found unused namespace",
        "missing a chunk",
        "has no blob index",
        "is reported as Written but it is empty",
    ];
    if integrity_failures
        .iter()
        .any(|failure| diagnostics.contains(failure))
    {
        bail!("nvs_readback=terminal reason=nvs_integrity_failed");
    }
    Ok(entries)
}

pub(crate) fn compare_expected_nvs(
    installed: &[NvsSemanticEntry],
    expected: &[NvsSemanticEntry],
) -> NvsComparison {
    let mut namespace_match = true;
    let mut key_set_match = !expected.is_empty();
    let mut encoding_match = true;
    let mut value_digest_match = true;
    let mut state_match = true;
    let mut expected_keys = BTreeSet::new();

    for expected_entry in expected {
        if expected_entry.namespace != NVS_NAMESPACE {
            namespace_match = false;
        }
        if !expected_keys.insert((
            expected_entry.namespace.as_str(),
            expected_entry.key.as_str(),
        )) {
            key_set_match = false;
        }
        let matching_key = installed
            .iter()
            .filter(|entry| {
                entry.namespace == expected_entry.namespace && entry.key == expected_entry.key
            })
            .collect::<Vec<_>>();
        if matching_key.len() != 1 {
            key_set_match = false;
            continue;
        }
        let installed_entry = matching_key[0];
        if installed_entry.encoding != expected_entry.encoding {
            encoding_match = false;
        }
        if semantic_value_digest(&installed_entry.data)
            != semantic_value_digest(&expected_entry.data)
        {
            value_digest_match = false;
        }
        if installed_entry.state != "Written"
            || expected_entry.state != "Written"
            || installed_entry.is_empty == Some(true)
            || expected_entry.is_empty == Some(true)
        {
            state_match = false;
        }
    }

    let nvs_match =
        namespace_match && key_set_match && encoding_match && value_digest_match && state_match;
    NvsComparison {
        namespace_match,
        key_set_match,
        encoding_match,
        value_digest_match,
        state_match,
        nvs_match,
    }
}

fn semantic_value_digest(value: &serde_json::Value) -> String {
    sha256_bytes(value.to_string().as_bytes())
}

fn validate_private_wifi_credentials(
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .context("nvs_readback=blocked reason=wifi_credentials_missing")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("nvs_readback=blocked reason=wifi_credentials_type");
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("nvs_readback=blocked reason=wifi_credentials_mode");
    }
    read_wifi_credentials(path, environment).map(|_| ())
}

fn validate_managed_tool(
    environment: &impl FlashEnvironment,
    relative: &str,
) -> Result<Utf8PathBuf> {
    let workspace = environment.workspace_path(Utf8Path::new("."));
    let tool = environment.workspace_path(Utf8Path::new(relative));
    let metadata = fs::symlink_metadata(tool.as_std_path()).with_context(|| {
        format!("nvs_readback=blocked reason=managed_tool_missing path={relative}")
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("nvs_readback=blocked reason=managed_tool_type path={relative}");
    }
    let canonical_workspace = fs::canonicalize(workspace.as_std_path())?;
    let canonical_tool = fs::canonicalize(tool.as_std_path())?;
    if !canonical_tool.starts_with(&canonical_workspace) {
        bail!("nvs_readback=blocked reason=managed_tool_escape path={relative}");
    }
    Ok(tool)
}

fn run_nvs_dump(python: &Utf8Path, nvs_tool: &Utf8Path, input: &Utf8Path) -> Result<Vec<u8>> {
    let output = Command::new(python.as_std_path())
        .args([
            nvs_tool.as_str(),
            "--integrity-check",
            "--dump",
            "minimal",
            "--format",
            "json",
            input.as_str(),
        ])
        .output()
        .context("nvs_readback=terminal reason=nvs_tool_launch")?;
    if !output.status.success() {
        bail!("nvs_readback=terminal reason=nvs_integrity_failed");
    }
    if output.stdout.len() > MAX_NVS_JSON_BYTES {
        bail!("nvs_readback=terminal reason=nvs_json_size");
    }
    Ok(output.stdout)
}

fn json_line(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}
