use std::time::Duration;
use std::{env, fs};

use anyhow::{Context, Result};
use bitaxe_device_session::{
    create_empty_private_root, finalize_display_uat, observe_usb_reboot_loop,
    run_admitted_inspection, run_admitted_transaction, run_display_uat_live, run_fixture_session,
    run_live_session, validate_private_input, DeviceInspectionIntent, DeviceTransactionIntent,
    DisplayUatIntent, FixtureTranscript, InspectionArtifacts, OtaIntent, RebootIntent,
    SessionArtifacts, SessionRequest, TerminalCategory, TransactionGoal, UsbRuntimeIdentity,
};
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "device-session")]
#[command(about = "Deterministic receive-only ESP device session owner.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Reboot(RebootArgs),
    #[command(name = "reboot-live")]
    RebootLive(RebootLiveArgs),
    #[command(name = "ota-live")]
    OtaLive(OtaLiveArgs),
    #[command(name = "transact")]
    Transact(TransactArgs),
    #[command(name = "transact-live")]
    TransactLive(TransactLiveArgs),
    #[command(name = "inspect-live")]
    InspectLive(InspectLiveArgs),
    #[command(name = "display-uat-live")]
    DisplayUatLive(DisplayUatLiveArgs),
    #[command(name = "display-uat-finalize")]
    DisplayUatFinalize(DisplayUatFinalizeArgs),
    #[command(name = "observe-usb-reboot-loop")]
    ObserveUsbRebootLoop(ObserveUsbRebootLoopArgs),
}

#[derive(Debug, Args)]
struct RebootArgs {
    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "request-input")]
    request_input: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "fixture-input")]
    fixture_input: Option<Utf8PathBuf>,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct RebootLiveArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct OtaLiveArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "ota-image")]
    ota_image: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct TransactArgs {
    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "fixture-input")]
    fixture_input: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct TransactLiveArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "ota-image")]
    maybe_ota_image: Option<Utf8PathBuf>,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct InspectLiveArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 30)]
    timeout_seconds: u64,
}

#[derive(Debug, Args)]
struct DisplayUatLiveArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "intent-input")]
    intent_input: Utf8PathBuf,

    #[arg(long = "runtime-observation-input")]
    runtime_observation_input: Utf8PathBuf,

    #[arg(long = "programmatic-evidence")]
    programmatic_evidence: Utf8PathBuf,
}

#[derive(Debug, Args)]
struct DisplayUatFinalizeArgs {
    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,
}

#[derive(Debug, Args)]
struct ObserveUsbRebootLoopArgs {
    #[arg(long)]
    port: String,

    #[arg(long = "timeout-seconds", default_value_t = 15)]
    timeout_seconds: u64,

    #[arg(
        long = "expected-source-commit",
        requires = "maybe_expected_app_elf_sha256"
    )]
    maybe_expected_source_commit: Option<String>,

    #[arg(
        long = "expected-app-elf-sha256",
        requires = "maybe_expected_source_commit"
    )]
    maybe_expected_app_elf_sha256: Option<String>,
}

fn main() {
    match run() {
        Ok(TerminalCategory::Ready) => {}
        Ok(category) => {
            eprintln!(
                "device_session_status=failed category={}",
                category.as_str()
            );
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("device_session_status=failed category=host_error");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<TerminalCategory> {
    let cli = Cli::parse();
    match cli.command {
        Command::Reboot(args) => run_reboot(args),
        Command::RebootLive(args) => run_reboot_live(args),
        Command::OtaLive(args) => run_ota_live(args),
        Command::Transact(args) => run_transact(args),
        Command::TransactLive(args) => run_transact_live(args),
        Command::InspectLive(args) => run_inspect_live(args),
        Command::DisplayUatLive(args) => run_display_uat(args),
        Command::DisplayUatFinalize(args) => run_display_uat_finalize(args),
        Command::ObserveUsbRebootLoop(args) => run_observe_usb_reboot_loop(args),
    }
}

fn run_observe_usb_reboot_loop(args: ObserveUsbRebootLoopArgs) -> Result<TerminalCategory> {
    if args.timeout_seconds == 0 || args.timeout_seconds > 30 {
        anyhow::bail!("timeout-seconds must be between 1 and 30");
    }
    let maybe_expected = args
        .maybe_expected_source_commit
        .as_deref()
        .zip(args.maybe_expected_app_elf_sha256.as_deref())
        .map(|(commit, digest)| UsbRuntimeIdentity::new(commit, digest))
        .transpose()?;
    let observation =
        observe_usb_reboot_loop(&args.port, Duration::from_secs(args.timeout_seconds))?;
    println!("usb_reboot_loop: {}", observation.category().label());
    println!("marker_count: {}", observation.marker_count());
    println!("reconnect_count: {}", observation.reconnect_count());
    println!("latest_boot_ordinal: {}", observation.latest_boot_ordinal());
    println!(
        "latest_reset_reason: {}",
        observation.latest_reset_reason().label()
    );
    match observation.latest_rust_panic() {
        Some(marker) => {
            println!("rust_panic_receipt: present");
            println!("rust_panic_file_hash: {:08x}", marker.file_hash());
            println!("rust_panic_line: {}", marker.line());
        }
        None => println!("rust_panic_receipt: missing"),
    }
    match observation.latest_allocation_failure() {
        Some(marker) => {
            println!("allocation_failure_receipt: present");
            println!("allocation_requested_bytes: {}", marker.requested_bytes());
            println!("allocation_capabilities: {:08x}", marker.capabilities());
        }
        None => println!("allocation_failure_receipt: missing"),
    }
    if let Some(context) = observation.maybe_allocation_context() {
        println!("{}", context.marker());
    }
    match observation.maybe_runtime_identity() {
        Some(identity) => {
            println!("runtime_firmware_commit: {}", identity.firmware_commit);
            println!("runtime_app_elf_sha256: {}", identity.app_elf_sha256);
        }
        None => println!("runtime_identity: missing"),
    }
    if let Some(startup) = observation.maybe_startup_progress() {
        println!("{}", startup.marker());
    }
    for checkpoint in observation.memory_checkpoints() {
        println!("usb_memory_checkpoint stage={} free_bytes={} largest_block_bytes={} reserve_bytes={} redacted=true",
            checkpoint.stage, checkpoint.free_bytes, checkpoint.largest_block_bytes, checkpoint.reserve_bytes);
    }
    println!(
        "worker_start_failure_observed: {}",
        observation.worker_start_failed()
    );

    if let Some(expected) = maybe_expected {
        if let Err(error) = observation.require_identity(&expected) {
            eprintln!("{error}");
            return Err(error);
        }
        println!("runtime_identity: exact_match");
    }
    Ok(TerminalCategory::Ready)
}

fn validate_timeout(timeout_seconds: u64) -> Result<()> {
    if timeout_seconds == 0 || timeout_seconds > 600 {
        anyhow::bail!("timeout-seconds must be between 1 and 600");
    }
    Ok(())
}

fn run_reboot(args: RebootArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    validate_private_input(&args.request_input)?;
    let request_bytes = fs::read(args.request_input.as_std_path())
        .context("failed to read private request input")?;
    let request: SessionRequest = serde_json::from_slice(&request_bytes)
        .context("private request input does not match the device-session schema")?;
    if let Some(fixture_input) = args.fixture_input {
        validate_private_input(&fixture_input)?;
        let fixture_bytes = fs::read(fixture_input.as_std_path())
            .context("failed to read private fixture input")?;
        let fixture: FixtureTranscript = serde_json::from_slice(&fixture_bytes)
            .context("private fixture input does not match the device-session fixture schema")?;
        let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
        return run_fixture_session(request, fixture, artifacts);
    }
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_live_session(
        request,
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}

fn run_reboot_live(args: RebootLiveArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    validate_private_input(&args.intent_input)?;
    let intent_bytes = fs::read(args.intent_input.as_std_path())
        .context("failed to read private reboot intent")?;
    let intent: RebootIntent = serde_json::from_slice(&intent_bytes)
        .context("private reboot intent does not match the device-session schema")?;
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_admitted_transaction(
        DeviceTransactionIntent::restart(intent),
        args.port,
        None,
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}

fn run_ota_live(args: OtaLiveArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    validate_private_input(&args.intent_input)?;
    let intent_bytes =
        fs::read(args.intent_input.as_std_path()).context("failed to read private OTA intent")?;
    let intent: OtaIntent = serde_json::from_slice(&intent_bytes)
        .context("private OTA intent does not match the device-session schema")?;
    let ota_image = fs::read(args.ota_image.as_std_path()).context("failed to read OTA image")?;
    if ota_image.is_empty() || ota_image.len() > 4 * 1024 * 1024 {
        anyhow::bail!("OTA image size is outside the admitted partition bound");
    }
    if !intent.image_matches(&ota_image) {
        anyhow::bail!("OTA image identity is invalid");
    }
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_admitted_transaction(
        DeviceTransactionIntent::ota_transition(intent),
        args.port,
        Some(ota_image),
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}

fn read_transaction_intent(path: &Utf8PathBuf) -> Result<DeviceTransactionIntent> {
    validate_private_input(path)?;
    let bytes =
        fs::read(path.as_std_path()).context("failed to read private transaction intent")?;
    serde_json::from_slice(&bytes)
        .context("private transaction intent does not match the device-session schema")
}

fn run_transact(args: TransactArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    let intent = read_transaction_intent(&args.intent_input)?;
    if !intent.schema_is_valid() {
        anyhow::bail!("device transaction intent schema is invalid");
    }
    validate_private_input(&args.fixture_input)?;
    let fixture_bytes = fs::read(args.fixture_input.as_std_path())
        .context("failed to read private fixture input")?;
    let fixture: FixtureTranscript = serde_json::from_slice(&fixture_bytes)
        .context("private fixture input does not match the device-session fixture schema")?;
    let request = match intent.goal {
        TransactionGoal::CommandEffects { reboot }
        | TransactionGoal::SettingsDurability { reboot }
        | TransactionGoal::Restart { reboot } => {
            reboot.bind_device("fixture-only".to_owned(), "f".repeat(64))
        }
        TransactionGoal::OtaTransition { ota } => {
            ota.bind_device("fixture-only".to_owned(), "f".repeat(64))
        }
    };
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_fixture_session(request, fixture, artifacts)
}

fn run_transact_live(args: TransactLiveArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    let intent = read_transaction_intent(&args.intent_input)?;
    if !intent.schema_is_valid() {
        anyhow::bail!("device transaction intent schema is invalid");
    }
    if intent.requires_ota_image() != args.maybe_ota_image.is_some() {
        anyhow::bail!("transaction OTA image presence does not match its goal");
    }
    let maybe_ota_image = match args.maybe_ota_image {
        Some(path) => {
            let bytes = fs::read(path.as_std_path()).context("failed to read OTA image")?;
            if bytes.is_empty() || bytes.len() > 4 * 1024 * 1024 {
                anyhow::bail!("OTA image size is outside the admitted partition bound");
            }
            Some(bytes)
        }
        None => None,
    };
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_admitted_transaction(
        intent,
        args.port,
        maybe_ota_image,
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}

fn run_inspect_live(args: InspectLiveArgs) -> Result<TerminalCategory> {
    validate_timeout(args.timeout_seconds)?;
    validate_private_input(&args.intent_input)?;
    let bytes = fs::read(args.intent_input.as_std_path())
        .context("failed to read private inspection intent")?;
    let intent: DeviceInspectionIntent = serde_json::from_slice(&bytes)
        .context("private inspection intent does not match the device-session schema")?;
    if !intent.schema_is_valid() {
        anyhow::bail!("device inspection intent schema is invalid");
    }
    let artifacts = InspectionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_admitted_inspection(
        intent,
        args.port,
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}

fn run_display_uat(args: DisplayUatLiveArgs) -> Result<TerminalCategory> {
    let intent_input = resolve_workspace_path(args.intent_input)?;
    let runtime_observation_input = resolve_workspace_path(args.runtime_observation_input)?;
    let programmatic_evidence = resolve_workspace_path(args.programmatic_evidence)?;
    let private_root = resolve_workspace_path(args.private_root)?;
    validate_private_input(&intent_input)?;
    validate_private_input(&runtime_observation_input)?;
    let intent: DisplayUatIntent =
        serde_json::from_slice(&fs::read(intent_input.as_std_path())?)
            .context("private display UAT intent does not match the device-session schema")?;
    let evidence = fs::read(programmatic_evidence.as_std_path())
        .context("failed to read programmatic command evidence")?;
    let runtime_observation = fs::read(runtime_observation_input.as_std_path())
        .context("failed to read private runtime observation")?;
    // The live command owns its fresh attempt root so callers cannot race a
    // permissive or pre-populated directory into the evidence transaction.
    create_empty_private_root(&private_root)?;
    run_display_uat_live(
        intent,
        args.port,
        &runtime_observation,
        &evidence,
        &private_root,
    )
}

fn run_display_uat_finalize(args: DisplayUatFinalizeArgs) -> Result<TerminalCategory> {
    let private_root = resolve_workspace_path(args.private_root)?;
    let projection_output = resolve_workspace_path(args.projection_output)?;
    finalize_display_uat(&private_root, &projection_output)?;
    Ok(TerminalCategory::Ready)
}

fn resolve_workspace_path(path: Utf8PathBuf) -> Result<Utf8PathBuf> {
    if path.is_absolute() {
        return Ok(path);
    }
    if let Ok(workspace) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        return Ok(Utf8PathBuf::from(workspace).join(path));
    }
    let current = env::current_dir().context("failed to resolve current working directory")?;
    let current = Utf8PathBuf::from_path_buf(current)
        .map_err(|_| anyhow::anyhow!("current working directory is not UTF-8"))?;
    Ok(current.join(path))
}
