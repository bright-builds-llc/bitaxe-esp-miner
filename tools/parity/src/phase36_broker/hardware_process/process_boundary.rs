use super::*;

pub(super) struct ProcessTransactionBoundary {
    attempt_child: Utf8PathBuf,
    candidate_output: Utf8PathBuf,
    wifi_credentials: Utf8PathBuf,
    effect_adapter: Utf8PathBuf,
    handle: AttemptHandle,
    capture_timeout_seconds: u64,
    maybe_ledger: Option<PrivateAppendOnlyLedger>,
    records: Vec<Phase36LedgerRecord>,
    maybe_port: Option<String>,
    maybe_origin: Option<String>,
    pub(super) interval_start_millis: u64,
    interval_end_millis: u64,
}

impl ProcessTransactionBoundary {
    pub(super) fn load(
        private_parent: &Utf8Path,
        attempt_handle_file: &Utf8Path,
        candidate_output: &Utf8Path,
        wifi_credentials: &Utf8Path,
        capture_timeout_seconds: u64,
    ) -> Result<Self, Phase36HardwareTransactionError> {
        validate_private_directory(private_parent)?;
        validate_private_file(attempt_handle_file)?;
        if candidate_output.exists() {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        let bytes = fs::read(attempt_handle_file)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        let handle = serde_json::from_slice::<AttemptHandle>(&bytes)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        if handle.schema_version != "phase36-attempt-handle-v2"
            || !valid_child_name(&handle.child_name)
            || !valid_digest(&handle.capability_digest)
            || !valid_commit(&handle.source_commit)
            || !valid_commit(&handle.reference_commit)
            || !valid_digest(&handle.evaluator_identity_digest)
            || handle.target != "xtensa-esp32s3-espidf"
            || handle.board != "205"
            || handle.asic != "BM1366"
            || !valid_digest(&handle.manifest_digest)
            || !valid_digest(&handle.firmware_elf_digest)
            || !valid_digest(&handle.executable_image_digest)
            || !valid_digest(&handle.factory_image_digest)
            || !valid_digest(&handle.package_identity_digest)
        {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        let attempt_child = private_parent.join(&handle.child_name);
        if attempt_child.exists() {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        Ok(Self {
            attempt_child,
            candidate_output: candidate_output.to_owned(),
            wifi_credentials: wifi_credentials.to_owned(),
            effect_adapter: resolve_effect_adapter()?,
            handle,
            capture_timeout_seconds,
            maybe_ledger: None,
            records: Vec::new(),
            maybe_port: None,
            maybe_origin: None,
            interval_start_millis: current_millis()?,
            interval_end_millis: 0,
        })
    }

    fn validate_exact_package(&self) -> Result<(), Phase36HardwareTransactionError> {
        if self.handle.evaluator_identity_digest
            != crate::phase36_evidence::current_phase36_evidence_evaluator_digest()
        {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::AdmissionFailed,
            ));
        }
        for (path, expected_digest) in [
            (&self.handle.manifest_path, &self.handle.manifest_digest),
            (
                &self.handle.firmware_elf_path,
                &self.handle.firmware_elf_digest,
            ),
            (
                &self.handle.executable_image_path,
                &self.handle.executable_image_digest,
            ),
            (
                &self.handle.factory_image_path,
                &self.handle.factory_image_digest,
            ),
        ] {
            if sha256_file(Utf8Path::new(path))?.as_str() != expected_digest {
                return Err(Phase36HardwareTransactionError::EffectFailed(
                    Phase36BrokerFailure::AdmissionFailed,
                ));
            }
        }
        let package_digest = sha256_hex(
            [
                b"phase36-package-identity-v1\\0".as_slice(),
                self.handle.source_commit.as_bytes(),
                b"\\0",
                self.handle.reference_commit.as_bytes(),
                b"\\0",
                self.handle.manifest_digest.as_bytes(),
                b"\\0",
                self.handle.firmware_elf_digest.as_bytes(),
                b"\\0",
                self.handle.executable_image_digest.as_bytes(),
                b"\\0",
                self.handle.factory_image_digest.as_bytes(),
            ]
            .concat(),
        );
        if package_digest != self.handle.package_identity_digest {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::AdmissionFailed,
            ));
        }
        Ok(())
    }

    fn run_detector(&mut self) -> Result<(), Phase36HardwareTransactionError> {
        let output = run_detector_process().map_err(|_| {
            Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::DetectorFailed)
        })?;
        if !output.status.success() {
            return Err(Phase36HardwareTransactionError::EffectFailed(
                Phase36BrokerFailure::DetectorFailed,
            ));
        }
        let port = parse_detector_port(&output.stdout).ok_or(
            Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::DetectorFailed),
        )?;
        let physical_identity_digest = sha256_hex(
            [
                b"phase36-physical-device-v1\0".as_slice(),
                port.as_bytes(),
                b"\0",
                &output.stderr,
            ]
            .concat(),
        );
        let facts = DetectorFacts {
            schema_version: "phase36-detector-facts-v1",
            board_category: "205",
            target: "xtensa-esp32s3-espidf",
            asic: "BM1366",
            candidate_count: 1,
            port: &port,
            physical_identity_digest: &physical_identity_digest,
        };
        write_new_private_json(&self.attempt_child.join("detector-facts.json"), &facts)?;
        validate_wifi_credentials(&self.wifi_credentials)?;
        self.maybe_port = Some(port);
        Ok(())
    }

    fn run_effect_classified(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
        let port = match (operation, self.maybe_port.as_deref()) {
            (Phase36AllowedOperation::Cleanup, None) => "unavailable",
            (_, Some(port)) => port,
            _ => {
                return Ok(classify_missing_effect_result(operation));
            }
        };
        let result_path = self
            .attempt_child
            .join(format!("effect-result-{}.json", operation_name(operation)));
        let mut command = Command::new("perl");
        command
            .arg("-e")
            .arg("alarm shift; exec @ARGV")
            .arg(MIN_WALL_CLOCK_TIMEOUT_SECONDS.to_string())
            .arg(&self.effect_adapter)
            .arg(format!("operation={}", operation_name(operation)))
            .arg(format!("board={}", 205))
            .arg(format!("port={port}"))
            .arg(format!("attempt-child={}", self.attempt_child))
            .arg(format!(
                "package-identity-digest={}",
                self.handle.package_identity_digest
            ))
            .arg(format!("manifest-path={}", self.handle.manifest_path))
            .arg(format!("manifest-digest={}", self.handle.manifest_digest))
            .arg(format!(
                "firmware-elf-path={}",
                self.handle.firmware_elf_path
            ))
            .arg(format!(
                "firmware-elf-digest={}",
                self.handle.firmware_elf_digest
            ))
            .arg(format!(
                "executable-image-path={}",
                self.handle.executable_image_path
            ))
            .arg(format!(
                "executable-image-digest={}",
                self.handle.executable_image_digest
            ))
            .arg(format!(
                "factory-image-path={}",
                self.handle.factory_image_path
            ))
            .arg(format!(
                "factory-image-digest={}",
                self.handle.factory_image_digest
            ))
            .arg(format!(
                "capture-timeout-seconds={}",
                self.capture_timeout_seconds
            ))
            .arg(format!(
                "wall-clock-timeout-seconds={MIN_WALL_CLOCK_TIMEOUT_SECONDS}"
            ))
            .arg(format!("result-path={result_path}"))
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        if operation == Phase36AllowedOperation::ExactPackageFlash {
            command.arg(format!("wifi-credentials={}", self.wifi_credentials));
        }
        if matches!(
            operation,
            Phase36AllowedOperation::ReadOnlySystemInfo
                | Phase36AllowedOperation::ReadOnlyWebSocket
                | Phase36AllowedOperation::ReadOnlyRetainedFacts
        ) {
            let origin = self.maybe_origin.as_deref().ok_or(
                Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::CaptureFailed),
            )?;
            command.arg(format!("trusted-origin={origin}"));
        }
        let Ok(output) = command.output() else {
            return Ok(classify_missing_effect_result(operation));
        };
        let Some(result) = maybe_read_effect_result(&result_path) else {
            return Ok(classify_missing_effect_result(operation));
        };
        let classified = classify_effect_result(
            result,
            operation,
            output.status.success(),
            &self.handle.package_identity_digest,
            &self.handle.factory_image_digest,
        );
        if operation == Phase36AllowedOperation::PassiveSerialObservation
            && matches!(classified, Phase36OperationResult::Completed { .. })
        {
            let stdout = String::from_utf8(output.stdout).map_err(|_| {
                Phase36HardwareTransactionError::EffectFailed(Phase36BrokerFailure::CaptureFailed)
            })?;
            let origins = stdout
                .lines()
                .filter_map(|line| line.strip_prefix("trusted_origin="))
                .filter(|origin| valid_origin(origin))
                .collect::<Vec<_>>();
            if origins.len() != 1 {
                return Ok(Phase36OperationResult::Failed {
                    failure: Phase36BrokerFailure::CaptureFailed,
                    maybe_partial_device_effect: None,
                });
            }
            self.maybe_origin = Some(origins[0].to_owned());
        }
        Ok(classified)
    }

    fn write_seal(
        &mut self,
        disposition: Phase36HardwareDisposition,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.interval_end_millis = current_millis()?.max(self.interval_start_millis + 1);
        let (status, first_failure, secondary_failure, recovery_disposition) = match disposition {
            Phase36HardwareDisposition::SealedEligible => (
                "sealed_eligible",
                None,
                None,
                Phase36RecoveryDisposition::NotRequired,
            ),
            Phase36HardwareDisposition::SealedNonPromotion {
                first_failure,
                secondary_failure,
                recovery_disposition,
            } => (
                "sealed_non_promotion",
                Some(first_failure),
                secondary_failure,
                recovery_disposition,
            ),
        };
        let mut maybe_candidate_digest = None;
        let mut maybe_private_capture_digest = None;
        if disposition == Phase36HardwareDisposition::SealedEligible {
            let private_capture = self.attempt_child.join("private-capture.json");
            let broker = BrokerCaptureDocument {
                observation_source: CaptureObservationSource::IndependentBrokerLedger,
                capability_digest: self.handle.capability_digest.clone(),
                package_digest: self.handle.package_identity_digest.clone(),
                same_physical_device_observed: true,
                interval_start_millis: self.interval_start_millis,
                interval_end_millis: self.interval_end_millis,
                records: self.records.clone(),
            };
            replace_broker_document(&private_capture, broker)
                .map_err(|_| Phase36HardwareTransactionError::Seal)?;
            let candidate =
                write_candidate_from_private_file(&private_capture, &self.candidate_output)
                    .map_err(|_| Phase36HardwareTransactionError::Seal)?;
            maybe_candidate_digest = Some(candidate.candidate_digest);
            maybe_private_capture_digest = Some(candidate.private_capture_digest);
        }
        let seal = AttemptSeal {
            schema_version: "phase36-attempt-seal-v2",
            status,
            first_failure,
            secondary_failure,
            recovery_disposition,
            capability_digest: &self.handle.capability_digest,
            package_identity_digest: &self.handle.package_identity_digest,
            candidate_digest: maybe_candidate_digest.as_deref(),
            private_capture_digest: maybe_private_capture_digest.as_deref(),
        };
        write_new_private_json(&self.attempt_child.join("seal.json"), &seal)
    }
}

impl Phase36HardwareTransactionBoundary for ProcessTransactionBoundary {
    fn prepare_private_attempt(&mut self) -> Result<(), Phase36HardwareTransactionError> {
        DirBuilder::new()
            .mode(0o700)
            .create(&self.attempt_child)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        let metadata = fs::symlink_metadata(&self.attempt_child)
            .map_err(|_| Phase36HardwareTransactionError::PrivateAttempt)?;
        if metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o777 != 0o700 {
            return Err(Phase36HardwareTransactionError::PrivateAttempt);
        }
        self.maybe_ledger = Some(
            PrivateAppendOnlyLedger::create(&self.attempt_child.join("effect-ledger.jsonl"))
                .map_err(Phase36HardwareTransactionError::Ledger)?,
        );
        Ok(())
    }

    fn record(
        &mut self,
        record: &Phase36LedgerRecord,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.maybe_ledger
            .as_mut()
            .ok_or(Phase36HardwareTransactionError::PrivateAttempt)?
            .append(record)?;
        self.records.push(record.clone());
        Ok(())
    }

    fn execute(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<(), Phase36HardwareTransactionError> {
        match operation {
            Phase36AllowedOperation::ExactPackageAdmission => self.validate_exact_package(),
            Phase36AllowedOperation::Board205DetectorProbe => self.run_detector(),
            _ => match self.run_effect_classified(operation)? {
                Phase36OperationResult::Completed { .. } => Ok(()),
                Phase36OperationResult::Failed { failure, .. } => {
                    Err(Phase36HardwareTransactionError::EffectFailed(failure))
                }
            },
        }
    }

    fn recovery_identity(
        &self,
    ) -> Result<Phase36RecoveryIdentity, Phase36HardwareTransactionError> {
        Phase36RecoveryIdentity::new(
            self.handle.package_identity_digest.clone(),
            self.handle.factory_image_digest.clone(),
        )
        .map_err(|_| Phase36HardwareTransactionError::InvalidRecoveryAuthority)
    }

    fn execute_classified(
        &mut self,
        operation: Phase36AllowedOperation,
    ) -> Result<Phase36OperationResult, Phase36HardwareTransactionError> {
        match operation {
            Phase36AllowedOperation::ExactPackageAdmission
            | Phase36AllowedOperation::Board205DetectorProbe => {
                let result = self.execute(operation);
                match result {
                    Ok(()) => Ok(Phase36OperationResult::Completed {
                        maybe_completed_device_effect: None,
                    }),
                    Err(Phase36HardwareTransactionError::EffectFailed(failure)) => {
                        Ok(Phase36OperationResult::Failed {
                            failure,
                            maybe_partial_device_effect: None,
                        })
                    }
                    Err(error) => Err(error),
                }
            }
            _ => self.run_effect_classified(operation),
        }
    }

    fn seal(
        &mut self,
        disposition: Phase36HardwareDisposition,
    ) -> Result<(), Phase36HardwareTransactionError> {
        self.maybe_ledger
            .as_mut()
            .ok_or(Phase36HardwareTransactionError::PrivateAttempt)?
            .seal()?;
        self.write_seal(disposition)
    }
}
