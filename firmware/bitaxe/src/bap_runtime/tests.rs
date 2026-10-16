use super::*;
use bitaxe_core::bap::{BapRestartPolicy, BapSettingIntent, BAP_SUBSCRIPTION_TIMEOUT_MS};

fn snapshot() -> BapRequestSnapshot {
    BapRequestSnapshot {
        device_model: "Ultra".to_owned(),
        asic_model: "BM1366".to_owned(),
        pool_endpoint: "private.invalid".to_owned(),
        pool_port: 3333,
        pool_user: "private-worker".to_owned(),
        shares_accepted: 3,
        shares_rejected: 1,
        block_height: 10,
        found_block: 0,
        show_new_block: false,
    }
}

#[test]
fn only_a_request_reads_the_published_snapshot() {
    // Arrange
    let mut runtime = BapRuntime::default();
    runtime.set_mode(BapConnectionMode::Connected, 100);
    let mut reads = 0;

    // Act
    runtime
        .admit(b"$BAP,SUB,hashrate,1000*00\r\n", 100, || {
            reads += 1;
            Some(snapshot())
        })
        .expect("subscription plans");
    runtime
        .admit(b"$BAP,UNSUB,hashrate\r\n", 2_000, || {
            reads += 1;
            Some(snapshot())
        })
        .expect("unsubscribe plans");
    let answered = runtime
        .admit(b"$BAP,REQ,shares*0B\r\n", 4_000, || {
            reads += 1;
            Some(snapshot())
        })
        .expect("request plans");

    // Assert
    assert_eq!(reads, 1);
    assert_eq!(answered.responses.len(), 1);
}

#[test]
fn accumulator_frames_fragmented_crlf_and_discards_oversized_input() {
    // Arrange
    let mut accumulator = BapFrameAccumulator::default();
    let oversized = format!("${}\n", "x".repeat(BAP_MAX_MESSAGE_LEN));

    // Act
    let first = accumulator.push(b"noise$BAP,REQ,sha");
    let second = accumulator.push(b"res*54\r\n");
    let discarded = accumulator.push(oversized.as_bytes());
    let recovered = accumulator.push(b"$BAP,REQ,shares*54\n");

    // Assert
    assert!(first.is_empty());
    assert_eq!(second, vec![b"$BAP,REQ,shares*54\r".to_vec()]);
    assert!(discarded.is_empty());
    assert_eq!(recovered, vec![b"$BAP,REQ,shares*54\n".to_vec()]);
}

fn publishes(actions: &[BapRuntimeAction]) -> usize {
    actions
        .iter()
        .filter(|action| matches!(action, BapRuntimeAction::PublishSubscription(_)))
        .count()
}

fn subscribed_runtime(frame: &[u8], now_ms: u64) -> (BapRuntime, BapDispatch) {
    let mut runtime = BapRuntime::default();
    runtime.set_mode(BapConnectionMode::Connected, 0);
    assert!(runtime.poll(0).is_empty());
    let dispatch = runtime
        .admit(frame, now_ms, || Some(snapshot()))
        .expect("subscription plans");
    (runtime, dispatch)
}

#[test]
fn the_first_update_waits_only_for_the_next_one_second_tick() {
    // Arrange: reference `last_response = 0`, so the default 3,000 ms interval
    // does not delay the first update.
    let (mut runtime, subscribed) = subscribed_runtime(b"$BAP,SUB,hashrate*00\r\n", 200);

    // Act
    let before_tick = runtime.poll(999);
    let first_tick = runtime.poll(1_000);
    let inside_interval = runtime.poll(2_000);
    let after_interval = runtime.poll(4_000);

    // Assert
    assert_eq!(subscribed.maybe_subscribed, Some(BapParameter::Hashrate));
    assert_eq!(
        [&before_tick, &first_tick, &inside_interval, &after_interval].map(|a| publishes(a)),
        [0, 1, 0, 1]
    );
}

#[test]
fn a_sub_second_interval_still_publishes_at_most_once_per_tick() {
    // Arrange
    let (mut runtime, _) = subscribed_runtime(b"$BAP,SUB,hashrate,100*00\r\n", 0);

    // Act: the owner polls every 100 ms for 3 s.
    let total: usize = (1..=30)
        .map(|step| publishes(&runtime.poll(step * 100)))
        .sum();

    // Assert: one update on each of the ticks at 1 s, 2 s and 3 s.
    assert_eq!(total, 3);
}

#[test]
fn a_subscription_times_out_strictly_after_its_window_since_the_last_sub() {
    // Arrange
    let (mut runtime, _) = subscribed_runtime(b"$BAP,SUB,hashrate,1000*00\r\n", 0);
    let window = u64::from(BAP_SUBSCRIPTION_TIMEOUT_MS);

    // Act: the tick exactly at the window still publishes; the next one times out.
    let at_window = runtime.poll(window);
    let after_window = runtime.poll(window + 1_000);

    // Assert
    assert_eq!(publishes(&at_window), 1);
    assert_eq!(
        after_window,
        vec![BapRuntimeAction::SubscriptionTimedOut(
            BapParameter::Hashrate
        )]
    );
    assert!(runtime.poll(window + 2_000).is_empty());
}

fn temperature(chip: &str, vr: &str) -> Vec<BapFrame> {
    [("chipTemp", chip), ("vrTemp", vr)]
        .into_iter()
        .map(|(token, value)| {
            BapFrame::new_token(BapCommand::Response, token, Some(value.to_owned()))
                .expect("update frame")
        })
        .collect()
}

fn tokens(frames: &[BapFrame]) -> Vec<&str> {
    frames.iter().map(BapFrame::parameter_token).collect()
}

#[test]
fn only_the_changed_token_of_a_multi_token_update_is_resent() {
    // Arrange
    let mut last = BapLastValues::default();
    let first = last.changed(
        BapParameter::Temperature,
        temperature("53.000000", "0.000000"),
    );

    // Act: `chipTemp` changes while `vrTemp` stays 0, as on the Ultra 205.
    let second = last.changed(
        BapParameter::Temperature,
        temperature("54.000000", "0.000000"),
    );
    let unchanged = last.changed(
        BapParameter::Temperature,
        temperature("54.000000", "0.000000"),
    );

    // Assert
    assert_eq!(tokens(&first), vec!["chipTemp", "vrTemp"]);
    assert_eq!(tokens(&second), vec!["chipTemp"]);
    assert!(unchanged.is_empty());
}

#[test]
fn a_reset_resends_every_token_even_when_unchanged() {
    // Arrange
    let mut last = BapLastValues::default();
    last.changed(
        BapParameter::Temperature,
        temperature("53.000000", "0.000000"),
    );

    // Act
    last.reset(BapParameter::Temperature);
    let resent = last.changed(
        BapParameter::Temperature,
        temperature("53.000000", "0.000000"),
    );

    // Assert
    assert_eq!(tokens(&resent), vec!["chipTemp", "vrTemp"]);
}

#[test]
fn subscriptions_keep_publishing_after_a_wifi_loss() {
    // Arrange
    let (mut runtime, _) = subscribed_runtime(b"$BAP,SUB,hashrate,1000*00\r\n", 0);
    assert_eq!(publishes(&runtime.poll(1_000)), 1);

    // Act
    runtime.set_mode(BapConnectionMode::AccessPoint, 1_500);
    let lost = runtime.poll(2_000);

    // Assert: the reference announces access-point mode and keeps the subscription.
    assert_eq!(
        lost,
        vec![
            BapRuntimeAction::AnnounceAccessPoint,
            BapRuntimeAction::PublishSubscription(BapParameter::Hashrate),
        ]
    );
}

#[test]
fn unsubscribe_ends_updates() {
    // Arrange
    let (mut runtime, _) = subscribed_runtime(b"$BAP,SUB,hashrate,1000*00\r\n", 0);
    assert_eq!(publishes(&runtime.poll(1_000)), 1);

    // Act
    let unsubscribed = runtime
        .admit(b"$BAP,UNSUB,hashrate\r\n", 1_500, || Some(snapshot()))
        .expect("unsubscribe plans");

    // Assert
    assert_eq!(unsubscribed.responses.len(), 1);
    assert_eq!(unsubscribed.maybe_subscribed, None);
    assert!(runtime.poll(2_000).is_empty());
}

#[test]
fn runtime_defers_setting_ack_until_the_adapter_applies_the_effect() {
    // Arrange
    let mut runtime = BapRuntime::default();
    runtime.set_mode(BapConnectionMode::Connected, 0);

    // Arrange
    let frame = BapFrame::new(
        BapCommand::Set,
        BapParameter::Password,
        Some("secret-value".to_owned()),
    )
    .expect("setting frame constructs")
    .encode()
    .expect("setting frame encodes");

    // Act
    let dispatch = runtime
        .admit(frame.as_bytes(), 0, || Some(snapshot()))
        .expect("setting plans");

    // Assert
    assert!(dispatch.responses.is_empty());
    assert!(matches!(
        dispatch.maybe_setting,
        Some(BapPendingSetting {
            setting: BapSettingIntent::WifiPassword(_),
            restart: BapRestartPolicy::Always,
            ..
        })
    ));
    assert!(!format!("{dispatch:?}").contains("secret-value"));
}

#[test]
fn access_point_announcement_is_immediate_bounded_and_coalesced() {
    // Arrange
    let mut runtime = BapRuntime::default();

    // Act
    let first = runtime.poll(0);
    let before = runtime.poll(BAP_AP_ANNOUNCEMENT_INTERVAL_MS - 1);
    let delayed = runtime.poll(BAP_AP_ANNOUNCEMENT_INTERVAL_MS * 4);
    let after = runtime.poll(BAP_AP_ANNOUNCEMENT_INTERVAL_MS * 4 + 1);

    // Assert
    assert_eq!(first, vec![BapRuntimeAction::AnnounceAccessPoint]);
    assert!(before.is_empty());
    assert_eq!(delayed, vec![BapRuntimeAction::AnnounceAccessPoint]);
    assert!(after.is_empty());
}

#[test]
fn updates_due_on_one_tick_follow_the_reference_parameter_order() {
    // Arrange: subscribed out of order; the reference walks `BAP_PARAM` order.
    let mut runtime = BapRuntime::default();
    runtime.set_mode(BapConnectionMode::Connected, 0);
    for frame in [
        &b"$BAP,SUB,found_block,1000\r\n"[..],
        b"$BAP,SUB,hashrate,1000\r\n",
        b"$BAP,SUB,fan_speed,1000\r\n",
        b"$BAP,SUB,temperature,1000\r\n",
    ] {
        runtime
            .admit(frame, 100, || Some(snapshot()))
            .expect("subscription plans");
    }

    // Act
    let actions = runtime.poll(1_000);

    // Assert
    assert_eq!(
        actions,
        vec![
            BapRuntimeAction::PublishSubscription(BapParameter::Hashrate),
            BapRuntimeAction::PublishSubscription(BapParameter::Temperature),
            BapRuntimeAction::PublishSubscription(BapParameter::FanSpeed),
            BapRuntimeAction::PublishSubscription(BapParameter::FoundBlock),
        ]
    );
}

#[test]
fn a_setting_holds_its_acknowledgement_until_the_effect_succeeds() {
    // Arrange
    let mut runtime = BapRuntime::default();
    runtime.set_mode(BapConnectionMode::Connected, 0);

    let frame = BapFrame::new(
        BapCommand::Set,
        BapParameter::Frequency,
        Some("500".to_owned()),
    )
    .expect("set frame")
    .encode()
    .expect("set frame encodes");

    // Act
    let dispatch = runtime
        .admit(frame.as_bytes(), 100, || Some(snapshot()))
        .expect("setting plans");

    // Assert: nothing is sent yet; the acknowledgement waits in the pending setting.
    let pending = dispatch.maybe_setting.expect("pending setting");
    assert!(dispatch.responses.is_empty());
    assert_eq!(pending.setting, BapSettingIntent::FrequencyMhz(500.0));
    assert_eq!(pending.acknowledgement[0].value(), Some("500.00"));
}
