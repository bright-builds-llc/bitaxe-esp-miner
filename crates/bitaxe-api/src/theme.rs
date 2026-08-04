//! Pure `/api/theme` projection and update planning.
//!
//! Reference: `reference/esp-miner/main/http_server/theme_api.c`.

use bitaxe_config::{
    NvsSnapshot, NvsWrite, StoredValueKind, DEFAULT_THEME_ACCENT_COLORS_JSON,
    DEFAULT_THEME_COLOR_SCHEME,
};
use serde::Serialize;
use serde_json::Value;

/// Upstream theme POST buffer capacity excluding its null terminator.
pub const MAX_THEME_POST_BODY_BYTES: usize = 1023;

/// Public GET `/api/theme` response.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ThemeSettings {
    #[serde(rename = "colorScheme")]
    color_scheme: String,
    #[serde(rename = "accentColors", skip_serializing_if = "Option::is_none")]
    maybe_accent_colors: Option<Value>,
}

impl ThemeSettings {
    /// Returns the projected color scheme.
    #[must_use]
    pub fn color_scheme(&self) -> &str {
        &self.color_scheme
    }

    /// Returns parsed accent colors when the stored JSON is valid.
    #[must_use]
    pub const fn maybe_accent_colors(&self) -> Option<&Value> {
        self.maybe_accent_colors.as_ref()
    }
}

/// Public success response emitted by POST `/api/theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ThemePostResponse {
    status: &'static str,
}

impl ThemePostResponse {
    /// Returns the exact upstream success object.
    #[must_use]
    pub const fn ok() -> Self {
        Self { status: "ok" }
    }
}

/// Closed failure categories for theme POST planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePostFailure {
    /// The request cannot fit the upstream 1024-byte stack buffer.
    BodyTooLarge,
    /// The request body is not valid JSON.
    InvalidJson,
}

impl ThemePostFailure {
    /// Returns the upstream-compatible public HTTP status.
    #[must_use]
    pub const fn status(self) -> u16 {
        400
    }

    /// Returns the upstream-compatible public error body.
    #[must_use]
    pub const fn body(self) -> &'static str {
        "Invalid JSON"
    }
}

/// Inert accepted theme update prepared before writable NVS is opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePostPlan {
    maybe_color_scheme: Option<String>,
    maybe_accent_colors_json: Option<String>,
}

impl ThemePostPlan {
    /// Returns whether at least one correctly typed field will be persisted.
    #[must_use]
    pub const fn has_writes(&self) -> bool {
        self.maybe_color_scheme.is_some() || self.maybe_accent_colors_json.is_some()
    }

    /// Returns exact inert NVS writes in upstream handler order.
    #[must_use]
    pub fn writes(&self) -> Vec<NvsWrite> {
        let mut writes = Vec::new();
        if let Some(color_scheme) = &self.maybe_color_scheme {
            writes.push(NvsWrite::string("themescheme", color_scheme.clone()));
        }
        if let Some(accent_colors_json) = &self.maybe_accent_colors_json {
            writes.push(NvsWrite::string("themecolors", accent_colors_json.clone()));
        }
        writes
    }

    /// Confirms every requested field against an independently reloaded snapshot.
    #[must_use]
    pub fn reconciles(&self, snapshot: &NvsSnapshot) -> bool {
        matches_requested_string(snapshot, "themescheme", self.maybe_color_scheme.as_deref())
            && matches_requested_string(
                snapshot,
                "themecolors",
                self.maybe_accent_colors_json.as_deref(),
            )
    }
}

/// Projects GET `/api/theme` from the last confirmed settings snapshot.
#[must_use]
pub fn theme_settings_from_snapshot(snapshot: &NvsSnapshot) -> ThemeSettings {
    let color_scheme = stored_string(snapshot, "themescheme")
        .unwrap_or(DEFAULT_THEME_COLOR_SCHEME)
        .to_owned();
    let colors_json =
        stored_string(snapshot, "themecolors").unwrap_or(DEFAULT_THEME_ACCENT_COLORS_JSON);
    let maybe_accent_colors = serde_json::from_str(colors_json).ok();

    ThemeSettings {
        color_scheme,
        maybe_accent_colors,
    }
}

/// Plans POST `/api/theme` without storage or firmware effects.
pub fn plan_theme_post(body: &str) -> Result<ThemePostPlan, ThemePostFailure> {
    if body.len() > MAX_THEME_POST_BODY_BYTES {
        return Err(ThemePostFailure::BodyTooLarge);
    }
    let root: Value = serde_json::from_str(body).map_err(|_| ThemePostFailure::InvalidJson)?;
    let maybe_object = root.as_object();
    let maybe_color_scheme = maybe_object
        .and_then(|object| object.get("colorScheme"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let maybe_accent_colors_json = maybe_object
        .and_then(|object| object.get("accentColors"))
        .map(|colors| serde_json::to_string(colors).expect("JSON values always serialize"));

    Ok(ThemePostPlan {
        maybe_color_scheme,
        maybe_accent_colors_json,
    })
}

fn stored_string<'snapshot>(snapshot: &'snapshot NvsSnapshot, key: &str) -> Option<&'snapshot str> {
    let stored = snapshot.maybe_stored_value(key)?;
    let StoredValueKind::String(value) = &stored.value else {
        return None;
    };
    Some(value)
}

fn matches_requested_string(
    snapshot: &NvsSnapshot,
    key: &str,
    maybe_expected: Option<&str>,
) -> bool {
    let Some(expected) = maybe_expected else {
        return true;
    };
    stored_string(snapshot, key) == Some(expected)
}

#[cfg(test)]
mod tests;
