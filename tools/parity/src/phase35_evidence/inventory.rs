use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use camino::Utf8Path;

use super::{
    inventory_contract_digest, sha256_hex, ArtifactFileKind, InventoryArtifact,
    Phase35EvidenceError, Phase35EvidenceRootInput, EVIDENCE_DOCUMENT, INVENTORY_ROLES,
};

pub(crate) fn load_phase35_evidence_root(
    evidence_root: &Utf8Path,
) -> Result<
    (
        Phase35EvidenceRootInput,
        BTreeMap<String, InventoryArtifact>,
    ),
    Phase35EvidenceError,
> {
    let input_bytes = fs::read(evidence_root.join(EVIDENCE_DOCUMENT).as_std_path())
        .map_err(|_| Phase35EvidenceError::InventoryMismatch)?;
    let input = serde_json::from_slice::<Phase35EvidenceRootInput>(&input_bytes)
        .map_err(|_| Phase35EvidenceError::InventoryMismatch)?;
    let mut artifacts = BTreeMap::new();
    for entry in &input.inventory {
        validate_relative_path(&entry.path)?;
        let path = evidence_root.join(&entry.path);
        let metadata = fs::symlink_metadata(path.as_std_path())
            .map_err(|_| Phase35EvidenceError::InventoryMismatch)?;
        let file_kind = if metadata.file_type().is_symlink() {
            ArtifactFileKind::Symlink
        } else if metadata.is_file() {
            ArtifactFileKind::Regular
        } else {
            ArtifactFileKind::Other
        };
        let bytes = if file_kind == ArtifactFileKind::Regular {
            fs::read(path.as_std_path()).map_err(|_| Phase35EvidenceError::InventoryMismatch)?
        } else {
            Vec::new()
        };
        artifacts.insert(entry.path.clone(), InventoryArtifact { bytes, file_kind });
    }
    Ok((input, artifacts))
}

pub(super) fn validate_inventory(
    input: &Phase35EvidenceRootInput,
    artifacts: &BTreeMap<String, InventoryArtifact>,
) -> Result<String, Phase35EvidenceError> {
    if input.inventory.len() != INVENTORY_ROLES.len() || artifacts.len() != INVENTORY_ROLES.len() {
        return Err(Phase35EvidenceError::InventoryMismatch);
    }
    let mut seen_roles = BTreeSet::new();
    let mut seen_paths = BTreeSet::new();
    for (index, entry) in input.inventory.iter().enumerate() {
        if entry.role != INVENTORY_ROLES[index]
            || !seen_roles.insert(entry.role.as_str())
            || !seen_paths.insert(entry.path.as_str())
        {
            return Err(Phase35EvidenceError::InventoryMismatch);
        }
        validate_relative_path(&entry.path)?;
        if !super::is_lower_hex(&entry.sha256, 64) {
            return Err(Phase35EvidenceError::InvalidDigest);
        }
        let Some(artifact) = artifacts.get(&entry.path) else {
            return Err(Phase35EvidenceError::InventoryMismatch);
        };
        match artifact.file_kind {
            ArtifactFileKind::Symlink => return Err(Phase35EvidenceError::Symlink),
            ArtifactFileKind::Other => return Err(Phase35EvidenceError::InventoryMismatch),
            ArtifactFileKind::Regular => {}
        }
        if sha256_hex(&artifact.bytes) != entry.sha256
            || entry.sha256 != expected_role_digest(input, &entry.role)
        {
            return Err(role_mismatch_error(&entry.role));
        }
    }
    Ok(inventory_contract_digest(&input.inventory))
}

fn expected_role_digest(input: &Phase35EvidenceRootInput, role: &str) -> String {
    match role {
        "package_manifest" => input.exact_package.manifest_digest.clone(),
        "executable_image" => input.exact_package.executable_image_digest.clone(),
        "factory_image" => input.exact_package.factory_image_digest.clone(),
        "package" => input.exact_package.package_digest.clone(),
        "runtime_identity" => input.exact_package.runtime_identity_digest.clone(),
        "target_lock" => input.admission_facts.target_lock_digest.clone(),
        "detector_capability" => input.detector_run.detector_capability_digest.clone(),
        "no_actuation" => event_payload(input),
        "boot_a_api" => sha256_hex(input.boot_a.system_info_document.as_bytes()),
        "boot_a_websocket" => sha256_hex(input.boot_a.websocket_document.as_bytes()),
        "boot_a_retained_log" => sha256_hex(input.boot_a.retained_log_document.as_bytes()),
        "boot_b_api" => sha256_hex(input.boot_b.system_info_document.as_bytes()),
        "boot_b_websocket" => sha256_hex(input.boot_b.websocket_document.as_bytes()),
        "boot_b_retained_log" => sha256_hex(input.boot_b.retained_log_document.as_bytes()),
        _ => String::new(),
    }
}

fn role_mismatch_error(role: &str) -> Phase35EvidenceError {
    match role {
        "executable_image" => Phase35EvidenceError::ExecutableImageMismatch,
        "factory_image" => Phase35EvidenceError::FactoryImageMismatch,
        "package_manifest" | "package" => Phase35EvidenceError::PackageMismatch,
        "runtime_identity" => Phase35EvidenceError::RuntimeIdentityMismatch,
        "no_actuation" => Phase35EvidenceError::NoActuationFailure,
        _ => Phase35EvidenceError::InventoryMismatch,
    }
}

fn event_payload(input: &Phase35EvidenceRootInput) -> String {
    input
        .events
        .iter()
        .find(|event| event.category == super::EvidenceEventCategory::NoActuationVerified)
        .map(|event| event.payload_digest.clone())
        .unwrap_or_default()
}

fn validate_relative_path(path: &str) -> Result<(), Phase35EvidenceError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(Phase35EvidenceError::UnsafePath);
    }
    if Utf8Path::new(path).is_absolute() {
        return Err(Phase35EvidenceError::UnsafePath);
    }
    Ok(())
}
