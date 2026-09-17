//! Test-only executable. Its filesystem writer is the production Rust module.
#[path = "../../tools/flash/src/evidence_output.rs"]
mod evidence_output;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::ExitCode;

fn run() -> Result<(), ()> {
    let mut args = std::env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or(())?);
    if args.next().is_some() {
        return Err(());
    }
    let temporary = fs::canonicalize(std::env::temp_dir()).map_err(|_| ())?;
    let canonical = fs::canonicalize(&root).map_err(|_| ())?;
    if root != canonical || !canonical.starts_with(temporary) {
        return Err(());
    }
    let marker = root.join(".operator-private-output-test");
    if fs::read(&marker).map_err(|_| ())? != b"synthetic-filesystem-only\n"
        || fs::metadata(&root).map_err(|_| ())?.permissions().mode() & 0o777 != 0o700
    {
        return Err(());
    }
    let output = root.join("install-0");
    if output.symlink_metadata().is_ok() {
        return Err(());
    }
    for (name, contents) in [
        ("flash-command-evidence.json", "{\"synthetic\":true}\n"),
        ("nested/flash-monitor.log", "synthetic-runtime-only\n"),
        ("nested/atomic.pending", "synthetic-atomic-output\n"),
    ] {
        evidence_output::write(&output.join(name), contents).map_err(|failure| {
            // Consume the real error without printing paths or supplied input.
            match failure {
                evidence_output::WriteFailure::CreateDirectory(path, error) => {
                    let _observed = (path.as_os_str(), error.kind());
                }
                evidence_output::WriteFailure::WriteFile(error) => {
                    let _observed = error.kind();
                }
            }
        })?;
    }
    fs::rename(output.join("nested/atomic.pending"), output.join("nested/atomic.json"))
        .map_err(|_| ())?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
