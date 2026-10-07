impl FakeFlashEnvironment {
    fn with_ports(ports: &str) -> Self {
        Self {
            ports: ports.to_owned(),
            workspace_dir: Utf8PathBuf::from_path_buf(env::current_dir().expect("current dir"))
                .expect("utf8 current dir"),
            capture_lifecycle: RefCell::new(Vec::new()),
            executed_commands: RefCell::new(Vec::new()),
            captured_commands: RefCell::new(Vec::new()),
            generated_nvs_partitions: RefCell::new(Vec::new()),
            capture_status: CaptureProcessStatus::ExitedSuccess,
            log_contents: trusted_monitor_log(),
            maybe_campaign_bytes: None,
            core_dump_reads: RefCell::new(std::collections::VecDeque::new()),
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
            maybe_execute_failure_command: None,
            maybe_execute_failure_offset: None,
            snapshot_write_failure: false,
            list_ports_calls: Cell::new(0),
            physical_inspection_ports: RefCell::new(Vec::new()),
            session_port: RefCell::new(String::new()),
            session_physical_digest: RefCell::new("6".repeat(64)),
            maybe_rom_reenumeration: None,
            read_string_paths: RefCell::new(Vec::new()),
            written_files: RefCell::new(Vec::new()),
            created_snapshot_paths: RefCell::new(Vec::new()),
            observed_flash: RefCell::new(Vec::new()),
            private_root_admitted: true,
            private_root_admission_calls: Cell::new(0),
            phase35_stage_gates: RefCell::new(Vec::new()),
            campaign_lease_id: 42,
            campaign_observations: RefCell::new(Vec::new()),
            input_uat_chunks: Vec::new(),
            input_uat_interrupted: false,
            cleanup_calls: Cell::new(0),
            cleanup_failure: false,
            application_exit_write_counts: RefCell::new(Vec::new()),
            application_exit_failure: false,
            installed_session_calls: Cell::new(0),
            maybe_installed_bytes: None,
            last_usb_command_diagnostic: RefCell::new(None),
            built_web_ui_variants: RefCell::new(Vec::new()),
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

    fn with_campaign_bytes(mut self, campaign_bytes: Vec<u8>) -> Self {
        self.maybe_campaign_bytes = Some(campaign_bytes);
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

    fn with_execute_failure_offset(mut self, offset: &str) -> Self {
        self.maybe_execute_failure_offset = Some(offset.to_owned());
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

    fn written_files(&self) -> std::cell::Ref<'_, Vec<(Utf8PathBuf, String)>> {
        self.written_files.borrow()
    }

    fn observed_flashes(&self) -> std::cell::Ref<'_, Vec<ObservedFlash>> {
        self.observed_flash.borrow()
    }

    fn phase35_stage_gates(&self) -> Vec<(String, String)> {
        self.phase35_stage_gates.borrow().clone()
    }

    fn campaign_observations(&self) -> Vec<(MiningCampaignStage, CampaignCaptureLimit)> {
        self.campaign_observations.borrow().clone()
    }

    fn cleanup_calls(&self) -> usize {
        self.cleanup_calls.get()
    }

    fn with_cleanup_failure(mut self) -> Self {
        self.cleanup_failure = true;
        self
    }

    fn with_input_uat_chunks(mut self, chunks: Vec<Vec<u8>>) -> Self {
        self.input_uat_chunks = chunks;
        self
    }

    fn with_input_uat_interrupted(mut self) -> Self {
        self.input_uat_interrupted = true;
        self
    }
}
