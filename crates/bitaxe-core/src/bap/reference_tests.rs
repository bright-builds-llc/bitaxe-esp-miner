//! Reference-behaviour regressions found by the BAP ordinal 008 review.
use super::*;

fn request(parameter: BapParameter) -> BapFrame {
    BapFrame::new(BapCommand::Request, parameter, None).expect("request frame")
}

fn snapshot() -> BapRequestSnapshot {
    BapRequestSnapshot {
        device_model: "Ultra".to_owned(),
        asic_model: "BM1366".to_owned(),
        pool_endpoint: String::new(),
        pool_port: 3_333,
        pool_user: "synthetic-worker".to_owned(),
        shares_accepted: 0,
        shares_rejected: 0,
        block_height: 0,
        found_block: 2,
        show_new_block: false,
    }
}

fn subscribed_interval(value: &str) -> u32 {
    let frame = BapFrame::new(
        BapCommand::Subscribe,
        BapParameter::Hashrate,
        Some(value.to_owned()),
    )
    .expect("subscription frame");
    match plan_command(&frame, BapConnectionMode::Connected, None)
        .expect("subscription plans")
        .effect()
    {
        Some(BapEffect::Subscribe { interval_ms, .. }) => *interval_ms,
        other => panic!("not a subscription: {other:?}"),
    }
}

#[test]
fn a_found_block_request_answers_the_reference_block_found_token() {
    // Arrange
    let snapshot = snapshot();
    // Act
    let plan = plan_command(
        &request(BapParameter::FoundBlock),
        BapConnectionMode::Connected,
        Some(&snapshot),
    )
    .expect("request plans");
    // Assert: `bap_handlers.c` answers `block_found`; only subscriptions use `found_block`.
    let answers: Vec<_> = plan
        .responses()
        .iter()
        .map(|frame| (frame.parameter_token(), frame.value()))
        .collect();
    assert_eq!(answers, vec![("block_found", Some("2"))]);
}

#[test]
fn an_empty_value_is_sent_without_a_value_field() {
    // Arrange: an unset pool URL, which the reference still answers.
    let snapshot = snapshot();
    // Act
    let plan = plan_command(
        &request(BapParameter::SystemInfo),
        BapConnectionMode::Connected,
        Some(&snapshot),
    )
    .expect("an empty field still answers");
    // Assert: `BAP_send_message` omits the value field when `strlen(value) == 0`.
    let pool = &plan.responses()[2];
    assert_eq!((pool.parameter_token(), pool.value()), ("pool", None));
    let encoded = pool.encode().expect("encodes");
    assert!(encoded.starts_with("$BAP,RES,pool*"), "{encoded}");
}

#[test]
fn intervals_parse_like_c_atoi() {
    // Arrange / Act / Assert: digits up to the first non-digit, after whitespace and a sign.
    assert_eq!(subscribed_interval("1000abc"), 1_000);
    assert_eq!(subscribed_interval("+1500"), 1_500);
    assert_eq!(
        subscribed_interval("abc"),
        BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS
    );
    assert_eq!(
        subscribed_interval("-5"),
        BAP_DEFAULT_SUBSCRIPTION_INTERVAL_MS
    );
    assert_eq!(subscribed_interval("99999999999"), 2_147_483_647);
}
