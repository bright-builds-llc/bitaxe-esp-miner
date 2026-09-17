//! Select the private V2 grammar before legacy Clap diagnostics can echo input.
use anyhow::{bail, Result};

pub(crate) fn maybe_v2_dispatch() -> Option<Result<()>> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    raw.iter()
        .any(|value| value == "v2-serial" || value == "--mode=v2-serial")
        .then(|| {
            crate::v2_serial::run(&raw).map_err(|_| anyhow::anyhow!("v2_serial_fixture_rejected"))
        })
}

pub(crate) fn validate_legacy_options(args: &crate::Args) -> Result<()> {
    if args.attempt_id.is_some()
        || args.read_timeout_seconds.is_some()
        || args.lifetime_seconds.is_some()
    {
        bail!("serial-only fixture options require noise-serial mode");
    }
    Ok(())
}
