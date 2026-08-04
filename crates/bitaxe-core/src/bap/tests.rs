use super::*;

const GOLDEN_CASES: &str = include_str!("../../fixtures/bap/protocol-cases.txt");

fn frame(command: BapCommand, parameter: BapParameter, maybe_value: Option<&str>) -> BapFrame {
    BapFrame::new(command, parameter, maybe_value.map(str::to_owned))
        .expect("test fixture fields are valid")
}

fn snapshot() -> BapRequestSnapshot {
    BapRequestSnapshot {
        device_model: "SyntheticDevice".to_owned(),
        asic_model: "SyntheticAsic".to_owned(),
        pool_endpoint: "synthetic.invalid".to_owned(),
        pool_port: 3_333,
        pool_user: "synthetic-worker".to_owned(),
        shares_accepted: 12,
        shares_rejected: 3,
        block_height: 900_001,
        found_block: 1,
        show_new_block: true,
    }
}

#[test]
fn vocabulary_round_trips_every_exact_command_and_parameter() {
    // Arrange / Act / Assert
    for command in BapCommand::ALL {
        assert_eq!(BapCommand::from_token(command.token()), Some(command));
        assert_eq!(
            BapCommand::from_token(&command.token().to_lowercase()),
            None
        );
    }
    for parameter in BapParameter::ALL {
        assert_eq!(BapParameter::from_token(parameter.token()), Some(parameter));
        assert_eq!(
            BapParameter::from_token(&parameter.token().to_uppercase()),
            None
        );
    }
}

#[test]
fn error_codes_render_the_exact_closed_public_vocabulary() {
    // Arrange
    let cases = [
        (
            BapErrorCode::ApModeNoSubscriptions,
            "ap_mode_no_subscriptions",
        ),
        (BapErrorCode::ApModeNoRequests, "ap_mode_no_requests"),
        (
            BapErrorCode::ApModeLimitedSettings,
            "ap_mode_limited_settings",
        ),
        (BapErrorCode::MissingParameter, "missing_parameter"),
        (BapErrorCode::SystemNotReady, "system_not_ready"),
        (BapErrorCode::InvalidRange, "invalid_range"),
        (BapErrorCode::SetFailed, "set_failed"),
        (BapErrorCode::UnsupportedParameter, "unsupported_parameter"),
        (BapErrorCode::SubscriptionTimeout, "subscription_timeout"),
    ];

    for (error, expected) in cases {
        // Act / Assert
        assert_eq!(error.token(), expected);
    }
}

#[test]
fn synthetic_golden_frames_encode_and_parse_exactly() {
    for line in GOLDEN_CASES.lines().filter(|line| !line.starts_with('#')) {
        // Arrange
        let fields: Vec<_> = line.split('|').collect();
        assert_eq!(fields.len(), 4);
        let command = BapCommand::from_token(fields[0]).expect("golden command is known");
        let parameter = BapParameter::from_token(fields[1]).expect("golden parameter is known");
        let maybe_value = (fields[2] != "-").then(|| fields[2].to_owned());
        let expected_wire = fields[3].replace("\\r\\n", "\r\n");
        let frame = BapFrame::new(command, parameter, maybe_value).expect("golden frame is valid");

        // Act
        let encoded = frame.encode().expect("golden frame fits the bound");
        let parsed = BapFrame::parse(expected_wire.as_bytes()).expect("golden wire parses");

        // Assert
        assert_eq!(encoded, expected_wire);
        assert_eq!(parsed.command(), command);
        assert_eq!(parsed.known_parameter(), Some(parameter));
        assert_eq!(parsed.value(), frame.value());
        assert_eq!(
            parsed.checksum_disposition(),
            BapChecksumDisposition::Verified
        );
    }
}

#[test]
fn parser_preserves_subscription_checksum_compatibility_only() {
    // Arrange / Act
    let subscribe_missing = BapFrame::parse(b"$BAP,SUB,hashrate,1000\r\n");
    let unsubscribe_missing = BapFrame::parse(b"$BAP,UNSUB,hashrate\r\n");
    let subscribe_mismatch = BapFrame::parse(b"$BAP,SUB,hashrate,1000*00\r\n");
    let unsubscribe_mismatch = BapFrame::parse(b"$BAP,UNSUB,hashrate*00\r\n");
    let request_missing = BapFrame::parse(b"$BAP,REQ,systemInfo\r\n");

    // Assert
    assert_eq!(
        subscribe_missing
            .expect("checksum-free SUB remains compatible")
            .checksum_disposition(),
        BapChecksumDisposition::SubscriptionMissingCompatibility
    );
    assert_eq!(
        unsubscribe_missing
            .expect("checksum-free UNSUB remains compatible")
            .checksum_disposition(),
        BapChecksumDisposition::SubscriptionMissingCompatibility
    );
    assert_eq!(
        subscribe_mismatch
            .expect("mismatched SUB remains compatible")
            .checksum_disposition(),
        BapChecksumDisposition::SubscriptionMismatchCompatibility
    );
    assert_eq!(unsubscribe_mismatch, Err(BapFrameError::ChecksumMismatch));
    assert_eq!(request_missing, Err(BapFrameError::MissingChecksum));
}

#[test]
fn parser_and_encoder_enforce_complete_message_bound_and_structure() {
    // Arrange
    let base = frame(BapCommand::Set, BapParameter::Ssid, Some("x"));
    let base_len = base.encode().expect("base frame fits").len();
    let maximum_value_len = 255 - (base_len - 1);
    let maximum = frame(
        BapCommand::Set,
        BapParameter::Ssid,
        Some(&"x".repeat(maximum_value_len)),
    );
    let oversized = frame(
        BapCommand::Set,
        BapParameter::Ssid,
        Some(&"x".repeat(maximum_value_len + 1)),
    );

    // Act / Assert
    assert_eq!(maximum.encode().expect("255 bytes fit").len(), 255);
    assert_eq!(oversized.encode(), Err(BapFrameError::TooLong));
    assert_eq!(
        BapFrame::parse(&vec![b'x'; BAP_MAX_MESSAGE_LEN]),
        Err(BapFrameError::TooLong)
    );
    assert_eq!(
        BapFrame::parse(b"BAP,REQ,systemInfo*3E\r\n"),
        Err(BapFrameError::MissingStart)
    );
    let unknown = BapFrame::parse(b"$BAP,REQ,unknown*7D\r\n")
        .expect("unknown bounded parameter remains handler-visible");
    assert_eq!(unknown.known_parameter(), None);
    assert_eq!(unknown.parameter_token(), "unknown");
}

#[test]
fn parser_returns_closed_categories_for_malformed_frames() {
    // Arrange
    let cases: &[(&[u8], BapFrameError)] = &[
        (b"", BapFrameError::Empty),
        (b"$XYZ,REQ,systemInfo*00\r\n", BapFrameError::InvalidTalker),
        (b"$BAP,BAD,systemInfo*00\r\n", BapFrameError::UnknownCommand),
        (b"$BAP,REQ,*00\r\n", BapFrameError::InvalidField),
        (
            b"$BAP,REQ,systemInfo,extra,field*00\r\n",
            BapFrameError::TooManyFields,
        ),
        (
            b"$BAP,REQ,systemInfo*GG\r\n",
            BapFrameError::MalformedChecksum,
        ),
        (&[0xff], BapFrameError::InvalidUtf8),
    ];

    for (wire, expected) in cases {
        // Act
        let result = BapFrame::parse(wire);

        // Assert
        assert_eq!(result, Err(*expected));
    }
}

#[test]
fn ingress_suppresses_duplicates_strictly_inside_one_second_and_redacts_bytes() {
    // Arrange
    let wire = frame(
        BapCommand::Set,
        BapParameter::Password,
        Some("synthetic-private-value"),
    )
    .encode()
    .expect("fixture fits");
    let mut ingress = BapIngress::default();

    // Act / Assert
    assert!(matches!(
        ingress.admit(wire.as_bytes(), 10_000),
        Ok(BapAdmission::Accepted(_))
    ));
    assert_eq!(
        ingress.admit(wire.as_bytes(), 10_999),
        Ok(BapAdmission::Duplicate)
    );
    assert!(matches!(
        ingress.admit(wire.as_bytes(), 11_000),
        Ok(BapAdmission::Accepted(_))
    ));
    let debug = format!("{ingress:?}");
    assert!(!debug.contains("synthetic-private-value"));
    assert!(!format!("{:?}", BapFrame::parse(wire.as_bytes())).contains("synthetic-private-value"));
}

#[test]
fn access_point_mode_returns_exact_request_subscription_and_setting_errors() {
    // Arrange
    let cases = [
        (
            frame(BapCommand::Request, BapParameter::Shares, None),
            BapErrorCode::ApModeNoRequests,
        ),
        (
            frame(BapCommand::Subscribe, BapParameter::Hashrate, None),
            BapErrorCode::ApModeNoSubscriptions,
        ),
        (
            frame(BapCommand::Set, BapParameter::Frequency, Some("500")),
            BapErrorCode::ApModeLimitedSettings,
        ),
    ];

    for (request, expected_error) in cases {
        // Act
        let plan = plan_command(&request, BapConnectionMode::AccessPoint, Some(&snapshot()))
            .expect("AP rejection is a public plan");

        // Assert
        assert_eq!(plan.responses().len(), 1);
        assert_eq!(plan.responses()[0].command(), BapCommand::Error);
        assert_eq!(plan.responses()[0].value(), Some(expected_error.token()));
        assert_eq!(plan.effect(), None);
    }
}

#[test]
fn supported_requests_project_exact_response_counts_and_values() {
    // Arrange
    let snapshot = snapshot();
    let system_info = frame(BapCommand::Request, BapParameter::SystemInfo, None);
    let shares = frame(BapCommand::Request, BapParameter::Shares, None);

    // Act
    let system_plan = plan_command(&system_info, BapConnectionMode::Connected, Some(&snapshot))
        .expect("snapshot projects");
    let shares_plan = plan_command(&shares, BapConnectionMode::Connected, Some(&snapshot))
        .expect("shares project");

    // Assert
    assert_eq!(system_plan.responses().len(), 5);
    assert_eq!(
        system_plan
            .responses()
            .iter()
            .map(|frame| (frame.parameter_token(), frame.value()))
            .collect::<Vec<_>>(),
        vec![
            ("deviceModel", Some("SyntheticDevice")),
            ("asicModel", Some("SyntheticAsic")),
            ("pool", Some("synthetic.invalid")),
            ("poolPort", Some("3333")),
            ("poolUser", Some("synthetic-worker")),
        ]
    );
    assert_eq!(shares_plan.responses()[0].value(), Some("12/3"));
    let debug = format!("{snapshot:?} {system_plan:?}");
    for sensitive in ["synthetic.invalid", "synthetic-worker"] {
        assert!(!debug.contains(sensitive));
    }
}

#[test]
fn subscriptions_use_positive_interval_or_reference_default() {
    // Arrange
    let explicit = frame(BapCommand::Subscribe, BapParameter::Hashrate, Some("750"));
    let invalid = frame(
        BapCommand::Subscribe,
        BapParameter::Hashrate,
        Some("invalid"),
    );
    let unsubscribe = frame(BapCommand::Unsubscribe, BapParameter::Hashrate, None);

    // Act
    let explicit_plan = plan_command(&explicit, BapConnectionMode::Connected, None)
        .expect("explicit subscription plans");
    let invalid_plan = plan_command(&invalid, BapConnectionMode::Connected, None)
        .expect("invalid interval falls back");
    let unsubscribe_plan = plan_command(&unsubscribe, BapConnectionMode::AccessPoint, None)
        .expect("unsubscribe remains available");

    // Assert
    assert_eq!(
        explicit_plan.effect(),
        Some(&BapEffect::Subscribe {
            parameter: BapParameter::Hashrate,
            interval_ms: 750,
            timeout_ms: BAP_SUBSCRIPTION_TIMEOUT_MS,
        })
    );
    assert_eq!(
        invalid_plan.effect(),
        Some(&BapEffect::Subscribe {
            parameter: BapParameter::Hashrate,
            interval_ms: BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS,
            timeout_ms: BAP_SUBSCRIPTION_TIMEOUT_MS,
        })
    );
    assert_eq!(
        unsubscribe_plan.effect(),
        Some(&BapEffect::Unsubscribe {
            parameter: BapParameter::Hashrate,
        })
    );
}

#[test]
fn settings_are_validated_and_remain_effect_intents_only() {
    // Arrange
    let frequency = frame(BapCommand::Set, BapParameter::Frequency, Some("500"));
    let invalid_voltage = frame(BapCommand::Set, BapParameter::AsicVoltage, Some("1401"));
    let password = frame(
        BapCommand::Set,
        BapParameter::Password,
        Some("synthetic-private-value"),
    );
    let fan = frame(BapCommand::Set, BapParameter::FanSpeed, Some("80"));
    let unsupported = frame(BapCommand::Set, BapParameter::Hashrate, Some("1"));

    // Act
    let frequency_plan =
        plan_command(&frequency, BapConnectionMode::Connected, None).expect("frequency plans");
    let invalid_plan = plan_command(&invalid_voltage, BapConnectionMode::Connected, None)
        .expect("invalid range produces public error");
    let password_plan = plan_command(&password, BapConnectionMode::AccessPoint, None)
        .expect("credential intent is allowed in AP mode");
    let fan_plan =
        plan_command(&fan, BapConnectionMode::Connected, None).expect("manual fan intent plans");
    let unsupported_plan = plan_command(&unsupported, BapConnectionMode::Connected, None)
        .expect("unsupported setting produces public error");

    // Assert
    assert_eq!(frequency_plan.responses()[0].value(), Some("500.00"));
    assert_eq!(invalid_plan.responses()[0].value(), Some("invalid_range"));
    assert_eq!(invalid_plan.effect(), None);
    assert!(matches!(
        password_plan.effect(),
        Some(BapEffect::ApplySetting {
            setting: BapSettingIntent::WifiPassword(_),
            restart: BapRestartPolicy::Always,
        })
    ));
    assert!(fan_plan.responses().is_empty());
    assert_eq!(
        fan_plan.effect(),
        Some(&BapEffect::ApplySetting {
            setting: BapSettingIntent::ManualFanPercent(80),
            restart: BapRestartPolicy::Never,
        })
    );
    assert_eq!(
        unsupported_plan.responses()[0].value(),
        Some("unsupported_parameter")
    );
    assert_eq!(unsupported_plan.effect(), None);
    assert!(!format!("{password_plan:?}").contains("synthetic-private-value"));
}

#[test]
fn device_to_host_commands_never_enter_request_handlers() {
    // Arrange
    let response = frame(BapCommand::Response, BapParameter::Hashrate, Some("20.5"));

    // Act
    let result = plan_command(&response, BapConnectionMode::Connected, Some(&snapshot()));

    // Assert
    assert_eq!(result, Err(BapPlanError::NoRegisteredHandler));
}
