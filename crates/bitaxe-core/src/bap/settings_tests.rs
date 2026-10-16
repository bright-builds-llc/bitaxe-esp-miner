//! Reference setting parsing (`atoi`/`atof`, `uint16_t` casts) and the replies
//! sent once a setting has been handled (`bap_handlers.c:BAP_handle_settings`).
use super::*;

fn plan_set(parameter: BapParameter, value: &str) -> BapPlan {
    let frame =
        BapFrame::new(BapCommand::Set, parameter, Some(value.to_owned())).expect("set frame");
    plan_command(&frame, BapConnectionMode::Connected, None).expect("setting plans")
}

fn pending(parameter: BapParameter, value: &str) -> BapPendingSetting {
    let plan = plan_set(parameter, value);
    let Some(BapEffect::ApplySetting { setting, restart }) = plan.effect() else {
        panic!("not a setting intent");
    };
    BapPendingSetting {
        setting: setting.clone(),
        restart: *restart,
        acknowledgement: plan.responses().to_vec(),
    }
}

fn applied(companion_credential_present: bool) -> BapSettingResult {
    BapSettingResult::Applied {
        companion_credential_present,
    }
}

fn sent(completion: &BapSettingCompletion) -> Vec<(BapCommand, Option<&str>)> {
    completion
        .acknowledgement
        .iter()
        .chain(&completion.maybe_restart_status)
        .map(|frame| (frame.command(), frame.value()))
        .collect()
}

#[test]
fn a_frequency_with_trailing_text_parses_like_atof() {
    // Arrange / Act
    let plan = plan_set(BapParameter::Frequency, "485.5MHz");
    // Assert
    assert_eq!(plan.responses()[0].value(), Some("485.50"));
}

#[test]
fn a_voltage_wraps_through_the_reference_uint16_cast() {
    // Arrange: (uint16_t)66236 is 700, inside the 700-1400 mV range.
    let plan = plan_set(BapParameter::AsicVoltage, "66236");
    // Act
    let effect = plan.effect();
    // Assert
    assert_eq!(
        effect,
        Some(&BapEffect::ApplySetting {
            setting: BapSettingIntent::AsicVoltageMillivolts(700),
            restart: BapRestartPolicy::Never,
        })
    );
}

#[test]
fn a_non_numeric_frequency_reads_as_zero_and_is_out_of_range() {
    // Arrange / Act
    let plan = plan_set(BapParameter::Frequency, "fast");
    // Assert
    assert_eq!(plan.responses()[0].value(), Some("invalid_range"));
}

#[test]
fn found_block_garbage_reads_as_zero_and_echoes_the_raw_value() {
    // Arrange / Act
    let plan = plan_set(BapParameter::FoundBlock, "abc");
    // Assert
    assert_eq!(
        (plan.effect(), plan.responses()[0].value()),
        (
            Some(&BapEffect::ApplySetting {
                setting: BapSettingIntent::FoundBlock(0),
                restart: BapRestartPolicy::Never,
            }),
            Some("abc")
        )
    );
}

#[test]
fn an_applied_frequency_sends_its_acknowledgement_without_restart() {
    // Arrange
    let pending = pending(BapParameter::Frequency, "500");
    // Act
    let completion = complete_setting(&pending, applied(false)).expect("completes");
    // Assert
    assert_eq!(
        sent(&completion),
        vec![(BapCommand::Acknowledge, Some("500.00"))]
    );
}

#[test]
fn an_applied_fan_speed_sends_nothing() {
    // Arrange: the reference stores the fan speed without acknowledging it.
    let pending = pending(BapParameter::FanSpeed, "80");
    // Act
    let completion = complete_setting(&pending, applied(false)).expect("completes");
    // Assert
    assert!(sent(&completion).is_empty());
}

#[test]
fn a_failed_setting_sends_set_failed() {
    // Arrange
    let pending = pending(BapParameter::AsicVoltage, "1200");
    // Act
    let completion = complete_setting(&pending, BapSettingResult::Failed).expect("completes");
    // Assert
    assert_eq!(
        sent(&completion),
        vec![(BapCommand::Error, Some("set_failed"))]
    );
}

#[test]
fn a_new_password_acknowledges_then_announces_the_restart() {
    // Arrange
    let pending = pending(BapParameter::Password, "synthetic-private-value");
    // Act
    let completion = complete_setting(&pending, applied(true)).expect("completes");
    // Assert
    assert_eq!(
        sent(&completion),
        vec![
            (BapCommand::Acknowledge, Some("password_set")),
            (BapCommand::Status, Some("restarting")),
        ]
    );
}

#[test]
fn an_unchanged_password_acknowledges_without_restart() {
    // Arrange
    let pending = pending(BapParameter::Password, "synthetic-private-value");
    // Act
    let completion = complete_setting(&pending, BapSettingResult::Unchanged).expect("completes");
    // Assert
    assert_eq!(
        sent(&completion),
        vec![(BapCommand::Acknowledge, Some("password_already_set"))]
    );
}

#[test]
fn an_unchanged_ssid_echoes_it_without_restart() {
    // Arrange
    let pending = pending(BapParameter::Ssid, "synthetic-network");
    // Act
    let completion = complete_setting(&pending, BapSettingResult::Unchanged).expect("completes");
    // Assert
    assert_eq!(
        sent(&completion),
        vec![(BapCommand::Acknowledge, Some("synthetic-network"))]
    );
}

#[test]
fn a_new_ssid_restarts_only_when_a_password_is_stored() {
    // Arrange
    let pending = pending(BapParameter::Ssid, "synthetic-network");
    // Act
    let with_password = complete_setting(&pending, applied(true)).expect("completes");
    let without_password = complete_setting(&pending, applied(false)).expect("completes");
    // Assert
    assert_eq!(
        (
            with_password.maybe_restart_status.is_some(),
            without_password.maybe_restart_status.is_some()
        ),
        (true, false)
    );
}
