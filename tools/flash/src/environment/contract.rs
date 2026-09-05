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
    fn usb_profile(&self, _port: &str) -> Result<UsbProfile> {
        Ok(UsbProfile::SerialJtagRuntime)
    }
    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()>;
    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()>;
    fn begin_usb_session(&self, operation: UsbOperation, port: &str) -> Result<()>;
    fn prepare_application_exit(&self) -> Result<Utf8PathBuf> {
        bail!("application_exit=blocked reason=adapter_unavailable")
    }
    fn execute_application_exit(&self, _esptool: &Utf8Path) -> Result<InstalledApplicationExit> {
        bail!("application_exit=blocked reason=adapter_unavailable")
    }
    fn begin_installed_session(&self, _port: &str, _root: &Utf8Path) -> Result<()> {
        bail!("start_installed=blocked reason=adapter_unavailable")
    }
    fn observe_installed_runtime(&self) -> Result<UsbRebootLoopObservation> {
        bail!("start_installed=blocked reason=adapter_unavailable")
    }
    fn usb_physical_identity_digest(&self) -> Result<String>;
    fn current_usb_physical_identity_digest(&self, port: &str) -> Result<String>;
    fn execute(&self, command_spec: &CommandSpec) -> Result<()>;
    fn execute_esptool_write_flash(&self, command: &ManagedEsptoolWriteFlash) -> Result<()>;
    fn admit_flash_read(&self) -> Result<()> {
        Ok(())
    }
    fn execute_flash_read(&self, read: &ManagedFlashRead) -> Result<()>;
    fn restore_application_runtime(&self, esptool: &Utf8Path) -> Result<ProfileObservationCounts>;
    fn execute_owner_recovery_exit(
        &self,
        _esptool: &Utf8Path,
        _observation_seconds: u64,
    ) -> Result<OwnerRecoveryExitCapture> {
        bail!("owner_recovery=blocked reason=adapter_unavailable")
    }
    fn execute_owner_rom_probe(&self, command: &CommandSpec) -> Result<Vec<u8>> {
        self.execute_with_output(command)
    }
    fn exit_boot_chain_rom(&self, _esptool: &Utf8Path) -> Result<UsbProfile> {
        bail!("boot_chain=blocked reason=adapter_unavailable")
    }
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
    fn pushed_firmware_commit(&self) -> String {
        self.firmware_commit()
    }
    fn reference_commit(&self) -> String;
    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()>;
}
