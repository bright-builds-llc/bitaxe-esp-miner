//! Web UI variant selection for package builds (ADR-0034).

use crate::*;

/// Web UI implementation packed into `www.bin`; mirrors `//firmware/bitaxe:web_ui`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum WebUiVariant {
    /// The handwritten default UI in `firmware/bitaxe/static/www`.
    Current,
    /// The SolidJS port in `firmware/bitaxe/web/solid`.
    Solid,
}

impl WebUiVariant {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Solid => "solid",
        }
    }

    /// Bazel command-line flag that selects this variant.
    pub(crate) fn bazel_flag(self) -> String {
        format!("--//firmware/bitaxe:web_ui={}", self.as_str())
    }
}

/// The variant a flash must find in its manifest. A package the tool builds
/// itself defaults to `current`; an explicit manifest only needs a recorded
/// variant unless `--web-ui` names one.
pub(crate) fn expected_web_ui_variant(
    maybe_requested: Option<WebUiVariant>,
    builds_package: bool,
) -> Option<WebUiVariant> {
    maybe_requested.or(builds_package.then_some(WebUiVariant::Current))
}

/// Fails closed unless the manifest records a known variant matching the expectation.
pub(crate) fn validate_manifest_web_ui_variant(
    maybe_recorded: Option<&str>,
    maybe_expected: Option<WebUiVariant>,
) -> Result<()> {
    let Some(recorded) = maybe_recorded else {
        bail!("identity_admission=blocked reason=manifest_web_ui_variant_missing");
    };
    let Some(recorded_variant) = <WebUiVariant as ValueEnum>::from_str(recorded, false).ok() else {
        bail!("identity_admission=blocked reason=manifest_web_ui_variant_unknown");
    };
    if maybe_expected.is_some_and(|expected| expected != recorded_variant) {
        bail!("identity_admission=blocked reason=web_ui_variant_mismatch");
    }
    Ok(())
}
