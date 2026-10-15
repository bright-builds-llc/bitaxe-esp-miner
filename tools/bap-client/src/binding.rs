//! Binds a sealed BAP probe record to the firmware image the board was running.
//!
//! BAP cannot name the installed firmware, but the board's native USB console
//! can: the receive-only observer (`just diagnose-usb-reboot-loop`) reports the
//! running build, the reset-retained boot ordinal, the reset reason and the
//! device uptime. Reading it before and after the probe, on the same physical
//! device, with the same boot ordinal and reset reason and an uptime that
//! advanced by the host's elapsed time, shows that one boot of that exact build
//! answered every BAP frame in between. Any reset or power cycle in between
//! changes the ordinal or restarts the uptime.

use serde_json::{json, Value};

/// How far the device's uptime advance may differ from the host's elapsed time
/// between the two observations: capture lengths, marker cadence (about 3 s)
/// and file-timestamp resolution.
pub const UPTIME_TOLERANCE_MS: u64 = 15_000;

/// The exact image the contract installed, from the package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub source_commit: String,
    pub app_elf_sha256: String,
}

/// One receive-only observation of the running application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub stable_boot: bool,
    pub exact_match: bool,
    pub boot_ordinal: u64,
    pub reset_reason: String,
    pub uptime_ms: u64,
    pub firmware_commit: String,
    pub app_elf_sha256: String,
}

/// The detector facts the binding needs: the running profile and physical identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectorObservation {
    pub physical_identity_sha256: String,
}

/// Everything the judge compares; the shell gathers it from private files.
pub struct BindingInputs<'a> {
    pub package: &'a Package,
    pub probe: &'a Value,
    pub detector_before: &'a DetectorObservation,
    pub detector_after: &'a DetectorObservation,
    pub runtime_before: &'a RuntimeObservation,
    pub runtime_after: &'a RuntimeObservation,
    pub host_elapsed_ms: u64,
}

/// The value of the only `label: value` line, or `None` when absent or repeated.
fn unique_value<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    let prefix = format!("{label}: ");
    let mut matches = text
        .lines()
        .filter_map(|line| line.strip_prefix(prefix.as_str()));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn required<'a>(
    text: &'a str,
    label: &str,
    category: &'static str,
) -> Result<&'a str, &'static str> {
    unique_value(text, label).ok_or(category)
}

fn number(text: &str, label: &str, category: &'static str) -> Result<u64, &'static str> {
    required(text, label, category)?
        .parse()
        .map_err(|_| category)
}

/// Parses the package manifest; a dirty source never binds.
pub fn parse_package(manifest: &Value) -> Result<Package, &'static str> {
    if manifest["source_dirty"] != json!(false) {
        return Err("package_not_clean");
    }
    let text = |key: &str| {
        manifest[key]
            .as_str()
            .map(str::to_owned)
            .ok_or("package_identity_missing")
    };
    Ok(Package {
        source_commit: text("source_commit")?,
        app_elf_sha256: text("app_elf_sha256")?,
    })
}

/// Parses a saved detector report for a running board.
pub fn parse_detector(text: &str) -> Result<DetectorObservation, &'static str> {
    required(text, "port", "detector_port_not_unique")?;
    if unique_value(text, "usb_profile") != Some("serial_jtag_runtime") {
        return Err("detector_not_runtime");
    }
    Ok(DetectorObservation {
        physical_identity_sha256: required(
            text,
            "physical_identity_sha256",
            "detector_physical_identity_missing",
        )?
        .to_owned(),
    })
}

/// Parses the receive-only observer's console report.
pub fn parse_runtime(text: &str) -> Result<RuntimeObservation, &'static str> {
    const MALFORMED: &str = "runtime_observation_malformed";
    Ok(RuntimeObservation {
        stable_boot: required(text, "usb_reboot_loop", MALFORMED)? == "stable_boot",
        exact_match: unique_value(text, "runtime_identity") == Some("exact_match"),
        boot_ordinal: number(text, "latest_boot_ordinal", MALFORMED)?,
        reset_reason: required(text, "latest_reset_reason", MALFORMED)?.to_owned(),
        uptime_ms: number(text, "latest_uptime_ms", MALFORMED)?,
        firmware_commit: required(text, "runtime_firmware_commit", "runtime_identity_missing")?
            .to_owned(),
        app_elf_sha256: required(text, "runtime_app_elf_sha256", "runtime_identity_missing")?
            .to_owned(),
    })
}

fn runs_package(observation: &RuntimeObservation, package: &Package) -> bool {
    observation.stable_boot
        && observation.exact_match
        && observation.firmware_commit == package.source_commit
        && observation.app_elf_sha256 == package.app_elf_sha256
}

/// The first failed check in contract order, or `None` when the probe is bound.
#[must_use]
pub fn first_failure(inputs: &BindingInputs<'_>) -> Option<&'static str> {
    let probe = inputs.probe;
    let before = inputs.runtime_before;
    let after = inputs.runtime_after;
    let advance = after.uptime_ms.checked_sub(before.uptime_ms);
    let checks = [
        (probe["outcome"] == json!("complete"), "probe_not_complete"),
        (
            probe["source"]["commit"] == json!(inputs.package.source_commit)
                && probe["source"]["clean"] == json!(true),
            "probe_source_not_package",
        ),
        (
            inputs.detector_before == inputs.detector_after,
            "physical_identity_changed",
        ),
        (
            runs_package(before, inputs.package),
            "runtime_before_not_package",
        ),
        (
            runs_package(after, inputs.package),
            "runtime_after_not_package",
        ),
        (
            before.boot_ordinal == after.boot_ordinal && before.reset_reason == after.reset_reason,
            "boot_changed",
        ),
        (
            advance.is_some_and(|advance| {
                advance > 0 && advance.abs_diff(inputs.host_elapsed_ms) <= UPTIME_TOLERANCE_MS
            }),
            "uptime_not_continuous",
        ),
    ];
    checks
        .into_iter()
        .find_map(|(passed, category)| (!passed).then_some(category))
}

/// The sealed binding record: identities that are already public (commit, ELF
/// digest), categories and numbers only; never a path or a physical identity.
#[must_use]
pub fn record(inputs: &BindingInputs<'_>) -> Value {
    let maybe_failure = first_failure(inputs);
    let before = inputs.runtime_before;
    let after = inputs.runtime_after;
    json!({
        "schema": "bap-identity-binding-v1",
        "task": "task-parity-bap-live-accessory-verification",
        "probe_ordinal": inputs.probe["ordinal"],
        "probe_outcome": inputs.probe["outcome"],
        "package": {
            "source_commit": inputs.package.source_commit,
            "app_elf_sha256": inputs.package.app_elf_sha256,
        },
        "same_physical_device": inputs.detector_before == inputs.detector_after,
        "runtime_before_exact": runs_package(before, inputs.package),
        "runtime_after_exact": runs_package(after, inputs.package),
        "boot_ordinal": [before.boot_ordinal, after.boot_ordinal],
        "reset_reason": [before.reset_reason, after.reset_reason],
        "uptime_advance_ms": after.uptime_ms.checked_sub(before.uptime_ms),
        "host_elapsed_ms": inputs.host_elapsed_ms,
        "uptime_tolerance_ms": UPTIME_TOLERANCE_MS,
        "outcome": if maybe_failure.is_none() { "bound" } else { "unbound" },
        "failure": maybe_failure,
    })
}

#[cfg(test)]
mod tests;
