use std::cell::Cell;

use anyhow::{anyhow, Result};
use camino::{Utf8Path, Utf8PathBuf};

use super::*;

const CHECKLIST: &str = r#"
# Parity Checklist

## Foundation

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| WF-001 | Read-only reference submodule | `reference/esp-miner` | `scripts/verify-reference-clean.sh` | implemented | pending | Guard exists. |
"#;

mod base;
mod guards;
mod phase28;
mod phase30;
mod release;

fn assert_validation_error_contains(
    errors: &[ValidationError],
    expected_id: &str,
    expected_message: &str,
) {
    assert!(
        errors
            .iter()
            .any(|error| { error.id == expected_id && error.message.contains(expected_message) }),
        "expected {expected_id} validation error containing {expected_message:?}, got {errors:#?}"
    );
}

struct FakeEnvironment {
    maybe_guard_error: Option<&'static str>,
    maybe_checklist: Option<&'static str>,
    maybe_phase30_artifact: Option<&'static str>,
    read_called: Cell<bool>,
}

impl FakeEnvironment {
    fn failing_guard(message: &'static str) -> Self {
        Self {
            maybe_guard_error: Some(message),
            maybe_checklist: None,
            maybe_phase30_artifact: None,
            read_called: Cell::new(false),
        }
    }

    fn with_documents(checklist: &'static str, phase30_artifact: &'static str) -> Self {
        Self {
            maybe_guard_error: None,
            maybe_checklist: Some(checklist),
            maybe_phase30_artifact: Some(phase30_artifact),
            read_called: Cell::new(false),
        }
    }
}

impl ReportEnvironment for FakeEnvironment {
    fn run_reference_guard(&self) -> Result<()> {
        if let Some(message) = self.maybe_guard_error {
            return Err(anyhow!(message));
        }

        Ok(())
    }

    fn read_checklist(&self, _path: &Utf8Path) -> Result<String> {
        self.read_called.set(true);
        Ok(self.maybe_checklist.unwrap_or(CHECKLIST).to_owned())
    }

    fn read_phase30_promotion_artifact(&self, _path: &Utf8Path) -> Result<String> {
        let Some(document) = self.maybe_phase30_artifact else {
            bail!("structured Phase 30 evidence artifact is missing");
        };
        Ok(document.to_owned())
    }

    fn reference_commit(&self) -> Result<String> {
        Ok("abc123".to_owned())
    }
}
