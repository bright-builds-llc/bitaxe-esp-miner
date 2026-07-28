use std::fs;

use camino::Utf8Path;
use serde::Deserialize;

use super::filesystem::{write_new_private_file, CaptureFileError};
use super::{
    BrokerCaptureDocument, CaptureObservationSource, Phase36PrivateCaptureBundle,
    RuntimeIdentityCaptureDocuments, SubstantiveCaptureDocuments, PHASE36_PRIVATE_CAPTURE_SCHEMA,
};

#[derive(Debug)]
pub struct HardwareCaptureAssembly<'a> {
    pub attempt_child: &'a Utf8Path,
    pub manifest: &'a Utf8Path,
    pub manifest_digest: &'a str,
    pub firmware_elf_digest: &'a str,
    pub executable_image_digest: &'a str,
    pub factory_image_digest: &'a str,
    pub package_identity_digest: &'a str,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    source_commit: String,
    reference_commit: String,
}

#[derive(Debug, Deserialize)]
struct DetectorFacts {
    schema_version: String,
    board_category: String,
    target: String,
    asic: String,
    candidate_count: u8,
    physical_identity_digest: String,
}

pub fn assemble_hardware_capture(
    input: &HardwareCaptureAssembly<'_>,
) -> Result<(), CaptureFileError> {
    let manifest = read_json::<PackageManifest>(input.manifest)?;
    let detector = read_json::<DetectorFacts>(&input.attempt_child.join("detector-facts.json"))?;
    validate_detector(&detector)?;
    let api_json = read_json_value(&input.attempt_child.join("http-immediate/body"))?;
    let boot_session = string_field(&api_json, "bootSession", 32)?;
    let revision = u64_field(&api_json, "operatorSnapshotRevision")?;
    let websocket_document = fs::read_to_string(input.attempt_child.join("websocket.json"))
        .map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    let websocket_json = extract_unique_websocket_json(&websocket_document)?;
    if websocket_json != api_json {
        return Err(CaptureFileError::ClassificationFailed);
    }
    let monitor_document = read_monitor_document(input.attempt_child)?;
    require_retained_join(&monitor_document, &boot_session, revision)?;
    let boot_ordinal = unique_u64_token(
        &monitor_document,
        "runtime_boot_identity",
        "boot_ordinal",
        Some((&boot_session, "session")),
    )?;
    let reset_reason_category = unique_token(
        &monitor_document,
        "runtime_boot_identity",
        "reset_reason",
        Some((&boot_session, "session")),
    )?;
    let trusted_origin = unique_token(
        &monitor_document,
        "runtime_origin",
        "device_url",
        Some((&boot_session, "session")),
    )?;
    let source_commit = unique_prefixed_value(&monitor_document, "firmware_commit=")?;
    let reference_commit = unique_prefixed_value(&monitor_document, "reference_commit=")?;
    let app_elf_sha256 = unique_prefixed_value(&monitor_document, "app_elf_sha256=")?;
    if source_commit != manifest.source_commit
        || reference_commit != manifest.reference_commit
        || app_elf_sha256 != input.firmware_elf_digest
    {
        return Err(CaptureFileError::ClassificationFailed);
    }
    let api_compact =
        serde_json::to_string(&api_json).map_err(|_| CaptureFileError::OutputFailed)?;
    let exact_package_document = serde_json::json!({
        "schema_version": "phase36-runtime-package-v1",
        "source_commit": source_commit,
        "reference_commit": reference_commit,
        "manifest_digest": input.manifest_digest,
        "executable_image_digest": input.executable_image_digest,
        "factory_image_digest": input.factory_image_digest,
        "firmware_elf_digest": input.firmware_elf_digest,
        "package_digest": input.package_identity_digest,
    });
    let hardware_observation_document = serde_json::json!({
        "schema_version": "phase36-hardware-runtime-observation-v1",
        "board_category": "205",
        "target": "xtensa-esp32s3-espidf",
        "asic": "BM1366",
        "detector_candidate_count": detector.candidate_count,
        "same_physical_device": true,
        "physical_identity_digest": detector.physical_identity_digest,
        "boot_session": boot_session,
        "boot_ordinal": boot_ordinal,
        "reset_reason_category": reset_reason_category,
        "trusted_origin": trusted_origin,
        "source_commit": manifest.source_commit,
        "reference_commit": manifest.reference_commit,
        "app_elf_sha256": input.firmware_elf_digest,
    });
    let bundle = Phase36PrivateCaptureBundle {
        schema_version: PHASE36_PRIVATE_CAPTURE_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        substantive: SubstantiveCaptureDocuments {
            system_info_document: format!(
                "system_info_json: {api_compact}\noperator_snapshot_boot_session: {boot_session}\noperator_snapshot_revision: {revision}\n"
            ),
            websocket_document: format!(
                "live_websocket_json: {api_compact}\noperator_snapshot_boot_session: {boot_session}\noperator_snapshot_revision: {revision}\n"
            ),
            retained_document: format!(
                "operator_snapshot session={boot_session} revision={revision} redacted=true\nsubstantive_snapshot_json: {api_compact}\n"
            ),
        },
        runtime_identity: RuntimeIdentityCaptureDocuments {
            exact_package_document: serde_json::to_string(&exact_package_document)
                .map_err(|_| CaptureFileError::OutputFailed)?,
            request_document: String::new(),
            event_ledger_document: String::new(),
            private_result_document: String::new(),
            public_projection_document: String::new(),
            hardware_observation_document: Some(
                serde_json::to_string(&hardware_observation_document)
                    .map_err(|_| CaptureFileError::OutputFailed)?,
            ),
        },
        broker: BrokerCaptureDocument {
            observation_source: CaptureObservationSource::IndependentBrokerLedger,
            capability_digest: "0".repeat(64),
            package_digest: input.package_identity_digest.to_owned(),
            same_physical_device_observed: true,
            interval_start_millis: 1,
            interval_end_millis: 2,
            records: Vec::new(),
        },
    };
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|_| CaptureFileError::OutputFailed)?;
    write_new_private_file(&input.attempt_child.join("private-capture.json"), &bytes)
}

fn validate_detector(detector: &DetectorFacts) -> Result<(), CaptureFileError> {
    if detector.schema_version != "phase36-detector-facts-v1"
        || detector.board_category != "205"
        || detector.target != "xtensa-esp32s3-espidf"
        || detector.asic != "BM1366"
        || detector.candidate_count != 1
        || !valid_hex(&detector.physical_identity_digest, 64)
    {
        return Err(CaptureFileError::ClassificationFailed);
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Utf8Path) -> Result<T, CaptureFileError> {
    let bytes = fs::read(path).map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    serde_json::from_slice(&bytes).map_err(|_| CaptureFileError::PrivateInputInvalid)
}

fn read_json_value(path: &Utf8Path) -> Result<serde_json::Value, CaptureFileError> {
    read_json(path)
}

fn read_monitor_document(attempt_child: &Utf8Path) -> Result<String, CaptureFileError> {
    for relative in [
        "passive-serial-observation/monitor.classifier-input.log",
        "passive-serial-observation/flash-monitor.classifier-input.log",
    ] {
        let path = attempt_child.join(relative);
        if path.is_file() {
            return fs::read_to_string(path).map_err(|_| CaptureFileError::PrivateInputInvalid);
        }
    }
    Err(CaptureFileError::PrivateInputInvalid)
}

fn extract_unique_websocket_json(document: &str) -> Result<serde_json::Value, CaptureFileError> {
    let values = document
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name.starts_with("websocket_frame_")
                && name != "websocket_frame_status"
                && value.starts_with('{'))
            .then_some(value)
        })
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CaptureFileError::PrivateInputInvalid)?;
    let Some(first) = values.first() else {
        return Err(CaptureFileError::ClassificationFailed);
    };
    if values.iter().any(|value| value != first) {
        return Err(CaptureFileError::ClassificationFailed);
    }
    Ok(first.clone())
}

fn require_retained_join(
    document: &str,
    boot_session: &str,
    revision: u64,
) -> Result<(), CaptureFileError> {
    let marker =
        format!("operator_snapshot session={boot_session} revision={revision} redacted=true");
    let health = format!(
        "runtime_health boot_session={boot_session} operator_snapshot_revision={revision} "
    );
    if document.matches(&marker).count() != 1 || document.matches(&health).count() != 1 {
        return Err(CaptureFileError::ClassificationFailed);
    }
    Ok(())
}

fn unique_prefixed_value(document: &str, prefix: &str) -> Result<String, CaptureFileError> {
    let values = document
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .find_map(|word| word.strip_prefix(prefix))
        })
        .collect::<Vec<_>>();
    unique_value(values)
}

fn unique_token(
    document: &str,
    record_prefix: &str,
    token: &str,
    maybe_join: Option<(&str, &str)>,
) -> Result<String, CaptureFileError> {
    let values = document
        .lines()
        .filter(|line| line.contains(record_prefix))
        .filter(|line| {
            maybe_join.is_none_or(|(expected, join_token)| {
                maybe_token_value(line, join_token) == Some(expected)
            })
        })
        .filter_map(|line| maybe_token_value(line, token))
        .collect::<Vec<_>>();
    unique_value(values)
}

fn unique_u64_token(
    document: &str,
    record_prefix: &str,
    token: &str,
    maybe_join: Option<(&str, &str)>,
) -> Result<u64, CaptureFileError> {
    unique_token(document, record_prefix, token, maybe_join)?
        .parse()
        .map_err(|_| CaptureFileError::PrivateInputInvalid)
}

fn maybe_token_value<'a>(line: &'a str, token: &str) -> Option<&'a str> {
    let prefix = format!("{token}=");
    line.split_whitespace()
        .find_map(|word| word.strip_prefix(&prefix))
}

fn unique_value(values: Vec<&str>) -> Result<String, CaptureFileError> {
    let Some(first) = values.first() else {
        return Err(CaptureFileError::ClassificationFailed);
    };
    if values.iter().any(|value| value != first) {
        return Err(CaptureFileError::ClassificationFailed);
    }
    Ok((*first).to_owned())
}

fn string_field(
    value: &serde_json::Value,
    field: &str,
    expected_len: usize,
) -> Result<String, CaptureFileError> {
    let value = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(CaptureFileError::PrivateInputInvalid)?;
    if !valid_hex(value, expected_len) {
        return Err(CaptureFileError::PrivateInputInvalid);
    }
    Ok(value.to_owned())
}

fn u64_field(value: &serde_json::Value, field: &str) -> Result<u64, CaptureFileError> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or(CaptureFileError::PrivateInputInvalid)
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::phase36_broker::{
        Phase36AllowedOperation, Phase36LedgerRecord, Phase36LedgerState, Phase36LedgerTransition,
    };
    use crate::phase36_evidence::capture::{
        replace_broker_document, write_candidate_from_private_file,
    };

    const SOURCE: &str = "1111111111111111111111111111111111111111";
    const REFERENCE: &str = "2222222222222222222222222222222222222222";
    const SESSION: &str = "0123456789abcdef0011223344556677";
    const ELF: &str = "6666666666666666666666666666666666666666666666666666666666666666";
    const PACKAGE: &str = "7777777777777777777777777777777777777777777777777777777777777777";

    #[test]
    fn qualified_fake_documents_assemble_into_an_eligible_private_capture() {
        // Arrange
        let root = test_root();
        let child = root.join("attempt-0123456789abcdef");
        fs::create_dir_all(child.join("http-immediate")).expect("test directories");
        fs::create_dir_all(child.join("passive-serial-observation")).expect("monitor directory");
        fs::set_permissions(&child, fs::Permissions::from_mode(0o700)).expect("private child mode");
        let projection = serde_json::json!({
            "bootSession": SESSION,
            "operatorSnapshotRevision": 15,
            "current": 1.25,
            "voltage": 5.1,
            "power": 6.375,
            "temp": 55.25,
            "fanrpm": 4800,
            "currentStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
            "voltageStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
            "powerStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":11,"acquiredAtMs":1100}},
            "chipTempStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":12,"acquiredAtMs":1200}},
            "fanRpmStatus": {"state":"fresh","stamp":{"bootSession":7,"sequence":13,"acquiredAtMs":1300}},
            "runtimeHealth": {
                "selfTestState":"idle",
                "supervisorAvailability":"available",
                "checkpointCategory":"telemetry",
                "checkpointSequence":14,
                "checkpointAgeMillis":250,
                "checkpointHealth":"healthy",
                "taskWatchdogParticipation":"unavailable",
                "taskWatchdogReason":"unproved"
            }
        });
        write_private(
            &child.join("detector-facts.json"),
            serde_json::json!({
                "schema_version":"phase36-detector-facts-v1",
                "board_category":"205",
                "target":"xtensa-esp32s3-espidf",
                "asic":"BM1366",
                "candidate_count":1,
                "port":"synthetic-protected-port",
                "physical_identity_digest":"8".repeat(64)
            })
            .to_string()
            .as_bytes(),
        );
        write_private(
            &child.join("http-immediate/body"),
            serde_json::to_string(&projection)
                .expect("projection")
                .as_bytes(),
        );
        write_private(
            &child.join("websocket.json"),
            format!("websocket_frame_1={projection}\nwebsocket_frame_status=passed frames=1\n")
                .as_bytes(),
        );
        let monitor = format!(
            "runtime_boot_identity session={SESSION} boot_ordinal=11 reset_reason=software_cpu uptime_ms=10 redacted=true\n\
             runtime_origin session={SESSION} boot_ordinal=11 device_url=http://device.invalid redacted=true\n\
             firmware_commit={SOURCE}\nreference_commit={REFERENCE}\napp_elf_sha256={ELF}\n\
             operator_snapshot session={SESSION} revision=15 redacted=true\n\
             runtime_health boot_session={SESSION} operator_snapshot_revision=15 self_test=idle supervisor=available redacted=true\n"
        );
        write_private(
            &child.join("passive-serial-observation/monitor.classifier-input.log"),
            monitor.as_bytes(),
        );
        let manifest = root.join("package.json");
        write_private(
            &manifest,
            serde_json::json!({
                "source_commit":SOURCE,
                "reference_commit":REFERENCE
            })
            .to_string()
            .as_bytes(),
        );
        let input = HardwareCaptureAssembly {
            attempt_child: &child,
            manifest: &manifest,
            manifest_digest: &"3".repeat(64),
            firmware_elf_digest: ELF,
            executable_image_digest: &"4".repeat(64),
            factory_image_digest: &"5".repeat(64),
            package_identity_digest: PACKAGE,
        };

        // Act
        assemble_hardware_capture(&input).expect("qualified documents assemble");
        let records = successful_records(100);
        replace_broker_document(
            &child.join("private-capture.json"),
            BrokerCaptureDocument {
                observation_source: CaptureObservationSource::IndependentBrokerLedger,
                capability_digest: "a".repeat(64),
                package_digest: PACKAGE.to_owned(),
                same_physical_device_observed: true,
                interval_start_millis: 100,
                interval_end_millis: 133,
                records,
            },
        )
        .expect("broker ledger replaces placeholder");
        let candidate = write_candidate_from_private_file(
            &child.join("private-capture.json"),
            &root.join("candidate.json"),
        )
        .expect("assembled capture classifies");

        // Assert
        assert_eq!(candidate.board_category, "205");
        assert_eq!(
            candidate.runtime_identity.observation_source,
            crate::phase36_evidence::runtime_identity::RuntimeIdentityObservationSource::ExactPackageFlashSession
        );
        fs::remove_dir_all(&root).expect("remove isolated test root");
    }

    fn successful_records(start: u64) -> Vec<Phase36LedgerRecord> {
        let mut state = Phase36LedgerState::start(start).expect("state");
        let mut records = Vec::new();
        let mut millis = start;
        for operation in Phase36AllowedOperation::SUCCESS_ORDER {
            for transition in [
                Phase36LedgerTransition::Authorized,
                Phase36LedgerTransition::Invoked,
                Phase36LedgerTransition::Completed,
                Phase36LedgerTransition::Closed,
            ] {
                millis += 1;
                let record = Phase36LedgerRecord::next(&state, operation, transition, millis)
                    .expect("record");
                state.apply(&record).expect("apply");
                records.push(record);
            }
        }
        records
    }

    fn write_private(path: &Utf8Path, bytes: &[u8]) {
        fs::write(path, bytes).expect("write fixture");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("fixture mode");
    }

    fn test_root() -> camino::Utf8PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "phase36-hardware-capture-{}-{nonce}",
            std::process::id()
        ));
        let path = camino::Utf8PathBuf::from_path_buf(path).expect("UTF-8 temp path");
        fs::create_dir(&path).expect("create test root");
        path
    }
}
