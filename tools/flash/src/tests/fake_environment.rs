#[derive(Debug)]
struct ObservedFlash {
    path: Utf8PathBuf,
    bytes: Vec<u8>,
    unix_mode: Option<u32>,
}

#[derive(Debug)]
struct FakeFlashEnvironment {
    ports: String,
    workspace_dir: Utf8PathBuf,
    executed_commands: RefCell<Vec<CommandSpec>>,
    captured_commands: RefCell<Vec<CommandSpec>>,
    generated_nvs_partitions: RefCell<Vec<(Utf8PathBuf, Utf8PathBuf, String)>>,
    capture_status: CaptureProcessStatus,
    log_contents: String,
    current_provenance: BuildProvenance,
    source_replacement: Option<(Utf8PathBuf, Vec<u8>)>,
    execute_failure: bool,
    snapshot_write_failure: bool,
    list_ports_calls: Cell<usize>,
    read_string_paths: RefCell<Vec<Utf8PathBuf>>,
    created_snapshot_paths: RefCell<Vec<Utf8PathBuf>>,
    observed_flash: RefCell<Vec<ObservedFlash>>,
    private_root_admitted: bool,
    private_root_admission_calls: Cell<usize>,
    phase35_stage_gates: RefCell<Vec<(String, String)>>,
}

impl Default for FakeFlashEnvironment {
    fn default() -> Self {
        Self::with_ports("/dev/cu.usbmodem101 USB JTAG")
    }
}

impl FakeFlashEnvironment {
    fn with_ports(ports: &str) -> Self {
        Self {
            ports: ports.to_owned(),
            workspace_dir: Utf8PathBuf::from_path_buf(env::current_dir().expect("current dir"))
                .expect("utf8 current dir"),
            executed_commands: RefCell::new(Vec::new()),
            captured_commands: RefCell::new(Vec::new()),
            generated_nvs_partitions: RefCell::new(Vec::new()),
            capture_status: CaptureProcessStatus::ExitedSuccess,
            log_contents: trusted_monitor_log(),
            current_provenance: BuildProvenance::new(
                "0.1.0",
                SOURCE_COMMIT,
                false,
                None::<&str>,
                REFERENCE_COMMIT,
            )
            .expect("default provenance"),
            source_replacement: None,
            execute_failure: false,
            snapshot_write_failure: false,
            list_ports_calls: Cell::new(0),
            read_string_paths: RefCell::new(Vec::new()),
            created_snapshot_paths: RefCell::new(Vec::new()),
            observed_flash: RefCell::new(Vec::new()),
            private_root_admitted: true,
            private_root_admission_calls: Cell::new(0),
            phase35_stage_gates: RefCell::new(Vec::new()),
        }
    }

    fn executed_commands(&self) -> Vec<CommandSpec> {
        self.executed_commands.borrow().clone()
    }

    fn captured_commands(&self) -> Vec<CommandSpec> {
        self.captured_commands.borrow().clone()
    }

    fn generated_nvs_partitions(&self) -> Vec<(Utf8PathBuf, Utf8PathBuf, String)> {
        self.generated_nvs_partitions.borrow().clone()
    }

    fn with_capture_status(mut self, capture_status: CaptureProcessStatus) -> Self {
        self.capture_status = capture_status;
        self
    }

    fn with_log_contents(mut self, log_contents: &str) -> Self {
        self.log_contents = log_contents.to_owned();
        self
    }

    fn with_workspace_dir(mut self, workspace_dir: Utf8PathBuf) -> Self {
        self.workspace_dir = workspace_dir;
        self
    }

    fn with_current_provenance(mut self, current_provenance: BuildProvenance) -> Self {
        self.current_provenance = current_provenance;
        self
    }

    fn with_source_replacement(mut self, path: Utf8PathBuf, bytes: Vec<u8>) -> Self {
        self.source_replacement = Some((path, bytes));
        self
    }

    fn with_execute_failure(mut self) -> Self {
        self.execute_failure = true;
        self
    }

    fn with_snapshot_write_failure(mut self) -> Self {
        self.snapshot_write_failure = true;
        self
    }

    fn with_private_root_rejected(mut self) -> Self {
        self.private_root_admitted = false;
        self
    }

    fn private_root_admission_calls(&self) -> usize {
        self.private_root_admission_calls.get()
    }

    fn created_snapshot_paths(&self) -> std::cell::Ref<'_, Vec<Utf8PathBuf>> {
        self.created_snapshot_paths.borrow()
    }

    fn list_ports_calls(&self) -> usize {
        self.list_ports_calls.get()
    }

    fn read_string_paths(&self) -> std::cell::Ref<'_, Vec<Utf8PathBuf>> {
        self.read_string_paths.borrow()
    }

    fn observed_flashes(&self) -> std::cell::Ref<'_, Vec<ObservedFlash>> {
        self.observed_flash.borrow()
    }

    fn phase35_stage_gates(&self) -> Vec<(String, String)> {
        self.phase35_stage_gates.borrow().clone()
    }
}

impl FlashEnvironment for FakeFlashEnvironment {
    fn build_package(&self) -> Result<()> {
        Ok(())
    }

    fn bazel_bin(&self) -> Result<Utf8PathBuf> {
        Ok(Utf8PathBuf::from("/tmp/bazel-bin"))
    }

    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }

    fn read_to_string(&self, path: &Utf8Path) -> Result<String> {
        self.read_string_paths.borrow_mut().push(path.to_owned());
        std::fs::read_to_string(path.as_std_path())
            .with_context(|| format!("failed to read fake manifest {path}"))
    }

    fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>> {
        std::fs::read(path.as_std_path())
            .with_context(|| format!("failed to read fake artifact {path}"))
    }

    fn create_admitted_execution_snapshot(
        &self,
        bytes: &[u8],
    ) -> Result<AdmittedExecutionSnapshot> {
        if self.snapshot_write_failure {
            bail!("identity_admission=blocked reason=execution_snapshot_write_failed");
        }
        let snapshot = AdmittedExecutionSnapshot::materialize(bytes)?;
        self.created_snapshot_paths
            .borrow_mut()
            .push(snapshot.path().to_owned());
        Ok(snapshot)
    }

    fn approve_private_evidence_root(&self, _path: &Utf8Path) -> Result<()> {
        self.private_root_admission_calls
            .set(self.private_root_admission_calls.get().saturating_add(1));
        if !self.private_root_admitted {
            bail!("private evidence root rejected by fixture");
        }
        Ok(())
    }

    fn current_provenance(&self) -> Result<BuildProvenance> {
        Ok(self.current_provenance.clone())
    }

    fn list_ports(&self) -> Result<String> {
        self.list_ports_calls
            .set(self.list_ports_calls.get().saturating_add(1));
        Ok(self.ports.clone())
    }

    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create fake file dir");
        }
        std::fs::write(path.as_std_path(), contents).expect("write fake file");
        Ok(())
    }

    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()> {
        self.generated_nvs_partitions.borrow_mut().push((
            csv_path.to_owned(),
            bin_path.to_owned(),
            size.to_owned(),
        ));
        if let Some(parent) = bin_path.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create fake nvs dir");
        }
        std::fs::write(bin_path.as_std_path(), b"nvs-image").expect("write fake nvs image");
        Ok(())
    }

    fn begin_usb_session(&self, _operation: UsbOperation, _port: &str) -> Result<()> {
        Ok(())
    }

    fn execute(&self, command_spec: &CommandSpec) -> Result<()> {
        self.executed_commands
            .borrow_mut()
            .push(command_spec.clone());
        if command_spec.args.first().map(String::as_str) == Some("write-bin")
            && command_spec.args.iter().any(|argument| argument == "0x0")
        {
            if let Some((path, bytes)) = &self.source_replacement {
                std::fs::write(path.as_std_path(), bytes).expect("replace package source");
            }
            let path = Utf8PathBuf::from(
                command_spec
                    .args
                    .last()
                    .expect("full flash command image path"),
            );
            let bytes = std::fs::read(path.as_std_path()).expect("read executed image");
            #[cfg(unix)]
            let unix_mode = {
                use std::os::unix::fs::PermissionsExt;
                Some(
                    std::fs::metadata(path.as_std_path())
                        .expect("executed image metadata")
                        .permissions()
                        .mode()
                        & 0o777,
                )
            };
            #[cfg(not(unix))]
            let unix_mode = None;
            self.observed_flash.borrow_mut().push(ObservedFlash {
                path,
                bytes,
                unix_mode,
            });
        }
        if self.execute_failure {
            bail!("sentinel child failure");
        }
        Ok(())
    }

    fn receive_only(
        &self,
        command_spec: &CommandSpec,
        _timeout_seconds: u64,
    ) -> Result<Vec<u8>> {
        self.executed_commands
            .borrow_mut()
            .push(command_spec.clone());
        if self.execute_failure {
            bail!("sentinel receive-only failure");
        }
        Ok(self.log_contents.as_bytes().to_vec())
    }

    fn finish_usb_session(&self) -> Result<()> {
        Ok(())
    }

    fn phase35_stage_readiness_gate(&self, stage: &str, port: &str) -> Result<()> {
        self.phase35_stage_gates
            .borrow_mut()
            .push((stage.to_owned(), port.to_owned()));
        Ok(())
    }

    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        _timeout_seconds: u64,
        redaction_mode: EvidenceRedactionMode,
        create_new: bool,
    ) -> Result<CaptureProcessResult> {
        self.captured_commands
            .borrow_mut()
            .push(command_spec.clone());
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create fake log dir");
        }
        let sanitized = sanitize_evidence_text(&self.log_contents, redaction_mode);
        if create_new {
            evidence::write_dual_private_text(log_path, &sanitized)
                .expect("write fake private monitor log");
        } else {
            std::fs::write(log_path.as_std_path(), sanitized).expect("write fake monitor log");
        }
        Ok(CaptureProcessResult {
            status: self.capture_status.clone(),
        })
    }

    fn firmware_commit(&self) -> String {
        "0123456789abcdef0123456789abcdef01234567".to_owned()
    }

    fn reference_commit(&self) -> String {
        "abcdef012345abcdef012345abcdef012345abcd".to_owned()
    }

    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create fake evidence dir");
        }
        std::fs::write(path.as_std_path(), contents).expect("write fake evidence");
        Ok(())
    }
}
