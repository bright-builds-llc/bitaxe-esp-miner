use sha2::{Digest, Sha256};

use super::{
    DetectorRunCapability, EvidenceEpochInput, EvidenceEventInput, ExactPackageCapability,
    InventoryEntryInput, Phase35EvidenceRootInput,
};

pub(crate) fn exact_package_capability_digest(package: &ExactPackageCapability) -> String {
    hash_fields(
        "phase35-exact-package-v1",
        &[
            &package.source_commit,
            &package.reference_commit,
            if package.reference_clean {
                "true"
            } else {
                "false"
            },
            &package.manifest_schema,
            &package.manifest_digest,
            &package.executable_image_digest,
            &package.factory_image_digest,
            &package.package_digest,
            &package.runtime_identity_digest,
            if package.current_head_verified {
                "true"
            } else {
                "false"
            },
        ],
    )
}

pub(crate) fn detector_run_capability_digest(detector: &DetectorRunCapability) -> String {
    hash_fields(
        "phase35-detector-run-v1",
        &[
            &detector.board_category,
            &detector.detector_capability_digest,
            &detector.physical_identity_digest,
            if detector.board_info_verified {
                "true"
            } else {
                "false"
            },
            if detector.single_candidate_verified {
                "true"
            } else {
                "false"
            },
            &detector.run_id_digest,
        ],
    )
}

pub(crate) fn inventory_contract_digest(inventory: &[InventoryEntryInput]) -> String {
    let fields = inventory
        .iter()
        .flat_map(|entry| [&entry.role, &entry.path, &entry.sha256])
        .map(String::as_str)
        .collect::<Vec<_>>();
    hash_fields("phase35-inventory-v1", &fields)
}

pub(crate) fn phase35_root_contract_digest(
    input: &Phase35EvidenceRootInput,
    inventory_digest: &str,
) -> String {
    hash_fields(
        "phase35-root-contract-v1",
        &[
            &input.exact_package.capability_digest,
            &input.detector_run.capability_digest,
            &input.admission_facts.target_lock_digest,
            &input.admission_facts.lifecycle_id,
            &input.detector_run.run_id_digest,
            inventory_digest,
            &input.boot_a.boot_ordinal.to_string(),
            &input.boot_a.boot_session_digest,
            &input.boot_b.boot_ordinal.to_string(),
            &input.boot_b.boot_session_digest,
            &input.exact_package.runtime_identity_digest,
        ],
    )
}

pub(crate) fn evidence_epoch_digest(epoch: &EvidenceEpochInput) -> String {
    hash_fields(
        "phase35-epoch-v1",
        &[
            &epoch.boot_ordinal.to_string(),
            &epoch.boot_session_digest,
            &epoch.started_millis.to_string(),
            &epoch.ended_millis.to_string(),
            &sha256_hex(epoch.system_info_document.as_bytes()),
            &sha256_hex(epoch.websocket_document.as_bytes()),
            &sha256_hex(epoch.retained_log_document.as_bytes()),
            &epoch.storage_revision.to_string(),
            &epoch.storage_value_digest,
            &epoch.reset_category,
            &epoch.package_capability_digest,
            &epoch.detector_capability_digest,
            &epoch.root_contract_digest,
            &epoch.target_lock_digest,
            &epoch.run_id_digest,
            &epoch.runtime_identity_digest,
            &epoch.physical_identity_digest,
        ],
    )
}

pub(crate) fn storage_confirmation_digest(epoch: &EvidenceEpochInput) -> String {
    hash_fields(
        "phase35-storage-confirmation-v1",
        &[
            &epoch.storage_revision.to_string(),
            &epoch.storage_value_digest,
        ],
    )
}

pub(crate) fn evidence_event_digest(event: &EvidenceEventInput) -> String {
    hash_fields(
        "phase35-event-v1",
        &[
            &event.sequence.to_string(),
            event.category.as_str(),
            &event.monotonic_millis.to_string(),
            &event.payload_digest,
            &event.predecessor_event_digest,
        ],
    )
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

pub(super) fn hash_fields(domain: &str, fields: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for field in fields {
        hasher.update([0]);
        hasher.update(field.as_bytes());
    }
    encode_hex(&hasher.finalize())
}

pub(super) fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
