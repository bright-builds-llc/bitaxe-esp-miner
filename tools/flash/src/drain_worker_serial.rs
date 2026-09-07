//! Metadata-only recovery drain. No package, downloader or serial transcript path.
use crate::*;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub(crate) struct DrainWorkerSerialCommand {
    #[arg(long, default_value = "205", value_parser = ["205"])]
    board: String,
    #[arg(long)]
    port: String,
    #[arg(long)]
    evidence_dir: PathBuf,
}
#[derive(Debug, Serialize)]
struct DrainEvidence {
    schema: &'static str,
    profile: &'static str,
    discarded_bytes: Option<usize>,
    elapsed_milliseconds: Option<u128>,
    cleanup_complete: bool,
    failure: Option<&'static str>,
}
fn prepare(command: &DrainWorkerSerialCommand) -> Result<()> {
    if command.board != "205"
        || !command.port.starts_with("/dev/cu.")
        || !command.evidence_dir.is_absolute()
    {
        bail!("drain_invocation_rejected");
    }
    let parent = command
        .evidence_dir
        .parent()
        .context("drain_evidence_parent")?;
    let metadata = fs::symlink_metadata(parent).context("drain_evidence_parent")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & 0o777 != 0o700
        || parent.canonicalize()? != parent
    {
        bail!("drain_evidence_parent_rejected");
    }
    match fs::symlink_metadata(&command.evidence_dir) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        _ => bail!("drain_evidence_must_be_absent"),
    }
    Ok(())
}
pub(crate) fn run(command: &DrainWorkerSerialCommand) -> Result<()> {
    prepare(command)?;
    let workspace = env::var_os("BUILD_WORKSPACE_DIRECTORY")
        .map(PathBuf::from)
        .map_or_else(env::current_dir, Ok)?;
    if !Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["check-ignore", "--quiet", "--"])
        .arg(&command.evidence_dir)
        .status()?
        .success()
    {
        bail!("drain_evidence_not_ignored");
    }
    fs::create_dir(&command.evidence_dir)?;
    fs::set_permissions(&command.evidence_dir, fs::Permissions::from_mode(0o700))?;
    let mut evidence = DrainEvidence {
        schema: "worker-serial-drain-v1",
        profile: "not_admitted",
        discarded_bytes: None,
        elapsed_milliseconds: None,
        cleanup_complete: false,
        failure: None,
    };
    match UsbSession::acquire(UsbOperation::Monitor, &command.port, &command.evidence_dir) {
        Err(error) => evidence.failure = Some(error.category.as_str()),
        Ok(mut session) => {
            match session.drain_worker_serial() {
                Ok(observed) => {
                    evidence.profile = "serial_jtag_runtime";
                    evidence.discarded_bytes = Some(observed.discarded_bytes);
                    evidence.elapsed_milliseconds = Some(observed.elapsed_milliseconds);
                }
                Err(error) => evidence.failure = Some(error.category.as_str()),
            }
            match session.finish() {
                Ok(_) => evidence.cleanup_complete = true,
                Err(error) => {
                    evidence.failure.get_or_insert(error.category.as_str());
                }
            }
        }
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(command.evidence_dir.join("drain.json"))?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    serde_json::to_writer(&mut file, &evidence)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    serde_json::to_writer(io::stdout().lock(), &evidence)?;
    if evidence.failure.is_some() || !evidence.cleanup_complete {
        bail!("drain_failed");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_existing_evidence_and_invalid_board_before_admission() {
        let root = tempfile::tempdir().expect("fixture");
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).expect("mode");
        let parent = root.path().canonicalize().expect("canonical");
        let mut command = DrainWorkerSerialCommand {
            board: "205".to_owned(),
            port: "/dev/cu.fixture".to_owned(),
            evidence_dir: parent.join("new"),
        };
        assert!(prepare(&command).is_ok());
        fs::create_dir(&command.evidence_dir).expect("existing");
        assert!(prepare(&command).is_err());
        command.evidence_dir = parent.join("other");
        command.board = "999".to_owned();
        assert!(prepare(&command).is_err());
    }
    #[test]
    fn cli_has_no_reset_flash_or_timeout_override() {
        assert!(Cli::try_parse_from([
            "flash",
            "drain-worker-serial",
            "--port",
            "/dev/cu.fixture",
            "--evidence-dir",
            "/tmp/new"
        ])
        .is_ok());
        assert!(Cli::try_parse_from([
            "flash",
            "drain-worker-serial",
            "--port",
            "/dev/cu.fixture",
            "--evidence-dir",
            "/tmp/new",
            "--reset"
        ])
        .is_err());
    }
}
