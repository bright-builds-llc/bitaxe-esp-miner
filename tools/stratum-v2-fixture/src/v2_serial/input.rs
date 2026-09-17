use anyhow::{bail, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::PathBuf;
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Scope {
    Channel,
    Share,
}
impl Scope {
    pub fn lifetime_seconds(self) -> u64 {
        if self == Self::Channel {
            150
        } else {
            300
        }
    }
}
pub(super) struct Options {
    pub root: PathBuf,
    pub attempt: String,
    pub scope: Scope,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Input {
    pub schema: String,
    pub scope: Scope,
    pub attempt_id: String,
    pub listen_ipv4: Zeroizing<String>,
    pub expected_peer_ipv4: Zeroizing<String>,
    pub user_identity: Zeroizing<String>,
}
pub(super) fn nonce(value: &str) -> bool {
    URL_SAFE_NO_PAD
        .decode(value)
        .is_ok_and(|bytes| bytes.len() == 16 && URL_SAFE_NO_PAD.encode(bytes) == value)
}
pub(super) fn parse_options(args: &[String]) -> Result<Options> {
    if args.len() != 8 {
        bail!("arguments");
    }
    let mut values = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !["--mode", "--scope", "--private-root", "--attempt-id"].contains(&pair[0].as_str())
            || values.insert(pair[0].as_str(), pair[1].as_str()).is_some()
        {
            bail!("arguments");
        }
    }
    if values.get("--mode") != Some(&"v2-serial") {
        bail!("mode");
    }
    let scope = match values.get("--scope") {
        Some(&"channel") => Scope::Channel,
        Some(&"share") => Scope::Share,
        _ => bail!("scope"),
    };
    let attempt = values
        .get("--attempt-id")
        .filter(|value| nonce(value))
        .ok_or_else(|| anyhow::anyhow!("attempt"))?
        .to_string();
    let root = PathBuf::from(
        values
            .get("--private-root")
            .ok_or_else(|| anyhow::anyhow!("root"))?,
    );
    let parent = root.parent().ok_or_else(|| anyhow::anyhow!("parent"))?;
    if !root.is_absolute()
        || fs::canonicalize(parent)? != parent
        || fs::symlink_metadata(parent)?.permissions().mode() & 0o777 != 0o700
        || fs::symlink_metadata(&root).is_ok()
    {
        bail!("private root");
    }
    Ok(Options {
        root,
        attempt,
        scope,
    })
}
pub(super) fn ipv4(value: &str) -> Result<Ipv4Addr> {
    let address: Ipv4Addr = value.parse().map_err(|_| anyhow::anyhow!("address"))?;
    if address.to_string() != value || !address.is_private() {
        bail!("address");
    }
    Ok(address)
}
pub(super) fn parse_input(bytes: &[u8], options: &Options) -> Result<Input> {
    if bytes.len() > 4096 || !bytes.ends_with(b"\n") || bytes[..bytes.len() - 1].contains(&b'\n') {
        bail!("input framing");
    }
    let input: Input = serde_json::from_slice(bytes).map_err(|_| anyhow::anyhow!("input"))?;
    if input.schema != "str005-v2-fixture-input-v1"
        || input.scope != options.scope
        || input.attempt_id != options.attempt
        || input.user_identity.is_empty()
        || input.user_identity.len() > 255
    {
        bail!("input binding");
    }
    ipv4(&input.listen_ipv4)?;
    ipv4(&input.expected_peer_ipv4)?;
    Ok(input)
}
pub(super) fn require_pipe(file: &File) -> Result<()> {
    let kind = file.metadata()?.file_type();
    if !kind.is_fifo() && !kind.is_socket() {
        bail!("private pipe required");
    }
    Ok(())
}
pub(super) fn read_input(mut file: File, options: &Options) -> Result<Input> {
    require_pipe(&file)?;
    let mut bytes = Zeroizing::new(Vec::new());
    Read::by_ref(&mut file).take(4097).read_to_end(&mut bytes)?;
    parse_input(&bytes, options)
}
pub(super) fn write_pipe(mut pipe: File, value: &impl Serialize) -> Result<()> {
    require_pipe(&pipe)?;
    let mut bytes = Zeroizing::new(serde_json::to_vec(value)?);
    bytes.push(b'\n');
    if bytes.len() > 4096 {
        bail!("pipe bound");
    }
    pipe.write_all(&bytes)?;
    pipe.flush()?;
    Ok(())
}

/// Validate and transfer a fixed inherited pipe, including a closed/missing fd.
/// Duplicating first avoids constructing an OwnedFd from an invalid descriptor.
pub(super) fn inherited_pipe(fd: i32) -> Result<File> {
    use std::os::fd::FromRawFd;
    unsafe extern "C" {
        fn dup(fd: i32) -> i32;
        fn close(fd: i32) -> i32;
    }
    // SAFETY: dup accepts any integer and returns a newly owned descriptor or -1.
    let copied = unsafe { dup(fd) };
    if copied < 0 {
        bail!("private pipe absent");
    }
    // SAFETY: a successful dup returned this unique owned descriptor.
    let file = unsafe { File::from_raw_fd(copied) };
    require_pipe(&file)?;
    // SAFETY: this dedicated mode owns the original inherited descriptor.
    if unsafe { close(fd) } != 0 {
        bail!("private pipe transfer");
    }
    Ok(file)
}
