//! Deterministic coverage validation for the reference-derived parity inventory.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use crate::{normalize, ChecklistRow, ValidationError};

pub(crate) const DEFAULT_REFERENCE_INVENTORY_PATH: &str =
    "docs/parity/reference-surface-inventory.json";
const INVENTORY_SCHEMA_VERSION: &str = "bitaxe-reference-surface-inventory-v1";
const INVENTORY_ERROR_ID: &str = "REFERENCE-INVENTORY";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceInventory {
    schema_version: String,
    reference_commit: String,
    domains: Vec<ReferenceDomain>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceDomain {
    id: String,
    category: String,
    scope: String,
    provenance: String,
    non_claim: String,
    checklist_rows: Vec<String>,
    locators: Vec<ReferenceLocator>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceLocator {
    path: String,
    anchors: Vec<String>,
}

pub(crate) fn validate_inventory(
    workspace: &Utf8Path,
    document: &str,
    reference_commit: &str,
    rows: &[ChecklistRow],
) -> Vec<ValidationError> {
    let inventory: ReferenceInventory = match serde_json::from_str(document) {
        Ok(inventory) => inventory,
        Err(error) => {
            return vec![inventory_error(format!(
                "inventory JSON is malformed: {error}"
            ))];
        }
    };

    let mut errors = validate_header(&inventory, reference_commit);
    errors.extend(validate_row_metadata(rows));
    errors.extend(validate_domains(workspace, &inventory.domains, rows));
    errors
}

fn validate_header(inventory: &ReferenceInventory, reference_commit: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if inventory.schema_version != INVENTORY_SCHEMA_VERSION {
        errors.push(inventory_error(format!(
            "schema_version must be {INVENTORY_SCHEMA_VERSION}"
        )));
    }
    if inventory.reference_commit != reference_commit {
        errors.push(inventory_error(format!(
            "reference_commit must match the pinned reference: expected {reference_commit}"
        )));
    }
    if inventory.domains.is_empty() {
        errors.push(inventory_error("inventory must define at least one domain"));
    }

    errors
}

fn validate_row_metadata(rows: &[ChecklistRow]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut row_counts = BTreeMap::<&str, usize>::new();

    for row in rows {
        *row_counts.entry(row.id.as_str()).or_default() += 1;

        for (label, value) in [
            ("ID", row.id.as_str()),
            ("Surface", row.surface.as_str()),
            ("Reference Breadcrumb", row.reference_breadcrumb.as_str()),
            ("Rust-Owned Target", row.rust_owned_target.as_str()),
            ("Status", row.status.as_str()),
            ("Evidence", row.evidence.as_str()),
            ("Notes", row.notes.as_str()),
        ] {
            if value.trim().is_empty() {
                errors.push(ValidationError {
                    id: row.id.clone(),
                    message: format!("checklist metadata {label} must not be empty"),
                });
            }
        }

        if ![
            "not-started",
            "in-progress",
            "implemented",
            "verified",
            "deferred",
        ]
        .contains(&normalize(&row.status).as_str())
        {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "checklist status is outside the closed vocabulary".to_owned(),
            });
        }
    }

    for (id, count) in row_counts {
        if count > 1 {
            errors.push(ValidationError {
                id: id.to_owned(),
                message: "checklist surface ID appears more than once".to_owned(),
            });
        }
    }

    errors
}

fn validate_domains(
    workspace: &Utf8Path,
    domains: &[ReferenceDomain],
    rows: &[ChecklistRow],
) -> Vec<ValidationError> {
    let checklist_ids = rows
        .iter()
        .map(|row| row.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut domain_ids = BTreeSet::new();
    let mut surface_counts = BTreeMap::<&str, usize>::new();
    let mut errors = Vec::new();

    for domain in domains {
        errors.extend(validate_domain_metadata(domain, &mut domain_ids));
        errors.extend(validate_domain_locators(workspace, domain));
        for checklist_id in &domain.checklist_rows {
            *surface_counts.entry(checklist_id.as_str()).or_default() += 1;
        }
    }

    for checklist_id in &checklist_ids {
        match surface_counts
            .get(checklist_id)
            .copied()
            .unwrap_or_default()
        {
            0 => errors.push(ValidationError {
                id: (*checklist_id).to_owned(),
                message: "checklist surface is missing from the reference inventory".to_owned(),
            }),
            1 => {}
            _ => errors.push(ValidationError {
                id: (*checklist_id).to_owned(),
                message: "checklist surface appears more than once in the inventory".to_owned(),
            }),
        }
    }

    for inventory_id in surface_counts.keys() {
        if !checklist_ids.contains(inventory_id) {
            errors.push(ValidationError {
                id: (*inventory_id).to_owned(),
                message: "inventory surface has no canonical checklist row".to_owned(),
            });
        }
    }

    errors
}

fn validate_domain_metadata<'domain>(
    domain: &'domain ReferenceDomain,
    domain_ids: &mut BTreeSet<&'domain str>,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let domain_id = if domain.id.trim().is_empty() {
        INVENTORY_ERROR_ID
    } else {
        domain.id.as_str()
    };

    if !domain_ids.insert(domain.id.as_str()) {
        errors.push(ValidationError {
            id: domain_id.to_owned(),
            message: "inventory domain ID appears more than once".to_owned(),
        });
    }
    for (label, value) in [
        ("id", domain.id.as_str()),
        ("category", domain.category.as_str()),
        ("scope", domain.scope.as_str()),
        ("provenance", domain.provenance.as_str()),
        ("non_claim", domain.non_claim.as_str()),
    ] {
        if value.trim().is_empty() {
            errors.push(ValidationError {
                id: domain_id.to_owned(),
                message: format!("inventory domain metadata {label} must not be empty"),
            });
        }
    }
    if domain.checklist_rows.is_empty() {
        errors.push(ValidationError {
            id: domain_id.to_owned(),
            message: "inventory domain must map at least one checklist surface".to_owned(),
        });
    }
    if domain.locators.is_empty() {
        errors.push(ValidationError {
            id: domain_id.to_owned(),
            message: "inventory domain must define at least one exact source locator".to_owned(),
        });
    }

    errors
}

fn validate_domain_locators(
    workspace: &Utf8Path,
    domain: &ReferenceDomain,
) -> Vec<ValidationError> {
    domain
        .locators
        .iter()
        .filter_map(|locator| {
            validate_locator(workspace, locator)
                .err()
                .map(|message| ValidationError {
                    id: domain.id.clone(),
                    message: format!("invalid source locator {}: {message}", locator.path),
                })
        })
        .collect()
}

fn validate_locator(workspace: &Utf8Path, locator: &ReferenceLocator) -> Result<(), String> {
    let relative = Utf8PathBuf::from(&locator.path);
    if relative.is_absolute()
        || relative.as_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                camino::Utf8Component::ParentDir | camino::Utf8Component::CurDir
            )
        })
        || locator.path.contains('*')
    {
        return Err("path must be an exact normalized repository-relative path".to_owned());
    }

    let path = workspace.join(&relative);
    let metadata =
        fs::symlink_metadata(path.as_std_path()).map_err(|_| "path does not exist".to_owned())?;
    if metadata.file_type().is_symlink() {
        return Err("path must not be a symbolic link".to_owned());
    }
    if !metadata.is_file() {
        return Err("path must be a regular source file".to_owned());
    }
    if locator.anchors.is_empty() {
        return Err("at least one symbol, route, key, or content anchor is required".to_owned());
    }

    let contents = fs::read_to_string(path.as_std_path())
        .map_err(|_| "source file is not readable UTF-8 text".to_owned())?;
    for anchor in &locator.anchors {
        if anchor.trim().is_empty() {
            return Err("anchors must not be empty".to_owned());
        }
        if !contents.contains(anchor) {
            return Err(format!("anchor is absent: {anchor}"));
        }
    }

    Ok(())
}

fn inventory_error(message: impl Into<String>) -> ValidationError {
    ValidationError {
        id: INVENTORY_ERROR_ID.to_owned(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use serde_json::json;

    use super::*;
    use crate::parse_checklist;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory {
        path: Utf8PathBuf,
    }

    impl TestDirectory {
        fn new() -> Self {
            let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "bitaxe-reference-inventory-{}-{ordinal}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test directory");
            Self {
                path: Utf8PathBuf::from_path_buf(path).expect("UTF-8 temp path"),
            }
        }

        fn write_source(&self) {
            let source = self.path.join("reference/esp-miner/main/main.c");
            fs::create_dir_all(source.parent().expect("source parent").as_std_path())
                .expect("create source parent");
            fs::write(source.as_std_path(), "void app_main(void) {}\n")
                .expect("write source fixture");
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(self.path.as_std_path()).expect("remove test directory");
        }
    }

    fn checklist() -> Vec<ChecklistRow> {
        parse_checklist(
            r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SYS-001 | Boot | `reference/esp-miner/main/main.c:app_main` | `firmware/bitaxe` | implemented | unit | Hardware boot parity remains unclaimed. |
"#,
        )
        .expect("parse checklist")
    }

    fn inventory(surface_ids: &[&str]) -> serde_json::Value {
        json!({
            "schema_version": INVENTORY_SCHEMA_VERSION,
            "reference_commit": "reference-commit",
            "domains": [{
                "id": "boot-runtime",
                "category": "boot and runtime",
                "scope": "all supported boards; hardware verification remains board-specific",
                "provenance": "independent Rust implementation guided by reference behavior",
                "non_claim": "inventory coverage is not behavior verification",
                "checklist_rows": surface_ids,
                "locators": [{
                    "path": "reference/esp-miner/main/main.c",
                    "anchors": ["app_main"]
                }]
            }]
        })
    }

    #[test]
    fn valid_inventory_maps_each_surface_once() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let document = inventory(&["SYS-001"]).to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "reference-commit", &rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_surface_mapping_fails_closed() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let document = inventory(&[]).to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "reference-commit", &rows);

        // Assert
        assert!(errors.iter().any(|error| {
            error.id == "SYS-001"
                && error
                    .message
                    .contains("missing from the reference inventory")
        }));
    }

    #[test]
    fn duplicate_surface_mapping_fails_closed() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let document = inventory(&["SYS-001", "SYS-001"]).to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "reference-commit", &rows);

        // Assert
        assert!(errors.iter().any(|error| {
            error.id == "SYS-001" && error.message.contains("more than once in the inventory")
        }));
    }

    #[test]
    fn reference_commit_drift_fails_closed() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let document = inventory(&["SYS-001"]).to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "different-commit", &rows);

        // Assert
        assert!(errors.iter().any(|error| {
            error.id == INVENTORY_ERROR_ID && error.message.contains("reference_commit")
        }));
    }

    #[test]
    fn absent_source_anchor_fails_closed() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let mut value = inventory(&["SYS-001"]);
        value["domains"][0]["locators"][0]["anchors"] = json!(["missing_symbol"]);
        let document = value.to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "reference-commit", &rows);

        // Assert
        assert!(errors.iter().any(|error| {
            error.id == "boot-runtime" && error.message.contains("anchor is absent")
        }));
    }

    #[test]
    fn empty_domain_metadata_fails_closed() {
        // Arrange
        let directory = TestDirectory::new();
        directory.write_source();
        let mut value = inventory(&["SYS-001"]);
        value["domains"][0]["scope"] = json!("");
        let document = value.to_string();
        let rows = checklist();

        // Act
        let errors = validate_inventory(&directory.path, &document, "reference-commit", &rows);

        // Assert
        assert!(errors.iter().any(|error| {
            error.id == "boot-runtime" && error.message.contains("metadata scope")
        }));
    }
}
