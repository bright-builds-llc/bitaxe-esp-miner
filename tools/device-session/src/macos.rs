use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::{DevicePhase, PhysicalMatch, SessionEvent};

mod receive_only;
mod reconnecting;
mod worker_evidence;

pub(crate) use receive_only::ReceiveOnlyReader;
pub(crate) use reconnecting::capture_reconnecting_receive_only;

const IOREG: &str = "/usr/sbin/ioreg";
const LSOF: &str = "/usr/sbin/lsof";

#[derive(Debug, Clone)]
struct Candidate {
    port: String,
    physical_identity_digest: String,
    enumeration_token: String,
    vendor: String,
    product: String,
    product_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UsbDeviceSnapshot {
    pub(crate) port: String,
    pub(crate) physical_identity_digest: String,
    pub(crate) enumeration_token: String,
    pub(crate) accessible: bool,
    pub(crate) holder_count: u16,
}

pub(crate) enum PhysicalSnapshotObservation {
    Absent,
    PhysicalMismatch,
    Match(UsbDeviceSnapshot),
}

pub(crate) struct UsbProfileFields {
    pub(crate) port: String,
    pub(crate) physical_identity_digest: String,
    pub(crate) enumeration_token: String,
    pub(crate) vendor: String,
    pub(crate) product: String,
    pub(crate) product_name: Option<String>,
}

#[derive(Debug, Default)]
struct NodeFields {
    usb_node: bool,
    vendor: Option<String>,
    product: Option<String>,
    serial: Option<String>,
    location: Option<String>,
    product_name: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DeviceObservation {
    pub(crate) event: SessionEvent,
    pub(crate) maybe_port: Option<String>,
}

pub(crate) struct MacOsDeviceAdapter;

impl MacOsDeviceAdapter {
    pub(crate) fn maybe_profile_fields(port: &str) -> Result<Option<UsbProfileFields>> {
        let candidates = scan_candidates()?;
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.port == port)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [candidate] => Ok(Some(UsbProfileFields {
                port: candidate.port.clone(),
                physical_identity_digest: candidate.physical_identity_digest.clone(),
                enumeration_token: candidate.enumeration_token.clone(),
                vendor: candidate.vendor.clone(),
                product: candidate.product.clone(),
                product_name: candidate.product_name.clone(),
            })),
            _ => bail!("multiple USB profiles matched the selected device node"),
        }
    }
    pub(crate) fn candidate_ports() -> Result<Vec<String>> {
        scan_candidates().map(|candidates| {
            candidates
                .into_iter()
                .map(|candidate| candidate.port)
                .collect()
        })
    }

    pub(crate) fn maybe_exact_snapshot(port: &str) -> Result<Option<UsbDeviceSnapshot>> {
        let candidates = scan_candidates()?;
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.port == port)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [candidate] => Ok(Some(snapshot(candidate)?)),
            _ => bail!("multiple USB candidates matched the selected device node"),
        }
    }

    pub(crate) fn maybe_physical_snapshot(
        expected_physical_identity: &str,
    ) -> Result<Option<UsbDeviceSnapshot>> {
        let candidates = scan_candidates()?;
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.physical_identity_digest == expected_physical_identity)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [candidate] => Ok(Some(snapshot(candidate)?)),
            _ => bail!("multiple USB candidates matched the admitted physical device"),
        }
    }

    pub(crate) fn profile_transition_snapshot(
        expected_physical_identity: &str,
        previous_port: &str,
    ) -> Result<PhysicalSnapshotObservation> {
        let candidates = scan_candidates()?;
        let matches = candidates
            .iter()
            .filter(|candidate| candidate.physical_identity_digest == expected_physical_identity)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [candidate] => return Ok(PhysicalSnapshotObservation::Match(snapshot(candidate)?)),
            [_, _, ..] => bail!("multiple USB candidates matched the admitted physical device"),
            [] => {}
        }
        if candidates.iter().any(|candidate| {
            candidate.port == previous_port
                && candidate.physical_identity_digest != expected_physical_identity
        }) {
            return Ok(PhysicalSnapshotObservation::PhysicalMismatch);
        }
        Ok(PhysicalSnapshotObservation::Absent)
    }

    pub(crate) fn initial_sample(
        admitted_port: &str,
        expected_physical_identity: &str,
    ) -> Result<DeviceObservation> {
        let candidates = scan_candidates()?;
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.port == admitted_port)
            .collect::<Vec<_>>();
        sample_from_candidates(
            DevicePhase::Initial,
            matches,
            expected_physical_identity,
            Some(admitted_port),
        )
    }

    pub(crate) fn recovery_sample(
        expected_physical_identity: &str,
        previous_port: &str,
    ) -> Result<DeviceObservation> {
        let candidates = scan_candidates()?;
        if candidates.iter().any(|candidate| {
            candidate.port == previous_port
                && candidate.physical_identity_digest != expected_physical_identity
        }) {
            return Ok(observation(
                DevicePhase::Recovery,
                PhysicalMatch::UniqueDifferent,
                None,
                false,
                0,
            ));
        }
        let matches = candidates
            .into_iter()
            .filter(|candidate| candidate.physical_identity_digest == expected_physical_identity)
            .collect::<Vec<_>>();
        sample_from_candidates(
            DevicePhase::Recovery,
            matches,
            expected_physical_identity,
            None,
        )
    }

    pub(crate) fn holder_count(port: &str) -> Result<u16> {
        let output = Command::new(LSOF)
            .args(["-t", "--", port])
            .output()
            .context("serial ownership probe failed to launch")?;
        if !output.status.success() {
            if output.status.code() == Some(1) {
                return Ok(0);
            }
            bail!("serial ownership probe failed");
        }
        let text = std::str::from_utf8(&output.stdout)
            .context("serial ownership probe returned invalid text")?;
        let own_pid = std::process::id().to_string();
        let count = text
            .lines()
            .filter(|line| {
                !line.is_empty()
                    && *line != own_pid
                    && line.bytes().all(|byte| byte.is_ascii_digit())
            })
            .count();
        Ok(u16::try_from(count).unwrap_or(u16::MAX))
    }
}

fn snapshot(candidate: &Candidate) -> Result<UsbDeviceSnapshot> {
    Ok(UsbDeviceSnapshot {
        port: candidate.port.clone(),
        physical_identity_digest: candidate.physical_identity_digest.clone(),
        enumeration_token: candidate.enumeration_token.clone(),
        accessible: receive_only_accessible(&candidate.port),
        holder_count: MacOsDeviceAdapter::holder_count(&candidate.port)?,
    })
}

fn sample_from_candidates(
    phase: DevicePhase,
    matches: Vec<Candidate>,
    expected_physical_identity: &str,
    maybe_expected_port: Option<&str>,
) -> Result<DeviceObservation> {
    if matches.is_empty() {
        return Ok(observation(phase, PhysicalMatch::None, None, false, 0));
    }
    if matches.len() > 1 {
        return Ok(observation(phase, PhysicalMatch::Multiple, None, false, 0));
    }
    let Some(candidate) = matches.into_iter().next() else {
        bail!("device candidate disappeared during classification");
    };
    let same_identity = candidate.physical_identity_digest == expected_physical_identity;
    let expected_port_matches = maybe_expected_port.is_none_or(|port| candidate.port == port);
    let physical_match = if same_identity && expected_port_matches {
        PhysicalMatch::UniqueSame
    } else {
        PhysicalMatch::UniqueDifferent
    };
    let accessible = receive_only_accessible(&candidate.port);
    let holder_count = MacOsDeviceAdapter::holder_count(&candidate.port)?;
    Ok(observation(
        phase,
        physical_match,
        Some(candidate),
        accessible,
        holder_count,
    ))
}

fn observation(
    phase: DevicePhase,
    physical_match: PhysicalMatch,
    maybe_candidate: Option<Candidate>,
    accessible: bool,
    holder_count: u16,
) -> DeviceObservation {
    let enumeration_token = maybe_candidate
        .as_ref()
        .map_or_else(String::new, |candidate| candidate.enumeration_token.clone());
    let maybe_port = maybe_candidate.map(|candidate| candidate.port);
    DeviceObservation {
        event: SessionEvent::DeviceSample {
            phase,
            physical_match,
            enumeration_token,
            accessible,
            holder_count,
        },
        maybe_port,
    }
}

fn receive_only_accessible(port: &str) -> bool {
    ReceiveOnlyReader::open(port).is_ok()
}

fn scan_candidates() -> Result<Vec<Candidate>> {
    let output = Command::new(IOREG)
        .args([
            "-p",
            "IOService",
            "-r",
            "-c",
            "IOSerialBSDClient",
            "-l",
            "-w",
            "0",
            "-t",
        ])
        .output()
        .context("macOS USB identity probe failed to launch")?;
    if !output.status.success() {
        bail!("macOS USB identity probe failed");
    }
    let text = std::str::from_utf8(&output.stdout)
        .context("macOS USB identity probe returned invalid text")?;
    parse_ioreg(text)
}

fn parse_ioreg(text: &str) -> Result<Vec<Candidate>> {
    let mut nodes: BTreeMap<usize, NodeFields> = BTreeMap::new();
    let mut current_indent = None;
    let mut candidates = Vec::new();
    for line in text.lines() {
        if let Some(marker) = line.find("+-o ") {
            nodes.retain(|indent, _| *indent < marker);
            nodes.insert(
                marker,
                NodeFields {
                    usb_node: line.contains("class IOUSBHostDevice")
                        || line.contains("class IOUSBHostInterface"),
                    ..NodeFields::default()
                },
            );
            current_indent = Some(marker);
            continue;
        }
        let Some(indent) = current_indent else {
            continue;
        };
        let Some((key, value)) = maybe_parse_property(line) else {
            continue;
        };
        if let Some(node) = nodes.get_mut(&indent) {
            if node.usb_node {
                match key {
                    "idVendor" => node.vendor = Some(value.to_owned()),
                    "idProduct" => node.product = Some(value.to_owned()),
                    "USB Serial Number" => node.serial = Some(value.to_owned()),
                    "locationID" => node.location = Some(value.to_owned()),
                    "USB Product Name" => node.product_name = maybe_unquote(value),
                    _ => {}
                }
            }
        }
        if key != "IOCalloutDevice" {
            continue;
        }
        let Some(port) = maybe_unquote(value) else {
            continue;
        };
        let mut vendor = None;
        let mut product = None;
        let mut serial = None;
        let mut location = None;
        let mut product_name = None;
        for (_, node) in nodes.range(..=indent) {
            if !node.usb_node {
                continue;
            }
            vendor = node.vendor.clone().or(vendor);
            product = node.product.clone().or(product);
            serial = node.serial.clone().or(serial);
            location = node.location.clone().or(location);
            product_name = node.product_name.clone().or(product_name);
        }
        let (Some(vendor), Some(product)) = (vendor, product) else {
            continue;
        };
        if serial.is_none() && location.is_none() {
            continue;
        }
        let mut physical = String::new();
        if let Some(location) = location {
            physical.push_str("locationID=");
            physical.push_str(&location);
            physical.push('\n');
        } else if let Some(serial) = serial {
            physical.push_str("USB Serial Number=");
            physical.push_str(&serial);
            physical.push('\n');
        }
        let metadata = fs::metadata(&port)
            .with_context(|| format!("failed to inspect serial enumeration for {port}"))?;
        let enumeration = format!(
            "port={port}\ndev={}\nino={}\nmode={}\nsize={}\n",
            metadata.dev(),
            metadata.ino(),
            metadata.mode(),
            metadata.size()
        );
        candidates.push(Candidate {
            port,
            physical_identity_digest: sha256(physical.as_bytes()),
            enumeration_token: sha256(enumeration.as_bytes()),
            vendor,
            product,
            product_name,
        });
    }
    Ok(candidates)
}

fn maybe_parse_property(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start_matches([' ', '|']);
    let rest = trimmed.strip_prefix('"')?;
    let (key, value) = rest.split_once("\" = ")?;
    Some((key, value.trim()))
}

fn maybe_unquote(value: &str) -> Option<String> {
    value
        .strip_prefix('"')?
        .strip_suffix('"')
        .map(str::to_owned)
}

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex_lower(&digest.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ioreg_parser_keeps_physical_and_enumeration_identity_separate() {
        // Arrange
        let worker = tempfile::NamedTempFile::new().expect("Worker node must exist");
        let worker_port = worker.path().to_string_lossy();
        let rom = tempfile::NamedTempFile::new().expect("ROM node must exist");
        let rom_port = rom.path().to_string_lossy();
        let worker_fixture = format!(
            "+-o usb  <class IOUSBHostDevice>\n  \"idVendor\" = 0x1209\n  \"idProduct\" = 0xb17a\n  \"USB Product Name\" = \"Bitaxe Ultra 205 BWG Worker\"\n  \"locationID\" = 99\n  +-o serial  <class IOSerialBSDClient>\n    \"IOCalloutDevice\" = \"{worker_port}\"\n"
        );
        let rom_fixture = format!(
            "+-o usb  <class IOUSBHostDevice>\n  \"idVendor\" = 0x303a\n  \"idProduct\" = 0x1001\n  \"USB Product Name\" = \"USB JTAG/serial debug unit\"\n  \"USB Serial Number\" = \"stable\"\n  \"locationID\" = 99\n  +-o serial  <class IOSerialBSDClient>\n    \"IOCalloutDevice\" = \"{rom_port}\"\n"
        );

        // Act
        let worker = parse_ioreg(&worker_fixture).expect("Worker fixture must parse");
        let rom = parse_ioreg(&rom_fixture).expect("ROM fixture must parse");

        // Assert
        assert_eq!(
            worker[0].physical_identity_digest,
            rom[0].physical_identity_digest
        );
        assert_ne!(worker[0].enumeration_token, rom[0].enumeration_token);
        assert_ne!(worker[0].vendor, rom[0].vendor);
        assert_ne!(worker[0].product_name, rom[0].product_name);
    }

    #[test]
    fn ioreg_parser_accepts_branch_prefixed_nested_properties() {
        // Arrange
        let port = tempfile::NamedTempFile::new().expect("temporary node must exist");
        let port = port.path().to_string_lossy();
        let fixture = format!(
            "+-o usb  <class IOUSBHostDevice>\n  | \"idVendor\" = 1234\n  | \"idProduct\" = 5678\n  | \"USB Serial Number\" = \"stable\"\n  | +-o interface  <class IOUSBHostInterface>\n  |   | \"idVendor\" = 1234\n  |   | \"idProduct\" = 5678\n  |   | +-o serial  <class IOSerialBSDClient>\n  |   |   \"IOCalloutDevice\" = \"{port}\"\n"
        );

        // Act
        let candidates = parse_ioreg(&fixture).expect("nested fixture must parse");

        // Assert
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].port, port);
    }

    #[test]
    fn ioreg_parser_canonicalizes_paired_serial_aliases_to_callout() {
        // Arrange
        let callout = tempfile::NamedTempFile::new().expect("callout node must exist");
        let callout = callout.path().to_string_lossy();
        let dialin = tempfile::NamedTempFile::new().expect("dial-in node must exist");
        let dialin = dialin.path().to_string_lossy();
        let fixture = format!(
            "+-o usb  <class IOUSBHostDevice>\n  | \"idVendor\" = 1234\n  | \"idProduct\" = 5678\n  | \"USB Serial Number\" = \"stable\"\n  | +-o serial  <class IOSerialBSDClient>\n  |   \"IOCalloutDevice\" = \"{callout}\"\n  |   \"IODialinDevice\" = \"{dialin}\"\n"
        );

        // Act
        let candidates = parse_ioreg(&fixture).expect("paired aliases must parse");

        // Assert
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].port, callout);
    }
}
