//! Shared filesystem operations for the ordinary single-output evidence writer.
//! Permissions intentionally come from its owned caller's process policy.
use std::fs;
use std::io;
use std::path::Path;

pub(crate) enum WriteFailure<'a> {
    CreateDirectory(&'a Path, io::Error),
    WriteFile(io::Error),
}

pub(crate) fn write<'a>(path: &'a Path, contents: &str) -> Result<(), WriteFailure<'a>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| WriteFailure::CreateDirectory(parent, error))?;
    }
    fs::write(path, contents).map_err(WriteFailure::WriteFile)
}
