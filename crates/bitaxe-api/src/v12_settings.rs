//! Effect-free v1.2 settings authority.
//!
//! Compatibility parsing remains broad. This module grants authority only to
//! an exact, validated hostname field set and never plans storage or runtime
//! effects.

use std::collections::BTreeSet;
use std::fmt;

use bitaxe_config::{all_settings_schema, Hostname as ConfigHostname};
use serde_json::Value;

use crate::settings::{
    parse_settings_patch_body, plan_settings_patch_value, wrong_input, SettingsPatchFailure,
    SettingsPatchFieldError,
};

const CREDENTIAL_FIELDS: &[&str] = &[
    "ssid",
    "wifiPass",
    "stratumURL",
    "stratumPort",
    "stratumUser",
    "stratumPassword",
    "stratumCert",
    "stratumV2AuthorityPubkey",
    "fallbackStratumURL",
    "fallbackStratumPort",
    "fallbackStratumUser",
    "fallbackStratumPassword",
    "fallbackStratumCert",
    "fallbackStratumV2AuthorityPubkey",
];

const HARDWARE_CONTROL_FIELDS: &[&str] = &[
    "frequency",
    "coreVoltage",
    "overclockEnabled",
    "autofanspeed",
    "manualFanSpeed",
    "minFanSpeed",
    "temptarget",
    "overheat_mode",
];

const MINING_OR_SELF_TEST_FIELDS: &[&str] = &[
    "stratumProtocol",
    "stratumSuggestedDifficulty",
    "stratumExtranonceSubscribe",
    "stratumTLS",
    "stratumV2ChannelType",
    "stratumDecodeCoinbase",
    "fallbackStratumProtocol",
    "fallbackStratumSuggestedDifficulty",
    "fallbackStratumExtranonceSubscribe",
    "fallbackStratumTLS",
    "fallbackStratumV2ChannelType",
    "fallbackStratumDecodeCoinbase",
    "useFallbackStratum",
    "selftest",
    "selfTest",
];

/// Validated v1.2 hostname value.
#[derive(Clone, PartialEq, Eq)]
pub struct Hostname(ConfigHostname);

impl Hostname {
    /// Returns the validated hostname for a later authorized adapter.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for Hostname {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Hostname([redacted])")
    }
}

/// Complete constructible v1.2 settings write authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V12SettingsChange {
    /// A validated hostname replacement.
    Hostname(Hostname),
}

/// Category-only reason that a compatibility input has no v1.2 authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V12SettingsExclusionReason {
    /// A known compatibility field outside the v1.2 allowlist.
    BroaderKnownField,
    /// An unknown compatibility field.
    UnknownField,
    /// More than one field was supplied.
    MixedFieldSet,
    /// A credential or secret-bearing field was supplied.
    CredentialField,
    /// A hardware-control field was supplied.
    HardwareControlField,
    /// A mining or self-test field was supplied.
    MiningOrSelfTestField,
    /// No fields were supplied.
    EmptyPatch,
}

/// Effect-free v1.2 authority decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V12SettingsDecision {
    /// The input is the one closed v1.2 write capability.
    Authorized(V12SettingsChange),
    /// The input remains compatibility-only and cannot create effects.
    CompatibilityOnly(V12SettingsExclusionReason),
}

/// Parses a raw body and classifies its v1.2 authority without side effects.
pub fn decide_v12_settings_body(body: &str) -> Result<V12SettingsDecision, SettingsPatchFailure> {
    let value = parse_settings_patch_body(body)?;
    decide_v12_settings_value(&value)
}

/// Classifies a parsed settings value without producing persistence authority.
pub fn decide_v12_settings_value(
    value: &Value,
) -> Result<V12SettingsDecision, SettingsPatchFailure> {
    let Some(object) = value.as_object() else {
        return plan_settings_patch_value(value).map(|_| {
            V12SettingsDecision::CompatibilityOnly(V12SettingsExclusionReason::UnknownField)
        });
    };

    if object.is_empty() {
        return Ok(V12SettingsDecision::CompatibilityOnly(
            V12SettingsExclusionReason::EmptyPatch,
        ));
    }

    if object.len() > 1 {
        return Ok(V12SettingsDecision::CompatibilityOnly(
            V12SettingsExclusionReason::MixedFieldSet,
        ));
    }

    let Some((field, raw_value)) = object.iter().next() else {
        return Ok(V12SettingsDecision::CompatibilityOnly(
            V12SettingsExclusionReason::EmptyPatch,
        ));
    };

    if field != "hostname" {
        return Ok(V12SettingsDecision::CompatibilityOnly(exclusion_for_field(
            field,
        )));
    }

    plan_settings_patch_value(value)?;
    let Some(raw_hostname) = raw_value.as_str() else {
        return plan_settings_patch_value(value).map(|_| {
            V12SettingsDecision::CompatibilityOnly(V12SettingsExclusionReason::BroaderKnownField)
        });
    };
    let hostname = ConfigHostname::parse(raw_hostname.to_owned())
        .map_err(|error| wrong_input(vec![SettingsPatchFieldError::Validation(error)]))?;

    Ok(V12SettingsDecision::Authorized(
        V12SettingsChange::Hostname(Hostname(hostname)),
    ))
}

fn exclusion_for_field(field: &str) -> V12SettingsExclusionReason {
    if CREDENTIAL_FIELDS.contains(&field) {
        return V12SettingsExclusionReason::CredentialField;
    }

    if HARDWARE_CONTROL_FIELDS.contains(&field) {
        return V12SettingsExclusionReason::HardwareControlField;
    }

    if MINING_OR_SELF_TEST_FIELDS.contains(&field) {
        return V12SettingsExclusionReason::MiningOrSelfTestField;
    }

    if known_rest_field_names().contains(field) {
        return V12SettingsExclusionReason::BroaderKnownField;
    }

    V12SettingsExclusionReason::UnknownField
}

fn known_rest_field_names() -> BTreeSet<String> {
    all_settings_schema()
        .into_iter()
        .filter_map(|setting| {
            setting
                .rest_name
                .map(|rest_name| rest_name.as_str().to_owned())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use bitaxe_config::all_settings_schema;
    use serde_json::{json, Map, Value};

    use super::{
        decide_v12_settings_body, decide_v12_settings_value, V12SettingsChange,
        V12SettingsDecision, V12SettingsExclusionReason,
    };
    use crate::SettingsPatchPublicError;

    #[test]
    fn settings_v12_exact_valid_hostname_is_the_only_authorized_change() {
        // Arrange
        let body = json!({"hostname": "axe-205"});

        // Act
        let decision = decide_v12_settings_value(&body).expect("valid hostname must classify");

        // Assert
        let V12SettingsDecision::Authorized(V12SettingsChange::Hostname(hostname)) = decision
        else {
            panic!("exact hostname must be authorized");
        };
        assert_eq!(hostname.as_str(), "axe-205");
        assert_eq!(format!("{hostname:?}"), "Hostname([redacted])");
    }

    #[test]
    fn settings_v12_every_other_known_schema_field_is_compatibility_only() {
        // Arrange
        let field_names = all_settings_schema()
            .into_iter()
            .filter_map(|setting| setting.rest_name)
            .map(|rest_name| rest_name.as_str().to_owned())
            .filter(|field| field != "hostname")
            .collect::<Vec<_>>();

        for field in field_names {
            let mut object = Map::new();
            object.insert(field.clone(), Value::Null);

            // Act
            let decision = decide_v12_settings_value(&Value::Object(object))
                .expect("well-formed broader field must classify");

            // Assert
            assert!(matches!(
                decision,
                V12SettingsDecision::CompatibilityOnly(_)
            ));
        }
    }

    #[test]
    fn settings_v12_exclusion_categories_are_deterministic() {
        // Arrange
        let cases = [
            (json!({}), V12SettingsExclusionReason::EmptyPatch),
            (
                json!({"futureField": true}),
                V12SettingsExclusionReason::UnknownField,
            ),
            (
                json!({"display": "compact"}),
                V12SettingsExclusionReason::BroaderKnownField,
            ),
            (
                json!({"wifiPass": "secret"}),
                V12SettingsExclusionReason::CredentialField,
            ),
            (
                json!({"manualFanSpeed": 80}),
                V12SettingsExclusionReason::HardwareControlField,
            ),
            (
                json!({"selfTest": true}),
                V12SettingsExclusionReason::MiningOrSelfTestField,
            ),
            (
                json!({"hostname": "axe-205", "display": "compact"}),
                V12SettingsExclusionReason::MixedFieldSet,
            ),
        ];

        for (input, expected) in cases {
            // Act
            let decision = decide_v12_settings_value(&input).expect("input must classify");

            // Assert
            assert_eq!(decision, V12SettingsDecision::CompatibilityOnly(expected));
        }
    }

    #[test]
    fn settings_v12_mixed_secret_input_debug_never_contains_raw_values() {
        // Arrange
        let secret = "must-not-appear";
        let body = json!({"hostname": "axe-205", "wifiPass": secret});

        // Act
        let decision = decide_v12_settings_value(&body).expect("mixed input must classify");
        let diagnostics = format!("{decision:?}");

        // Assert
        assert_eq!(
            decision,
            V12SettingsDecision::CompatibilityOnly(V12SettingsExclusionReason::MixedFieldSet)
        );
        assert!(!diagnostics.contains(secret));
        assert!(!diagnostics.contains("axe-205"));
    }

    #[test]
    fn settings_v12_hostname_mixed_with_every_broader_category_is_ineligible() {
        // Arrange
        let broader_fields = [
            ("wifiPass", json!("secret")),
            ("manualFanSpeed", json!(80)),
            ("coreVoltage", json!(1200)),
            ("frequency", json!(485)),
            ("stratumProtocol", json!("SV1")),
            ("selfTest", json!(true)),
            ("display", json!("compact")),
        ];

        for (field, raw_value) in broader_fields {
            let mut object = Map::new();
            object.insert("hostname".to_owned(), json!("axe-205"));
            object.insert(field.to_owned(), raw_value);

            // Act
            let decision = decide_v12_settings_value(&Value::Object(object))
                .expect("mixed compatibility input must classify");

            // Assert
            assert_eq!(
                decision,
                V12SettingsDecision::CompatibilityOnly(V12SettingsExclusionReason::MixedFieldSet)
            );
        }
    }

    #[test]
    fn settings_v12_authority_module_has_no_persistence_or_effect_surface() {
        // Arrange
        let source = include_str!("v12_settings.rs");
        let domain_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("module source always precedes its tests");

        // Act
        let prohibited_symbols = ["SettingsPersistence", "NvsWrite", "execute_settings"];

        // Assert
        for symbol in prohibited_symbols {
            assert!(!domain_source.contains(symbol));
        }
    }

    #[test]
    fn settings_v12_invalid_hostname_keeps_generic_public_error() {
        // Arrange
        let invalid_hostname = json!({"hostname": ""});

        // Act
        let error = decide_v12_settings_value(&invalid_hostname)
            .expect_err("invalid hostname must not be authorized");

        // Assert
        assert_eq!(
            error.public_error(),
            SettingsPatchPublicError::WrongApiInput
        );
        assert_eq!(error.public_error().body(), "Wrong API input");
    }

    #[test]
    fn settings_v12_malformed_json_keeps_generic_public_error() {
        // Arrange
        let malformed = "{bad json";

        // Act
        let error = decide_v12_settings_body(malformed).expect_err("malformed JSON must reject");

        // Assert
        assert_eq!(error.public_error(), SettingsPatchPublicError::InvalidJson);
        assert_eq!(error.public_error().body(), "Invalid JSON");
    }
}
