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
    load_operator_evidence_documents, publish_phase35_generation, render_operator_evidence_report,
    validate_operator_evidence_documents_with_snapshot_coherence, OperatorEvidenceFilters,
    OperatorEvidenceProfile, Phase35GenerationDocuments, Phase35PublicationOptions,
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

const BAZEL_REFERENCE_GUARD_TARGET: &str = "//tools/automation:verify_reference";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
const DEFAULT_OPENAPI_PATH: &str = "reference/esp-miner/main/http_server/openapi.yaml";
const DEFAULT_API_COMPARE_MANIFEST: &str = "tools/parity/fixtures/api/phase05-required-routes.json";
const DEFAULT_AXEOS_ROUTE_USAGE: &str = "tools/parity/fixtures/api/axeos-route-usage.json";
const DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH: &str =
    "docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md";
const PHASE35_DESTINATION_ROOT: &str =
    "docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion";
const PHASE35_CHECKLIST_PATH: &str = "docs/parity/checklist.md";

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
mod release_evidence;
mod release_gate;
mod safety_allow;
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
        CliCommand::VerifySettingsDurability(args) => {
            run_phase33_classify_command(args, &environment)?
        }
        CliCommand::ClassifyCorrelatedFlash(args) => {
            run_classify_phase35_flash_command(args, &environment)?
        }
        CliCommand::ClassifyCorrelatedHttp(args) => {
            run_classify_phase35_http_command(args, &environment)?
        }
        CliCommand::ProbeCorrelatedHttp(args) => {
            run_probe_phase35_http_command(args, &environment)?
        }
        CliCommand::ValidateCorrelatedRuntimeEvidence(args) => {
            run_validate_phase35_evidence_command(args, &environment)?
        }
        CliCommand::AdmitCorrelatedRuntimeEvidence(args) => {
            run_admit_phase35_evidence_command(args, &environment)?
        }
    };

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}")?;

    Ok(())
}
