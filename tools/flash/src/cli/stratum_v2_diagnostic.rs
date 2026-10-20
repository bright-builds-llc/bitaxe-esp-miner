use crate::*;

#[derive(Debug, Parser, Clone)]
pub(crate) struct TcpPayloadDiagnosticCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,
    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,
    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) intent: Utf8PathBuf,
    #[arg(long = "capture-timeout-seconds", default_value_t = 360)]
    pub(crate) capture_timeout_seconds: u64,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct NoiseDiagnosticCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: String,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,

    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) intent: Utf8PathBuf,

    #[arg(long = "capture-timeout-seconds", default_value_t = 120)]
    pub(crate) capture_timeout_seconds: u64,

    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}
