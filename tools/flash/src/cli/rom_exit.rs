use crate::*;

#[derive(Debug, Parser, Clone)]
pub(crate) struct StartInstalledCommand {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Option<Utf8PathBuf>,
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "expected-source-commit")]
    pub(crate) expected_source_commit: String,
    #[arg(long = "expected-app-elf-sha256")]
    pub(crate) expected_app_elf_sha256: String,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct RomExitDiagnosticCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "package-manifest", value_parser = parse_utf8_path)]
    pub(crate) package_manifest: Utf8PathBuf,
    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "observation-seconds", default_value_t = 30)]
    pub(crate) observation_seconds: u64,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}
