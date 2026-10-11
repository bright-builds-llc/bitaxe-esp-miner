//! What the client will send. A setting needs its name repeated as confirmation;
//! credentials come only from a private file; ASIC frequency and core voltage need
//! an explicit allowance, because they drive the hashing chip's power.

use crate::redaction::is_sensitive;
use bitaxe_core::bap::BapParameter;
use std::path::PathBuf;

/// Where a setting's value comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueSource {
    Inline(String),
    PrivateFile(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    UnknownParameter,
    ConfirmationMismatch,
    CredentialOnCommandLine,
    AsicSettingNotAllowed,
    ValueSourceAmbiguous,
}

/// One known parameter by its exact wire token.
pub fn parameter(token: &str) -> Result<BapParameter, Refusal> {
    BapParameter::from_token(token).ok_or(Refusal::UnknownParameter)
}

/// Exactly one of an inline value or a value file.
pub fn value_source(
    maybe_value: Option<String>,
    maybe_file: Option<PathBuf>,
) -> Result<ValueSource, Refusal> {
    match (maybe_value, maybe_file) {
        (Some(value), None) => Ok(ValueSource::Inline(value)),
        (None, Some(path)) => Ok(ValueSource::PrivateFile(path)),
        _ => Err(Refusal::ValueSourceAmbiguous),
    }
}

/// Admits one setting change before any byte is sent.
pub fn admit_setting(
    parameter: BapParameter,
    source: &ValueSource,
    confirm: &str,
    allow_asic_setting: bool,
) -> Result<(), Refusal> {
    if confirm != parameter.token() {
        return Err(Refusal::ConfirmationMismatch);
    }
    if is_sensitive(parameter.token()) && matches!(source, ValueSource::Inline(_)) {
        return Err(Refusal::CredentialOnCommandLine);
    }
    if matches!(
        parameter,
        BapParameter::Frequency | BapParameter::AsicVoltage
    ) && !allow_asic_setting
    {
        return Err(Refusal::AsicSettingNotAllowed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_confirmed_fan_setting_is_admitted() {
        let source = ValueSource::Inline("80".into());
        assert_eq!(
            admit_setting(BapParameter::FanSpeed, &source, "fan_speed", false),
            Ok(())
        );
    }

    #[test]
    fn a_mismatched_confirmation_is_refused() {
        let source = ValueSource::Inline("80".into());
        assert_eq!(
            admit_setting(BapParameter::FanSpeed, &source, "auto_fan", false),
            Err(Refusal::ConfirmationMismatch)
        );
    }

    #[test]
    fn a_credential_on_the_command_line_is_refused() {
        let source = ValueSource::Inline("secret".into());
        assert_eq!(
            admit_setting(BapParameter::Password, &source, "password", false),
            Err(Refusal::CredentialOnCommandLine)
        );
    }

    #[test]
    fn a_credential_from_a_private_file_is_admitted() {
        let source = ValueSource::PrivateFile("value".into());
        assert_eq!(
            admit_setting(BapParameter::Ssid, &source, "ssid", false),
            Ok(())
        );
    }

    #[test]
    fn asic_settings_need_an_explicit_allowance() {
        let source = ValueSource::Inline("490".into());
        assert_eq!(
            admit_setting(BapParameter::Frequency, &source, "frequency", false),
            Err(Refusal::AsicSettingNotAllowed)
        );
        assert_eq!(
            admit_setting(BapParameter::Frequency, &source, "frequency", true),
            Ok(())
        );
    }

    #[test]
    fn exactly_one_value_source_is_required() {
        assert_eq!(value_source(None, None), Err(Refusal::ValueSourceAmbiguous));
        assert_eq!(
            value_source(Some("1".into()), Some("f".into())),
            Err(Refusal::ValueSourceAmbiguous)
        );
    }

    #[test]
    fn parameters_are_exact_wire_tokens() {
        assert_eq!(parameter("hashrate"), Ok(BapParameter::Hashrate));
        assert_eq!(parameter("Hashrate"), Err(Refusal::UnknownParameter));
    }
}
