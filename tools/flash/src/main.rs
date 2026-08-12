use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use bitaxe_api::{
    classify_runtime_boot_attestations, BuildProvenance, ExpectedRuntimeAttestationIdentity,
    RuntimeAttestationAccumulator, RuntimeAttestationStatus,
};
use bitaxe_config::{
    apply_settings_patch, ultra_205_default_seed_values, ConfigValidationError, NvsWrite,
    RawSettingValue, SettingsPatch, SettingsUpdateDecision, StoredValue, StoredValueKind,
    NVS_NAMESPACE,
};
use bitaxe_device_session::{discover_usb_ports, UsbDeviceEffectState, UsbOperation, UsbSession};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod campaign;
mod cli;
mod commands;
mod environment;
mod esp32s3_image;
mod evidence;
mod evidence_record;
mod execution_snapshot;
mod model;
mod monitor;
mod output;
mod package;
mod package_admission;
mod redaction;
mod support;
mod wifi;

#[cfg(test)]
mod tests;

pub(crate) use campaign::*;
pub(crate) use cli::*;
pub(crate) use commands::*;
pub(crate) use environment::*;
pub(crate) use evidence_record::*;
pub(crate) use execution_snapshot::*;
pub(crate) use model::*;
pub(crate) use monitor::*;
pub(crate) use output::*;
pub(crate) use package::*;
pub(crate) use redaction::*;
pub(crate) use support::*;
pub(crate) use wifi::*;

const PACKAGE_BUILD_DISPLAY: &str = "bazel build //firmware/bitaxe:firmware_image";
const PACKAGE_BUILD_TARGET: &str = "//firmware/bitaxe:firmware_image";
const PACKAGE_MANIFEST_RELATIVE_PATH: &str = "firmware/bitaxe/bitaxe-ultra205-package.json";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS: u64 = 360;
const MIN_COMMIT_PREFIX_LEN: usize = 12;
const NVS_PARTITION_OFFSET: &str = "0x9000";
const NVS_PARTITION_SIZE: &str = "0x6000";
const NVS_GENERATOR_PYTHON_RELATIVE_PATH: &str =
    ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python";
const UNAVAILABLE: &str = "Unavailable";
const PROTECTED_OPERATIONAL: &str = "protected-operational";
const ESPFLASH_EXPECTED_VERSION: &str = "4.5.0";
const PHASE35_FLASH_SCHEMA: &str = "phase35-flash-boundary-v1";
const PHASE36_EFFECT_SCHEMA: &str = "phase36-effect-result-v1";

fn main() -> Result<()> {
    let cli = match parse_cli(env::args()) {
        Ok(cli) => cli,
        Err(error) => {
            maybe_write_phase36_pre_effect_result("parser_failed")
                .context("phase36_effect_result=failed reason=parser_result_write_failed")?;
            return Err(error);
        }
    };
    let environment = match LocalFlashEnvironment::detect() {
        Ok(environment) => environment,
        Err(error) => {
            maybe_write_phase36_pre_effect_result("invocation_construction_failed")
                .context("phase36_effect_result=failed reason=invocation_result_write_failed")?;
            return Err(error);
        }
    };
    emit_line("espflash_version", ESPFLASH_EXPECTED_VERSION)?;
    emit_line("espflash_executable_sha256", &environment.espflash_sha256)?;

    let operation_result = match cli.command {
        CliCommand::Detect(command) => run_detect(&command, &environment),
        CliCommand::Flash(command) => run_flash(&command, &environment).map(|_| ()),
        CliCommand::Monitor(command) => run_monitor(&command, &environment),
        CliCommand::FlashMonitor(command) => run_flash_monitor(&command, &environment),
        CliCommand::FinalizeEvidence(command) => run_finalize_evidence(&command, &environment),
        CliCommand::MiningCampaign(command) => run_mining_campaign(&command, &environment),
        CliCommand::ConfirmIdentify(command) => run_confirm_identify(&command, &environment),
        CliCommand::Phase35Probe(command) => run_phase35_probe(&command, &environment),
    };
    let device_effect_state = environment.device_effect_state();
    let cleanup_result = environment.finish_usb_session();
    let result = combine_operation_and_cleanup(operation_result, cleanup_result);
    maybe_write_phase36_operation_result(result.is_ok(), device_effect_state)
        .context("phase36_effect_result=failed reason=operation_result_write_failed")?;
    result
}
