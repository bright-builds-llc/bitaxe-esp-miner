use super::*;

fn connected_snapshot() -> ScreenSnapshot {
    ScreenSnapshot {
        wifi_connected: true,
        ssid: "fixture-network".to_owned(),
        wifi_status: "connected".to_owned(),
        ipv4: "192.0.2.1".to_owned(),
        model: "Ultra 205".to_owned(),
        board: "BM1366".to_owned(),
        version: "abcdef123456-dev".to_owned(),
        pool_host: "pool.example.test".to_owned(),
        maybe_hashrate_ghs: Some(500.0),
        maybe_power_watts: Some(12.0),
        maybe_best_difficulty: Some(42.0),
        maybe_temperature_celsius: Some(55.5),
        maybe_block_height: Some(900_000),
        maybe_network_difficulty: Some(100_000.0),
        maybe_scriptsig: Some("fixture".to_owned()),
        maybe_rssi_dbm: Some(-55),
        ..ScreenSnapshot::default()
    }
}

#[test]
fn priority_routes_follow_the_pinned_order_and_release_to_intro() {
    // Arrange
    let mut snapshot = connected_snapshot();
    snapshot.maybe_self_test = Some(["running".to_owned(), String::new(), String::new()]);
    snapshot.maybe_firmware_update = Some(["firmware.bin".to_owned(), "writing".to_owned()]);
    snapshot.maybe_asic_status = Some("initializing".to_owned());
    snapshot.overheat = true;
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act / Assert
    assert_eq!(
        flow.update(0, &snapshot).expect("self-test").page,
        ScreenPage::SelfTest
    );
    snapshot.maybe_self_test = None;
    assert_eq!(
        flow.update(500, &snapshot).expect("firmware").page,
        ScreenPage::FirmwareUpdate
    );
    snapshot.maybe_firmware_update = None;
    assert_eq!(
        flow.update(1_000, &snapshot).expect("ASIC").page,
        ScreenPage::AsicStatus
    );
    snapshot.maybe_asic_status = None;
    assert_eq!(
        flow.update(1_500, &snapshot).expect("overheat").page,
        ScreenPage::Overheat
    );
    snapshot.overheat = false;
    assert_eq!(
        flow.update(2_000, &snapshot).expect("intro").page,
        ScreenPage::BitaxeIntro
    );
}

#[test]
fn welcome_and_connection_precede_the_normal_flow() {
    // Arrange
    let mut snapshot = ScreenSnapshot {
        ap_enabled: true,
        ..ScreenSnapshot::default()
    };
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act / Assert
    assert_eq!(
        flow.update(0, &snapshot).expect("welcome").page,
        ScreenPage::Welcome
    );
    snapshot.ssid = "configured".to_owned();
    assert_eq!(
        flow.update(500, &snapshot).expect("connection").page,
        ScreenPage::Connection
    );
    snapshot.ap_enabled = false;
    snapshot.wifi_connected = true;
    assert_eq!(
        flow.update(1_000, &snapshot).expect("intro").page,
        ScreenPage::BitaxeIntro
    );
}

#[test]
fn intro_and_carousel_use_exact_pinned_delay_boundaries() {
    // Arrange
    let snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act / Assert
    assert_eq!(
        flow.update(3_000, &snapshot).expect("boundary").page,
        ScreenPage::BitaxeIntro
    );
    assert_eq!(
        flow.update(3_500, &snapshot).expect("next tick").page,
        ScreenPage::OpenSourceIntro
    );
    assert_eq!(
        flow.update(6_500, &snapshot).expect("boundary").page,
        ScreenPage::OpenSourceIntro
    );
    assert_eq!(
        flow.update(7_000, &snapshot).expect("urls").page,
        ScreenPage::Urls
    );
    assert_eq!(
        flow.update(17_000, &snapshot).expect("boundary").page,
        ScreenPage::Urls
    );
    assert_eq!(
        flow.update(17_500, &snapshot).expect("statistics").page,
        ScreenPage::Statistics
    );
}

#[test]
fn carousel_wraps_after_wifi() {
    // Arrange
    let snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);
    let times = [3_500, 7_000, 17_500, 28_000, 38_500, 49_000];

    // Act
    let pages = times.map(|time| flow.update(time, &snapshot).expect("advance").page);

    // Assert
    assert_eq!(
        pages,
        [
            ScreenPage::OpenSourceIntro,
            ScreenPage::Urls,
            ScreenPage::Statistics,
            ScreenPage::Mining,
            ScreenPage::Wifi,
            ScreenPage::Urls,
        ]
    );
}

#[test]
fn delayed_evaluation_coalesces_complete_dwell_intervals() {
    // Arrange
    let snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act
    let decision = flow
        .update(7_000 + 3 * 10_500, &snapshot)
        .expect("coalesced carousel");

    // Assert
    assert_eq!(decision.page, ScreenPage::Wifi);
}

#[test]
fn released_priority_resumes_carousel_after_intro_completion() {
    // Arrange
    let mut snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);
    flow.update(7_000, &snapshot).expect("complete intro");
    snapshot.overheat = true;
    flow.update(7_500, &snapshot).expect("priority");

    // Act
    snapshot.overheat = false;
    let resumed = flow.update(8_000, &snapshot).expect("resume");

    // Assert
    assert_eq!(resumed.page, ScreenPage::Urls);
}

#[test]
fn identify_and_new_block_force_priority_visibility() {
    // Arrange
    let mut snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act / Assert
    snapshot.identify_active = true;
    let identify = flow.update(500, &snapshot).expect("identify");
    assert!(identify.priority_visible);
    assert!(identify.private_frame_line(1).contains("IDENTIFY"));

    snapshot.identify_active = false;
    snapshot.show_new_block = true;
    let block = flow.update(1_000, &snapshot).expect("block");
    assert_eq!(block.page, ScreenPage::Statistics);
    assert!(block.priority_visible);
    assert!(block.frame.private_lines()[2].contains("BLOCK"));
}

#[test]
fn notification_deltas_are_one_evaluation_and_pause_is_steady() {
    // Arrange
    let mut snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(0, &snapshot);

    // Act
    snapshot.shares_accepted = 1;
    snapshot.shares_rejected = 1;
    snapshot.work_received = 1;
    let changed = flow.update(500, &snapshot).expect("delta");
    let steady = flow.update(1_000, &snapshot).expect("steady");
    snapshot.mining_paused = true;
    let paused = flow.update(1_500, &snapshot).expect("paused");

    // Assert
    assert!(changed.frame.private_lines()[0].ends_with("XWA"));
    assert!(!steady.frame.private_lines()[0].ends_with("XWA"));
    assert!(paused.frame.private_lines()[0].ends_with("||"));
}

#[test]
fn frames_are_bounded_sanitized_and_debug_redacted() {
    // Arrange
    let mut snapshot = connected_snapshot();
    snapshot.pool_host = "sensitive-sentinel\nvalue-that-is-much-longer-than-one-line".to_owned();
    let mut flow = ScreenFlow::new(0, &snapshot);
    flow.update(3_500, &snapshot).expect("second intro");

    // Act
    let decision = flow.update(7_000, &snapshot).expect("URL frame");
    let rendered_debug = format!("{:?} {:?}", snapshot, decision.frame);

    // Assert
    assert!(decision.frame.fits_ultra205());
    assert!(decision
        .frame
        .private_lines()
        .into_iter()
        .all(|line| !line.contains('\n') && !line.contains('\r')));
    assert!(!rendered_debug.contains("sensitive-sentinel"));
    assert!(!rendered_debug.contains("fixture-network"));
}

#[test]
fn unavailable_values_and_signal_thresholds_are_explicit() {
    // Arrange
    let mut snapshot = connected_snapshot();
    snapshot.maybe_rssi_dbm = None;
    let mut flow = ScreenFlow::new(0, &snapshot);
    for time in [3_500, 7_000, 17_500, 28_000, 38_500] {
        flow.update(time, &snapshot).expect("advance");
    }

    // Act
    let wifi = flow.update(39_000, &snapshot).expect("wifi");

    // Assert
    assert_eq!(wifi.page, ScreenPage::Wifi);
    assert_eq!(wifi.frame.private_lines()[1], "RSSI: -- dBm");
    assert_eq!(wifi.frame.private_lines()[2], "Signal: --");
}

#[test]
fn regressed_clock_fails_without_advancing_state() {
    // Arrange
    let snapshot = connected_snapshot();
    let mut flow = ScreenFlow::new(100, &snapshot);
    let before = flow.clone();

    // Act / Assert
    assert_eq!(
        flow.update(99, &snapshot),
        Err(ScreenFlowError::ClockRegressed)
    );
    assert_eq!(flow, before);
}

#[test]
fn high_monotonic_timestamps_do_not_wrap_deadlines() {
    // Arrange
    let snapshot = connected_snapshot();
    let started_at_ms = u64::MAX - INTRO_DELAY_MS - SCREEN_UPDATE_MS;
    let mut flow = ScreenFlow::new(started_at_ms, &snapshot);

    // Act
    let decision = flow.update(u64::MAX, &snapshot).expect("bounded deadline");

    // Assert
    assert_eq!(decision.page, ScreenPage::OpenSourceIntro);
}

trait PrivateDecisionLine {
    fn private_frame_line(&self, index: usize) -> &str;
}

impl PrivateDecisionLine for ScreenDecision {
    fn private_frame_line(&self, index: usize) -> &str {
        self.frame.private_lines()[index]
    }
}
