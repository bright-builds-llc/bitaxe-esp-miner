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
    classify_runtime_boot_attestations, runtime_boot_attestation_marker_start, BuildProvenance,
    ExpectedRuntimeAttestationIdentity, RuntimeAttestationAccumulator, RuntimeAttestationStatus,
    UsbBootProfileMarker, USB_BOOT_PROFILE_MARKER,
};
use bitaxe_automation_contracts::{
    InputUatEvidence, InputUatObservationEvidence, ReleaseRecoveryEvidence,
    INPUT_UAT_EVIDENCE_SCHEMA, RELEASE_RECOVERY_EVIDENCE_SCHEMA,
};
use bitaxe_config::{
    apply_settings_patch, ultra_205_default_seed_values, ConfigValidationError, NvsWrite,
    RawSettingValue, SettingsPatch, SettingsUpdateDecision, StoredValue, StoredValueKind,
    NVS_NAMESPACE,
};
use bitaxe_device_session::{
    admit_application_execution, admit_rom_downloader, board_info_reports_esp32s3,
    discover_usb_ports, handoff_worker_to_rom, inspect_usb_profile,
    native_usb_transition_module_sha256, plan_usb_operation, run_installed_application,
    verify_native_usb_transition, MonitorOutput, NativeUsbTransitionOutcome,
    ProfileObservationCounts, UsbCommandDiagnostic, UsbDeviceEffectState, UsbExecutionOwner,
    UsbIntent, UsbOperation, UsbOperationPlan, UsbProfile, UsbSession, UsbTerminalCategory,
};
#[cfg(test)]
use bitaxe_device_session::{UsbCommandTermination, UsbConnectionSignature};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod boot_chain;
mod campaign;
mod cli;
mod commands;
mod display_recovery;
mod environment;
mod esp32s3_image;
mod evidence;
mod evidence_record;
mod execution_snapshot;
mod input_uat;
mod model;
mod monitor;
mod native_usb_transition;
mod noise_diagnostic;
mod nvs_readback;
mod output;
mod owner_recovery;
mod package;
mod package_admission;
mod redaction;
mod release_recovery;
mod restore_installed;
mod rom_exit;
mod self_test_intent;
mod support;
mod tcp_payload_diagnostic;
mod thermal_fault_intent;
mod usb_stability;
mod wifi;

#[cfg(test)]
mod tests;

pub(crate) use boot_chain::*;
pub(crate) use campaign::*;
pub(crate) use cli::*;
pub(crate) use commands::*;
pub(crate) use display_recovery::*;
pub(crate) use environment::*;
pub(crate) use evidence_record::*;
pub(crate) use execution_snapshot::*;
pub(crate) use input_uat::*;
pub(crate) use model::*;
pub(crate) use monitor::*;
pub(crate) use native_usb_transition::*;
pub(crate) use noise_diagnostic::*;
pub(crate) use nvs_readback::*;
pub(crate) use output::*;
pub(crate) use owner_recovery::*;
pub(crate) use package::*;
pub(crate) use redaction::*;
pub(crate) use release_recovery::*;
pub(crate) use restore_installed::*;
pub(crate) use rom_exit::*;
pub(crate) use self_test_intent::*;
pub(crate) use support::*;
pub(crate) use tcp_payload_diagnostic::*;
pub(crate) use thermal_fault_intent::*;
pub(crate) use usb_stability::*;
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
        CliCommand::InputUat(command) => run_input_uat(&command, &environment),
        CliCommand::SignalIdentify(command) => run_signal_identify(&command, &environment),
        CliCommand::Phase35Probe(command) => run_phase35_probe(&command, &environment),
        CliCommand::ReleaseRecovery(command) => run_release_recovery(&command, &environment),
        CliCommand::RestoreInstalled(command) => run_restore_installed(&command, &environment),
        CliCommand::NoiseDiagnostic(command) => {
            run_noise_diagnostic_command(&command, &environment)
        }
        CliCommand::TcpPayloadDiagnostic(command) => {
            run_tcp_payload_diagnostic_command(&command, &environment)
        }
        CliCommand::VerifyNativeUsbTransition(command) => {
            run_verify_native_usb_transition(&command, &environment)
        }
        CliCommand::DisplayRecoveryStart(command) => {
            run_display_recovery_start(&command, &environment)
        }
        CliCommand::NvsReadback(command) => run_nvs_readback(&command, &environment),
        CliCommand::NvsRuntimeRestore(command) => run_nvs_runtime_restore(&command, &environment),
        CliCommand::RomExitDiagnostic(command) => run_rom_exit_diagnostic(&command, &environment),
        CliCommand::OwnerRecovery(command) => run_owner_recovery(command, &environment),
        CliCommand::BootChainReadback(command) => run_boot_chain_readback(command, &environment),
        CliCommand::UsbStabilityRead(command) => run_usb_stability_read(command, &environment),
    };
    let device_effect_state = environment.device_effect_state();
    let cleanup_result = environment.finish_usb_session();
    let result = combine_operation_and_cleanup(operation_result, cleanup_result);
    maybe_write_phase36_operation_result(result.is_ok(), device_effect_state)
        .context("phase36_effect_result=failed reason=operation_result_write_failed")?;
    result
}
