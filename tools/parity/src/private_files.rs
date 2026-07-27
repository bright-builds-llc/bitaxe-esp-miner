use crate::*;

pub(crate) fn validate_private_input(path: &Utf8Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path.as_std_path())
        .context("failed to inspect private Phase 35 HTTP input")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("Phase 35 HTTP diagnostic input must be a non-aliased regular file");
    }
    if metadata.permissions().mode() & 0o777 != 0o600 {
        bail!("Phase 35 HTTP diagnostic input must have mode 0600");
    }
    validate_private_parent(path)?;
    let _ = canonical_private_path(path)?;
    Ok(metadata)
}

pub(crate) fn validate_private_output(path: &Utf8Path) -> Result<Utf8PathBuf> {
    if fs::symlink_metadata(path.as_std_path()).is_ok() {
        bail!("Phase 35 HTTP diagnostic output must not pre-exist");
    }
    if let Err(error) = fs::symlink_metadata(path.as_std_path()) {
        if error.kind() != ErrorKind::NotFound {
            return Err(error).context("failed to inspect private Phase 35 HTTP output");
        }
    }
    validate_private_parent(path)?;
    let parent = path
        .parent()
        .context("Phase 35 HTTP diagnostic output has no parent")?;
    let file_name = path
        .file_name()
        .context("Phase 35 HTTP diagnostic output has no file name")?;
    Ok(canonical_private_path(parent)?.join(file_name))
}

pub(crate) fn validate_private_parent(path: &Utf8Path) -> Result<()> {
    let parent = path
        .parent()
        .context("Phase 35 HTTP diagnostic path has no parent")?;
    let metadata = fs::symlink_metadata(parent.as_std_path())
        .context("failed to inspect private Phase 35 HTTP parent")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Phase 35 HTTP diagnostic parent must be a non-aliased directory");
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("Phase 35 HTTP diagnostic parent must have mode 0700");
    }
    let _ = canonical_private_path(parent)?;
    Ok(())
}

pub(crate) fn canonical_private_path(path: &Utf8Path) -> Result<Utf8PathBuf> {
    let canonical = fs::canonicalize(path.as_std_path())
        .context("failed to canonicalize private Phase 35 HTTP path")?;
    Utf8PathBuf::from_path_buf(canonical)
        .map_err(|_| anyhow::anyhow!("private Phase 35 HTTP path is not valid UTF-8"))
}

pub(crate) fn write_private_new(path: &Utf8Path, contents: &[u8]) -> Result<()> {
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path.as_std_path())
        .context("failed to create private Phase 35 HTTP output")?;
    output
        .write_all(contents)
        .context("failed to write private Phase 35 HTTP output")?;
    output
        .sync_all()
        .context("failed to sync private Phase 35 HTTP output")?;
    if output
        .metadata()
        .context("failed to inspect private Phase 35 HTTP output")?
        .permissions()
        .mode()
        & 0o777
        != 0o600
    {
        bail!("Phase 35 HTTP diagnostic output must have mode 0600");
    }
    Ok(())
}
