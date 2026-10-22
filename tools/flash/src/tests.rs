use super::*;
use std::cell::{Cell, RefCell};
use tempfile::{tempdir, TempDir};

const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const REFERENCE_COMMIT: &str = "abcdef0123456789abcdef0123456789abcdef01";
const BUILD_LABEL: &str = "0123456789ab-dev";
const APP_ELF_SHA256: &str = "05486740ec9e1f2e23c5dfc471ec1b6d01d97b1a1b3f78ed97c44ce894c1f83e";

include!("tests/fixtures.rs");
include!("tests/fake_environment.rs");
include!("tests/fake_usb_environment.rs");

mod admission;
mod admission_layout;
mod board_target_flash;
mod campaign;
mod capture;
mod cli;
mod cli_identify;
mod cli_noise_diagnostic;
mod cli_owner_recovery;
mod cli_physical_binding;
mod cli_release_recovery;
mod cli_restore_installed;
mod cli_tcp_payload;
mod cli_usb_stability;
mod display_recovery;
#[path = "tests/evidence.rs"]
mod evidence_cases;
mod execution_profile;
mod firmware_variant;
mod input_uat;
mod installed_application;
mod monitor;
mod nvs_readback;
mod phase35;
mod redaction;
mod release_recovery;
mod restore_installed;
mod segmented_updates;
mod web_ui;
mod workflow;

#[path = "core_dump/run_tests.rs"]
mod core_dump_run;
