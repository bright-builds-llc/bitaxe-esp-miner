use crate::*;

pub(crate) trait FlashEnvironment {
    fn build_package(&self) -> Result<()>;
    fn bazel_bin(&self) -> Result<Utf8PathBuf>;
    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        path.to_owned()
    }
    fn read_to_string(&self, path: &Utf8Path) -> Result<String>;
    fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>>;
    fn create_admitted_execution_snapshot(
        &self,
        bytes: &[u8],
    ) -> Result<AdmittedExecutionSnapshot> {
        AdmittedExecutionSnapshot::materialize(bytes)
    }
    fn approve_private_evidence_root(&self, path: &Utf8Path) -> Result<()>;
    fn current_provenance(&self) -> Result<BuildProvenance>;
    fn list_ports(&self) -> Result<String>;
    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()>;
    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()>;
    fn begin_usb_session(&self, operation: UsbOperation, port: &str) -> Result<()>;
    fn execute(&self, command_spec: &CommandSpec) -> Result<()>;
    fn execute_with_output(&self, command_spec: &CommandSpec) -> Result<Vec<u8>>;
    fn receive_only(&self, command_spec: &CommandSpec, timeout_seconds: u64) -> Result<Vec<u8>>;
    fn campaign_lease_id(&self) -> u64;
    fn receive_campaign_until(
        &self,
        admission: CampaignAdmission,
        expected_runtime: ExpectedRuntimeAttestationIdentity,
        evidence_root: &Utf8Path,
        capture_limit: CampaignCaptureLimit,
    ) -> Result<campaign::network::CampaignObservationCapture>;
    fn receive_input_uat(&self, stop: &mut dyn FnMut(&[u8]) -> bool) -> Result<MonitorOutput>;
    fn finish_usb_session(&self) -> Result<()>;
    fn device_effect_state(&self) -> UsbDeviceEffectState {
        UsbDeviceEffectState::None
    }
    fn last_usb_command_diagnostic(&self) -> Option<UsbCommandDiagnostic> {
        None
    }
    fn phase35_stage_readiness_gate(&self, _stage: &str, _port: &str) -> Result<()> {
        Ok(())
    }
    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        timeout_seconds: u64,
        redaction_mode: EvidenceRedactionMode,
        create_new: bool,
    ) -> Result<CaptureProcessResult>;
    fn firmware_commit(&self) -> String;
    fn reference_commit(&self) -> String;
    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()>;
}

pub(crate) struct LocalFlashEnvironment {
    pub(crate) workspace_dir: Utf8PathBuf,
    pub(crate) espflash_bin: Utf8PathBuf,
    pub(crate) espflash_version: String,
    pub(crate) espflash_sha256: String,
    pub(crate) usb_session: RefCell<Option<UsbSession>>,
}

impl LocalFlashEnvironment {
    pub(crate) fn detect() -> Result<Self> {
        let espflash_bin = resolve_espflash_executable()?;
        let espflash_version = format!("espflash {ESPFLASH_EXPECTED_VERSION}");
        let espflash_sha256 = sha256_bytes(
            &fs::read(espflash_bin.as_std_path())
                .context("failed to digest espflash executable")?,
        );
        Ok(Self {
            workspace_dir: detect_workspace_dir()?,
            espflash_bin,
            espflash_version,
            espflash_sha256,
            usb_session: RefCell::new(None),
        })
    }
}

pub(crate) fn approve_local_private_evidence_root(
    workspace_dir: &Utf8Path,
    requested_root: &Utf8Path,
) -> Result<()> {
    let canonical_workspace = fs::canonicalize(workspace_dir.as_std_path())
        .context("failed to resolve workspace for private evidence admission")?;
    let canonical_workspace = Utf8PathBuf::from_path_buf(canonical_workspace)
        .map_err(|_| anyhow::anyhow!("private_evidence_root=blocked reason=non_utf8_workspace"))?;
    let relative_root = if requested_root.is_absolute() {
        requested_root
            .strip_prefix(&canonical_workspace)
            .or_else(|_| requested_root.strip_prefix(workspace_dir))
            .map(Utf8Path::to_owned)
            .map_err(|_| {
                anyhow::anyhow!("private_evidence_root=blocked reason=outside_workspace")
            })?
    } else {
        requested_root.to_owned()
    };
    if relative_root.as_str().is_empty()
        || relative_root.as_std_path().components().any(|component| {
            !matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        bail!("private_evidence_root=blocked reason=invalid_workspace_path");
    }

    let canonical_candidate = canonical_workspace.join(&relative_root);
    let mut maybe_existing = Some(canonical_candidate.as_path());
    let existing_ancestor = loop {
        let Some(candidate) = maybe_existing else {
            bail!("private_evidence_root=blocked reason=missing_workspace_ancestor");
        };
        if candidate.exists() {
            break candidate;
        }
        maybe_existing = candidate.parent();
    };
    let canonical_ancestor = fs::canonicalize(existing_ancestor.as_std_path())
        .context("failed to resolve private evidence ancestor")?;
    if !canonical_ancestor.starts_with(canonical_workspace.as_std_path()) {
        bail!("private_evidence_root=blocked reason=symlink_escape");
    }

    let status = Command::new("git")
        .current_dir(canonical_workspace.as_std_path())
        .args(["check-ignore", "--quiet", "--"])
        .arg(relative_root.as_std_path())
        .status()
        .context("failed to verify private evidence ignore admission")?;
    if !status.success() {
        bail!("private_evidence_root=blocked reason=not_repo_ignored");
    }
    Ok(())
}

impl FlashEnvironment for LocalFlashEnvironment {
    fn build_package(&self) -> Result<()> {
        let status = Command::new("bazel")
            .current_dir(self.workspace_dir.as_std_path())
            .arg("build")
            .arg(PACKAGE_BUILD_TARGET)
            .status()
            .context("failed to run bazel build for firmware package")?;
        if !status.success() {
            bail!("{PACKAGE_BUILD_DISPLAY} failed with {status}");
        }

        Ok(())
    }

    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        timeout_seconds: u64,
        redaction_mode: EvidenceRedactionMode,
        create_new: bool,
    ) -> Result<CaptureProcessResult> {
        let bytes = self.receive_only(command_spec, timeout_seconds)?;
        let raw = String::from_utf8_lossy(&bytes);
        let sanitized = sanitize_evidence_text(&raw, redaction_mode);
        if create_new {
            evidence::write_dual_private_text(log_path, &sanitized)?;
        } else {
            self.write_evidence(log_path, &sanitized)?;
        }
        let result = CaptureProcessResult {
            status: CaptureProcessStatus::TimedOut,
        };
        if let Ok(stage_root) = env::var("PHASE35_FLASH_STAGE_ROOT") {
            if !stage_root.is_empty() {
                let private_log = fs::read(log_path.as_std_path())?;
                let observed_bytes = !private_log.is_empty();
                let launched = !matches!(result.status, CaptureProcessStatus::SpawnFailed);
                let connected = launched && observed_bytes;
                let completed = connected
                    && !matches!(
                        result.status,
                        CaptureProcessStatus::SpawnFailed | CaptureProcessStatus::ExitedFailure(_)
                    );
                let metrics = serde_json::json!({
                    "schema_version": PHASE35_FLASH_SCHEMA,
                    "stage": "monitor",
                    "tool_version_valid": true,
                    "launched": launched,
                    "connected": connected,
                    "device_info_complete": connected,
                    "transfer_started": connected,
                    "completed": completed,
                    "duration_millis": timeout_seconds.saturating_mul(1_000),
                });
                let stage_root = Utf8Path::new(&stage_root);
                fs::create_dir_all(stage_root.as_std_path())?;
                set_private_directory_mode(stage_root)?;
                let monitor_log = stage_root.join("monitor.private.log");
                let monitor_metrics = stage_root.join("monitor.metrics.json");
                write_private_new_bytes(&monitor_log, &private_log)?;
                let mut encoded = serde_json::to_vec_pretty(&metrics)?;
                encoded.push(b'\n');
                write_private_new_bytes(&monitor_metrics, &encoded)?;
            }
        }
        Ok(result)
    }

    fn approve_private_evidence_root(&self, path: &Utf8Path) -> Result<()> {
        approve_local_private_evidence_root(&self.workspace_dir, path)
    }

    fn bazel_bin(&self) -> Result<Utf8PathBuf> {
        let output = Command::new("bazel")
            .current_dir(self.workspace_dir.as_std_path())
            .arg("info")
            .arg("bazel-bin")
            .output()
            .context("failed to run bazel info bazel-bin")?;
        command_output_to_string(output, "bazel info bazel-bin").map(Utf8PathBuf::from)
    }

    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }

    fn read_to_string(&self, path: &Utf8Path) -> Result<String> {
        fs::read_to_string(path.as_std_path()).with_context(|| format!("failed to read {path}"))
    }

    fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>> {
        fs::read(path.as_std_path()).with_context(|| format!("failed to read {path}"))
    }

    fn current_provenance(&self) -> Result<BuildProvenance> {
        let output = Command::new("cargo")
            .current_dir(self.workspace_dir.as_std_path())
            .args([
                "run",
                "--quiet",
                "-p",
                "xtask",
                "--",
                "build-identity-status",
            ])
            .output()
            .context("failed to run canonical build identity status command")?;
        let status = command_output_to_string(output, "build identity status command")?;
        BuildProvenance::parse_workspace_status(&status)
            .context("current workspace build identity is invalid")
    }

    fn list_ports(&self) -> Result<String> {
        discover_usb_ports()
            .map(|ports| ports.join("\n"))
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create directory {parent}"))?;
        }

        fs::write(path.as_std_path(), contents).with_context(|| format!("failed to write {path}"))
    }

    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()> {
        let python = self.nvs_generator_python()?;
        let output = Command::new(python.as_std_path())
            .arg("-m")
            .arg("esp_idf_nvs_partition_gen")
            .arg("generate")
            .arg(csv_path.as_str())
            .arg(bin_path.as_str())
            .arg(size)
            .output()
            .context("failed to run ESP-IDF NVS partition generator")?;
        if !output.status.success() {
            bail!(
                "ESP-IDF NVS partition generator failed: {}",
                command_stderr_or_status(&output)
            );
        }

        Ok(())
    }

    fn begin_usb_session(&self, operation: UsbOperation, port: &str) -> Result<()> {
        if self.usb_session.borrow().is_some() {
            return Ok(());
        }
        let trace_root = self
            .workspace_dir
            .join("scratch/device-sessions")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
        let mut session = UsbSession::acquire(operation, port, trace_root.as_std_path())
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let version_validation = session
            .run_espflash_probe(
                self.espflash_bin.as_std_path(),
                &["--version".to_owned()],
                Duration::from_secs(10),
            )
            .map_err(|error| anyhow::anyhow!("{error}"))
            .and_then(|output| {
                let observed = String::from_utf8(output.stdout)
                    .context("supervised espflash version output was not valid UTF-8")?;
                if observed.trim() != self.espflash_version {
                    bail!("espflash_version_mismatch expected={ESPFLASH_EXPECTED_VERSION}");
                }
                Ok(())
            });
        if let Err(primary_error) = version_validation {
            return match session.finish() {
                Ok(_) => Err(primary_error),
                Err(_) => Err(primary_error.context("cleanup_failure=secondary")),
            };
        }
        *self.usb_session.borrow_mut() = Some(session);
        Ok(())
    }

    fn execute(&self, command_spec: &CommandSpec) -> Result<()> {
        self.execute_with_output(command_spec).map(|_| ())
    }

    fn execute_with_output(&self, command_spec: &CommandSpec) -> Result<Vec<u8>> {
        if command_spec.program != "espflash" {
            bail!("unsupported command program: {}", command_spec.program);
        }

        let mut session_slot = self.usb_session.borrow_mut();
        let Some(session) = session_slot.as_mut() else {
            bail!("cleanup_failed: USB effect attempted without a repository session");
        };
        let args = command_with_port(command_spec, session.port())?;
        let output = session
            .run_espflash(
                self.espflash_bin.as_std_path(),
                &args,
                Duration::from_secs(360),
            )
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let mut combined = output.stdout;
        combined.extend_from_slice(&output.stderr);
        Ok(combined)
    }

    fn last_usb_command_diagnostic(&self) -> Option<UsbCommandDiagnostic> {
        self.usb_session
            .borrow()
            .as_ref()
            .and_then(UsbSession::last_command_diagnostic)
    }

    fn receive_only(&self, command_spec: &CommandSpec, timeout_seconds: u64) -> Result<Vec<u8>> {
        if command_spec.program != "bitaxe-receive-only" {
            bail!("unsupported receive-only command program");
        }
        let mut session_slot = self.usb_session.borrow_mut();
        let Some(session) = session_slot.as_mut() else {
            bail!("cleanup_failed: monitor attempted without a repository session");
        };
        emit_line("usb_reader", "admitted")?;
        let output = session
            .observe_receive_only(Duration::from_secs(timeout_seconds))
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        Ok(output.bytes)
    }

    fn campaign_lease_id(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = (timestamp as u64) ^ ((timestamp >> 64) as u64);
        (folded ^ u64::from(std::process::id())).max(1)
    }

    fn receive_campaign_until(
        &self,
        admission: CampaignAdmission,
        expected_runtime: ExpectedRuntimeAttestationIdentity,
        evidence_root: &Utf8Path,
        capture_limit: CampaignCaptureLimit,
    ) -> Result<campaign::network::CampaignObservationCapture> {
        let mut session_slot = self.usb_session.borrow_mut();
        let Some(session) = session_slot.as_mut() else {
            bail!("cleanup_failed: campaign observation attempted without a repository session");
        };
        let mut analyzer = CampaignSerialAnalyzer::new(admission);
        let mut network = campaign::network::CampaignNetworkCoordinator::new(
            admission,
            expected_runtime,
            evidence_root.to_owned(),
        );
        let mut observe = |chunk: &[u8]| {
            analyzer.observe_chunk(chunk);
            network.observe_serial_chunk(chunk);
            analyzer.terminal_consumed() || network.should_stop()
        };
        match capture_limit {
            CampaignCaptureLimit::Bounded(timeout_seconds) => session
                .observe_receive_only_ephemeral_chunks_until(
                    Duration::from_secs(timeout_seconds),
                    &mut observe,
                ),
        }
        .map_err(|error| anyhow::anyhow!("{error}"))?;
        let serial = analyzer.finish();
        let network = network.finish(&serial);
        Ok(campaign::network::CampaignObservationCapture { serial, network })
    }

    fn receive_input_uat(&self, stop: &mut dyn FnMut(&[u8]) -> bool) -> Result<MonitorOutput> {
        let mut session_slot = self.usb_session.borrow_mut();
        let Some(session) = session_slot.as_mut() else {
            bail!("cleanup_failed: input UAT attempted without a repository session");
        };
        emit_line("usb_reader", "admitted")?;
        session
            .observe_receive_only_ephemeral_chunks_operator_gated(stop)
            .map_err(|error| anyhow::anyhow!("{error}"))
    }

    fn finish_usb_session(&self) -> Result<()> {
        let maybe_session = self.usb_session.borrow_mut().take();
        let Some(session) = maybe_session else {
            return Ok(());
        };
        let operation = session.operation();
        let ready = session
            .finish()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if operation == UsbOperation::Detect {
            emit_line("port", &ready.port)?;
        }
        emit_line("usb_session", "ready")
    }

    fn device_effect_state(&self) -> UsbDeviceEffectState {
        self.usb_session
            .borrow()
            .as_ref()
            .map_or(UsbDeviceEffectState::None, UsbSession::device_effect_state)
    }

    fn phase35_stage_readiness_gate(&self, _stage: &str, _port: &str) -> Result<()> {
        let Ok(stage_root) = env::var("PHASE35_FLASH_STAGE_ROOT") else {
            return Ok(());
        };
        if stage_root.is_empty() {
            return Ok(());
        }
        bail!("phase35_stage_readiness=blocked reason=legacy_gate_removed stage_root={stage_root}")
    }

    fn firmware_commit(&self) -> String {
        maybe_git_output(&self.workspace_dir, ["rev-parse", "HEAD"])
            .unwrap_or_else(|| UNAVAILABLE.to_owned())
    }

    fn reference_commit(&self) -> String {
        maybe_git_output(
            &self.workspace_dir,
            ["-C", "reference/esp-miner", "rev-parse", "HEAD"],
        )
        .unwrap_or_else(|| UNAVAILABLE.to_owned())
    }

    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        let maybe_parent = path.parent();
        if let Some(parent) = maybe_parent {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create evidence directory {parent}"))?;
        }

        fs::write(path.as_std_path(), contents)
            .with_context(|| format!("failed to write evidence {path}"))
    }
}

impl LocalFlashEnvironment {
    pub(crate) fn nvs_generator_python(&self) -> Result<Utf8PathBuf> {
        if let Ok(path) = env::var("ESP_IDF_NVS_PYTHON") {
            if !path.is_empty() {
                return Ok(Utf8PathBuf::from(path));
            }
        }

        let candidate = self.workspace_dir.join(NVS_GENERATOR_PYTHON_RELATIVE_PATH);
        if !candidate.is_file() {
            bail!(
                "ESP-IDF NVS generator python not found at {candidate}; run just bootstrap-esp or build firmware once"
            );
        }

        Ok(candidate)
    }
}
