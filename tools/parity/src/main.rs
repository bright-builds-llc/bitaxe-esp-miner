use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};
use bitaxe_api::phase33_evidence::{
    classify_phase33_baseline, classify_phase33_delivery, classify_phase33_post_restart,
};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand, ValueEnum};
use operator_evidence::{
    load_operator_evidence_documents, publish_phase35_generation, read_phase36_public_checklist,
    render_operator_evidence_report, validate_operator_evidence_documents_with_snapshot_coherence,
    OperatorEvidenceFilters, OperatorEvidenceProfile, Phase35GenerationDocuments,
    Phase35PublicationOptions,
};
use release_evidence::{
    parse_flash_evidence_json, parse_release_evidence_manifest_json,
    render_release_evidence_report, validate_release_evidence, ReleaseEvidenceDocuments,
};
use release_gate::{
    render_release_gate_report, validate_release_gate, ReleaseGateDocuments,
    DEFAULT_CARGO_ABOUT_PATH, DEFAULT_LICENSE_INVENTORY_PATH, DEFAULT_PROVENANCE_PATH,
};
use serde::Serialize;
use std::io::ErrorKind;

const BAZEL_REFERENCE_GUARD_TARGET: &str = "//scripts:verify_reference_clean";
const DEFAULT_REFERENCE_GUARD_PATH: &str = "scripts/verify-reference-clean.sh";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
const DEFAULT_OPENAPI_PATH: &str = "reference/esp-miner/main/http_server/openapi.yaml";
const DEFAULT_API_COMPARE_MANIFEST: &str = "tools/parity/fixtures/api/phase05-required-routes.json";
const DEFAULT_AXEOS_ROUTE_USAGE: &str = "tools/parity/fixtures/api/axeos-route-usage.json";
const DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH: &str =
    "docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md";
const PHASE35_DESTINATION_ROOT: &str =
    "docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion";
const PHASE35_CHECKLIST_PATH: &str = "docs/parity/checklist.md";
const PHASE36_DESTINATION_ROOT: &str =
    "docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion";
const PHASE36_STAGING_ROOT: &str =
    "docs/parity/evidence/.phase-36-substantive-evidence-admission-and-exact-re-promotion.staging";
const PHASE35_MANIFEST_PATH: &str =
    "docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json";

mod api_compare;
mod checklist_revision;
mod checklist_targets;
mod claim_ladder;
mod mining_allow;
mod operator_evidence;
mod operator_snapshot_evidence;
mod parity_work;
#[cfg(test)]
mod phase34_source_guard;
mod phase35_evidence;
mod phase35_flash;
mod phase35_http;
mod phase35_http_probe;
mod phase35_promotion;
pub mod phase36_broker;
pub mod phase36_evidence;
mod phase36_offline;
mod phase36_promotion;
mod protected_input;
mod release_evidence;
mod release_gate;
mod safety_allow;
mod sys004_version_evidence;
mod v12_admission;

mod cli;
mod commands;
mod environment;
mod private_files;
mod reference_inventory;
mod report;
#[cfg(test)]
mod tests;

pub(crate) use cli::*;
pub(crate) use commands::*;
pub(crate) use environment::*;
pub(crate) use parity_work::*;
pub(crate) use private_files::*;
pub(crate) use report::*;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        CliCommand::ClassifyPhase36Evidence(args) => {
            let output = run_classify_phase36_evidence_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::ClassifyPhase36Effects(args) => {
            let output = run_classify_phase36_effects_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::Phase36EvaluatorIdentity => {
            let mut stdout = io::stdout().lock();
            writeln!(
                stdout,
                "evaluator_identity_digest={}",
                phase36_evidence::current_phase36_evidence_evaluator_digest()
            )?;
            return Ok(());
        }
        CliCommand::Phase36AssembleHardwareCapture(args) => {
            let input = phase36_evidence::capture::HardwareCaptureAssembly {
                attempt_child: &args.attempt_child,
                manifest: &args.manifest,
                manifest_digest: &args.manifest_digest,
                firmware_elf_digest: &args.firmware_elf_digest,
                executable_image_digest: &args.executable_image_digest,
                factory_image_digest: &args.factory_image_digest,
                package_identity_digest: &args.package_identity_digest,
            };
            phase36_evidence::capture::assemble_hardware_capture(&input)
                .map_err(|error| anyhow::anyhow!("category={error}"))?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "category=hardware_capture_assembled")?;
            return Ok(());
        }
        CliCommand::Phase36SyntheticCapture(args) => {
            let output = run_phase36_synthetic_capture_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::Phase36HardwareCapture(args) => {
            let output = run_phase36_hardware_capture_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::InspectPhase36Candidate(args) => {
            let output = run_inspect_phase36_candidate_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::ClassifyPhase36Candidate(args) => {
            let output = run_classify_phase36_candidate_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::ReevaluatePhase36Attempt31(args) => {
            let output = run_reevaluate_phase36_attempt31_command(args)?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{output}")?;
            return Ok(());
        }
        CliCommand::ProjectSys004VersionEvidence(args) => {
            sys004_version_evidence::project_sys004_version_evidence(
                &args.private_parent,
                &args.attempt_handle_file,
                &args.package_manifest,
                &args.output,
            )
            .map_err(|error| anyhow::anyhow!("category={error}"))?;
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "category=sys004_version_evidence_projected")?;
            return Ok(());
        }
        _ => {}
    }
    let environment = LocalEnvironment::detect()?;

    let output = match cli.command {
        CliCommand::Report(args) => {
            let request = ReportRequest::from(args);
            run_report(&request, &environment)?
        }
        CliCommand::NextItem(args) => run_next_item_command(&args, &environment)?,
        CliCommand::Progress(args) => run_progress_command(&args, &environment)?,
        CliCommand::SyncProgress(args) => run_sync_progress_command(&args, &environment)?,
        CliCommand::TransitionItem(args) => run_transition_item_command(&args, &environment)?,
        CliCommand::ReviseChecklistDocumentation(args) => {
            run_revise_checklist_documentation_command(&args, &environment)?
        }
        CliCommand::ApiCompare(args) => run_api_compare_command(args, &environment)?,
        CliCommand::ReleaseGate(args) => run_release_gate_command(args, &environment)?,
        CliCommand::ReleaseEvidence(args) => run_release_evidence_command(args, &environment)?,
        CliCommand::SafetyAllow(args) => run_safety_allow_command(args, &environment)?,
        CliCommand::MiningAllow(args) => run_mining_allow_command(args, &environment)?,
        CliCommand::OperatorEvidence(args) => run_operator_evidence_command(args, &environment)?,
        CliCommand::Phase33Classify(args) => run_phase33_classify_command(args, &environment)?,
        CliCommand::ClassifyPhase35Flash(args) => {
            run_classify_phase35_flash_command(args, &environment)?
        }
        CliCommand::ClassifyPhase35Http(args) => {
            run_classify_phase35_http_command(args, &environment)?
        }
        CliCommand::ProbePhase35Http(args) => run_probe_phase35_http_command(args, &environment)?,
        CliCommand::ValidatePhase35Evidence(args) => {
            run_validate_phase35_evidence_command(args, &environment)?
        }
        CliCommand::AdmitPhase35Evidence(args) => {
            run_admit_phase35_evidence_command(args, &environment)?
        }
        CliCommand::ClassifyPhase36Evidence(_) => {
            bail!("Phase 36 classifier dispatch entered workspace-aware path")
        }
        CliCommand::ClassifyPhase36Effects(_) => {
            bail!("Phase 36 effect classifier dispatch entered workspace-aware path")
        }
        CliCommand::Phase36EvaluatorIdentity => {
            bail!("Phase 36 evaluator identity dispatch entered workspace-aware path")
        }
        CliCommand::Phase36AssembleHardwareCapture(_) => {
            bail!("Phase 36 hardware assembler dispatch entered workspace-aware path")
        }
        CliCommand::Phase36SyntheticCapture(_) => {
            bail!("Phase 36 synthetic broker dispatch entered workspace-aware path")
        }
        CliCommand::Phase36HardwareCapture(_) => {
            bail!("Phase 36 hardware broker dispatch entered workspace-aware path")
        }
        CliCommand::InspectPhase36Candidate(_) => {
            bail!("Phase 36 candidate inspector dispatch entered workspace-aware path")
        }
        CliCommand::ClassifyPhase36Candidate(_) => {
            bail!("Phase 36 candidate classifier dispatch entered workspace-aware path")
        }
        CliCommand::ReevaluatePhase36Attempt31(_) => {
            bail!("Phase 36 offline re-evaluation dispatch entered workspace-aware path")
        }
        CliCommand::ProjectSys004VersionEvidence(_) => {
            bail!("SYS-004 projection dispatch entered workspace-aware path")
        }
    };

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}")?;

    Ok(())
}
