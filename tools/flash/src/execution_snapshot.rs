use crate::*;

pub(crate) struct AdmittedExecutionSnapshot {
    pub(crate) _file: tempfile::NamedTempFile,
    pub(crate) path: Utf8PathBuf,
}

impl AdmittedExecutionSnapshot {
    pub(crate) fn materialize(bytes: &[u8]) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_create_failed")
        })?;
        file.as_file_mut().write_all(bytes).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file_mut().flush().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file().sync_all().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_sync_failed")
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = file
                .as_file()
                .metadata()
                .map_err(|_| {
                    anyhow::anyhow!(
                        "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                    )
                })?
                .permissions();
            permissions.set_mode(0o600);
            file.as_file().set_permissions(permissions).map_err(|_| {
                anyhow::anyhow!(
                    "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                )
            })?;
        }
        let path = Utf8PathBuf::from_path_buf(file.path().to_path_buf()).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_path_invalid")
        })?;

        Ok(Self { _file: file, path })
    }

    pub(crate) fn path(&self) -> &Utf8Path {
        &self.path
    }
}
