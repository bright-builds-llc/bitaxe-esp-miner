use std::collections::BTreeMap;
use std::fs;

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};

use super::{OperatorEvidenceDocuments, OperatorEvidenceProfile};

const FORBIDDEN_SENTINELS: &[&str] = &[
    "stratum+tcp://sentinel-pool.invalid:3333",
    "bc1qsentinelowneraddress.bitaxe",
    "sentinel-password",
    "target=00000000sentinel",
    "extranonce=sentinel-extra",
    "share_payload=sentinel-share",
    "socket_error=sentinel-private-host",
    "device_url=http://192.0.2.55",
    "ip=192.0.2.55",
    "mac=aa:bb:cc:dd:ee:ff",
    "ssid=SentinelWifi",
    "token=sentinel-token",
    "nvs_secret=sentinel-nvs",
    "raw_bm1366_frame=aa55sentinel",
];

#[derive(Debug)]
pub(super) enum OperatorEvidenceArtifact {
    Directory,
    RegularFile(Vec<u8>),
}

pub(super) fn load_operator_evidence_artifacts(
    evidence_root: &Utf8Path,
    directory: &Utf8Path,
    artifacts: &mut BTreeMap<Utf8PathBuf, OperatorEvidenceArtifact>,
) -> Result<()> {
    for entry in fs::read_dir(directory.as_std_path())
        .with_context(|| format!("failed to inventory operator evidence directory {directory}"))?
    {
        let entry = entry.with_context(|| {
            format!("failed to inventory operator evidence directory {directory}")
        })?;
        let absolute_path = Utf8PathBuf::from_path_buf(entry.path())
            .map_err(|_| anyhow::anyhow!("operator evidence contains a non-UTF-8 path"))?;
        let relative_path = absolute_path
            .strip_prefix(evidence_root)
            .context("operator evidence entry escaped its root")?
            .to_owned();
        let metadata = fs::symlink_metadata(absolute_path.as_std_path())
            .context("failed to inspect operator evidence artifact")?;
        if metadata.file_type().is_symlink() {
            bail!("operator evidence must not contain symlinks");
        }
        if metadata.is_dir() {
            artifacts.insert(relative_path, OperatorEvidenceArtifact::Directory);
            load_operator_evidence_artifacts(evidence_root, &absolute_path, artifacts)?;
            continue;
        }
        if !metadata.is_file() {
            bail!("operator evidence contains a non-regular artifact");
        }
        let bytes = fs::read(absolute_path.as_std_path())
            .context("failed to read operator evidence artifact")?;
        artifacts.insert(relative_path, OperatorEvidenceArtifact::RegularFile(bytes));
    }
    Ok(())
}

pub(super) fn validate_artifact_inventory(
    validation_errors: &mut Vec<String>,
    profile: OperatorEvidenceProfile,
    documents: &OperatorEvidenceDocuments,
) {
    let allowed = profile.descriptor().root_entries();
    for (relative_path, artifact) in &documents.artifacts {
        let Some(top_name) = relative_path.components().next().map(|part| part.as_str()) else {
            validation_errors.push("operator evidence contains an empty artifact path".to_owned());
            continue;
        };
        let maybe_entry = allowed
            .iter()
            .copied()
            .find(|entry| entry.name() == top_name);
        let Some(entry) = maybe_entry else {
            validation_errors.push(format!(
                "operator evidence contains unknown artifact {relative_path}"
            ));
            continue;
        };
        let is_top_level = relative_path.as_str() == top_name;
        if !is_top_level && !entry.is_directory() {
            validation_errors.push(format!(
                "operator evidence artifact {relative_path} is nested below a regular-file entry"
            ));
            continue;
        }
        if is_top_level
            && matches!(artifact, OperatorEvidenceArtifact::Directory) != entry.is_directory()
        {
            validation_errors.push(format!(
                "operator evidence artifact {relative_path} has the wrong file type"
            ));
        }
    }
}

pub(super) fn validate_artifact_redaction(
    validation_errors: &mut Vec<String>,
    documents: &OperatorEvidenceDocuments,
) {
    for (relative_path, artifact) in &documents.artifacts {
        let OperatorEvidenceArtifact::RegularFile(bytes) = artifact else {
            continue;
        };
        let contents = String::from_utf8_lossy(bytes);
        let contains_sentinel = FORBIDDEN_SENTINELS
            .iter()
            .any(|sentinel| contents.contains(sentinel));
        if contains_sentinel || contains_private_runtime_value(&contents) {
            validation_errors.push(format!(
                "{relative_path} contains a forbidden redaction sentinel or private runtime value"
            ));
        }
    }
}

fn contains_private_runtime_value(contents: &str) -> bool {
    contents.lines().any(|line| {
        let normalized = line.trim().to_ascii_lowercase();
        let forbidden_assignment = [
            "device_url=http://",
            "device_url=https://",
            "device_url: http://",
            "device_url: https://",
            "ipv4=",
            "ip=",
            "mac=",
            "ssid=",
            "nvs_secret=",
            "raw_bm1366_frame=",
            "poolurl",
            "pooluser",
            "poolpassword",
            "share_payload=",
            "extranonce=",
        ]
        .iter()
        .any(|prefix| normalized.contains(prefix));
        forbidden_assignment
            || contains_unredacted_wifi_ssid(&normalized)
            || contains_unredacted_url(&normalized)
            || normalized.contains("/dev/cu.")
            || normalized.contains("/dev/tty")
            || line
                .split_ascii_whitespace()
                .any(looks_like_private_network_token)
    })
}

fn contains_unredacted_wifi_ssid(normalized: &str) -> bool {
    [
        ("wifi:connected with ", Some(", aid =")),
        ("connected to ssid:", None),
    ]
    .iter()
    .any(|(marker, maybe_delimiter)| {
        normalized.match_indices(marker).any(|(index, _)| {
            let remainder = normalized[index + marker.len()..].trim_start();
            let value = maybe_delimiter
                .and_then(|delimiter| remainder.find(delimiter))
                .map_or(remainder, |delimiter_index| &remainder[..delimiter_index]);
            value.trim() != "[redacted-ssid]"
        })
    })
}

fn contains_unredacted_url(normalized: &str) -> bool {
    [
        "stratum+tcp://",
        "stratum+ssl://",
        "https://",
        "http://",
        "wss://",
        "ws://",
        "tls://",
        "tcp://",
    ]
    .iter()
    .any(|scheme| {
        normalized.match_indices(scheme).any(|(index, _)| {
            let value = &normalized[index + scheme.len()..];
            let authority_end = value
                .find(is_uri_authority_delimiter)
                .unwrap_or(value.len());
            let authority = &value[..authority_end];
            !matches!(
                authority,
                "[redacted]" | "[redacted-url]" | "[redacted-host]" | "[redacted-ip]"
            )
        })
    })
}

fn is_uri_authority_delimiter(character: char) -> bool {
    character.is_ascii_whitespace()
        || matches!(
            character,
            '/' | '?' | '#' | '"' | '\'' | ',' | ';' | ')' | '}' | '>' | '`'
        )
}

fn looks_like_private_network_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|character: char| {
        matches!(
            character,
            '"' | '\'' | ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}'
        )
    });
    let ipv4_parts = trimmed.split('.').collect::<Vec<_>>();
    if ipv4_parts.len() == 4
        && ipv4_parts
            .iter()
            .all(|part| !part.is_empty() && part.parse::<u8>().is_ok())
    {
        return true;
    }
    let mac_parts = trimmed.split(':').collect::<Vec<_>>();
    mac_parts.len() == 6
        && mac_parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|character| character.is_ascii_hexdigit())
        })
}
