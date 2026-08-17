use super::*;
use std::cell::{Cell, RefCell};
use tempfile::{tempdir, TempDir};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const REFERENCE_COMMIT: &str = "abcdef0123456789abcdef0123456789abcdef01";
const BUILD_LABEL: &str = "0123456789ab-dev";
const APP_ELF_SHA256: &str = "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";

include!("tests/fixtures.rs");
include!("tests/fake_environment.rs");

mod admission;
mod admission_layout;
mod campaign;
mod capture;
mod cli;
mod cli_identify;
mod cli_release_recovery;
#[path = "tests/evidence.rs"]
mod evidence_cases;
mod input_uat;
mod monitor;
mod phase35;
mod redaction;
mod release_recovery;
mod workflow;
