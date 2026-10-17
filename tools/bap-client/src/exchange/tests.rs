use super::*;
use crate::test_link::{frame, Scripted};
use bitaxe_core::bap::BapFrame;

const TIMING: Timing = Timing {
    answer_ms: 1_000,
    quiet_ms: 200,
    listen_ms: 3_000,
};

#[test]
fn a_request_collects_every_value_until_the_device_goes_quiet() {
    // Arrange
    let first = frame(BapCommand::Response, "deviceModel", "Ultra");
    let second = frame(BapCommand::Response, "asicModel", "BM1366");
    let mut link = Scripted::new(&[(10, &first), (20, &second)]);
    // Act
    let report = run(
        &Operation::Request(BapParameter::SystemInfo),
        &mut link,
        TIMING,
    )
    .expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::Answered);
    assert_eq!(report.frames.len(), 2);
}

#[test]
fn a_request_with_no_answer_times_out() {
    // Arrange
    let mut link = Scripted::new(&[]);
    // Act
    let report = run(
        &Operation::Request(BapParameter::Hashrate),
        &mut link,
        TIMING,
    )
    .expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::TimedOut);
    assert_eq!(link.now, TIMING.answer_ms);
}

#[test]
fn an_error_for_the_parameter_rejects_the_request() {
    // Arrange
    let error = frame(BapCommand::Error, "hashrate", "ap_mode_no_requests");
    let mut link = Scripted::new(&[(10, &error)]);
    // Act
    let report = run(
        &Operation::Request(BapParameter::Hashrate),
        &mut link,
        TIMING,
    )
    .expect("run");
    // Assert
    assert_eq!(
        report.outcome,
        Outcome::Rejected("ap_mode_no_requests".into())
    );
}

#[test]
fn a_subscription_listens_then_unsubscribes() {
    // Arrange
    let subscribed = frame(BapCommand::Acknowledge, "hashrate", "subscribed");
    let update = frame(BapCommand::Response, "hashrate", "512.3");
    let unsubscribed = frame(BapCommand::Acknowledge, "hashrate", "unsubscribed");
    let mut link = Scripted::new(&[(10, &subscribed), (1_000, &update), (3_100, &unsubscribed)]);
    let operation = Operation::Subscribe {
        parameter: BapParameter::Hashrate,
        maybe_interval_ms: Some(1_000),
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::Confirmed);
    let commands: Vec<_> = link.sent.iter().map(BapFrame::command).collect();
    assert_eq!(commands, [BapCommand::Subscribe, BapCommand::Unsubscribe]);
    assert_eq!(link.sent[0].value(), Some("1000"));
    assert!(report
        .frames
        .iter()
        .any(|frame| frame.value() == Some("512.3")));
}

#[test]
fn an_unacknowledged_unsubscribe_is_reported() {
    // Arrange
    let subscribed = frame(BapCommand::Acknowledge, "hashrate", "subscribed");
    let mut link = Scripted::new(&[(10, &subscribed)]);
    let operation = Operation::Subscribe {
        parameter: BapParameter::Hashrate,
        maybe_interval_ms: None,
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::UnsubscribeUnconfirmed);
}

#[test]
fn a_setting_is_confirmed_and_a_following_restart_status_is_kept() {
    // Arrange
    let acknowledged = frame(BapCommand::Acknowledge, "password", "password_set");
    let restarting = frame(BapCommand::Status, "status", "restarting");
    let mut link = Scripted::new(&[(10, &acknowledged), (50, &restarting)]);
    let operation = Operation::Set {
        parameter: BapParameter::Password,
        value: "secret".into(),
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::Confirmed);
    assert!(report
        .frames
        .iter()
        .any(|frame| frame.command() == BapCommand::Status));
}

#[test]
fn lines_split_across_reads_and_garbage_are_handled() {
    // Arrange
    let answer = frame(BapCommand::Response, "hashrate", "512.3");
    let (head, tail) = answer.split_at(7);
    let garbage = "not a frame\r\n";
    let mut link = Scripted::new(&[(5, garbage), (10, head), (15, tail)]);
    // Act
    let report = run(
        &Operation::Request(BapParameter::Hashrate),
        &mut link,
        TIMING,
    )
    .expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::Answered);
    assert_eq!(report.malformed_lines, 1);
    assert_eq!(report.frames.len(), 1);
}

#[test]
fn an_oversized_line_is_discarded_whole() {
    // Arrange
    let oversized = format!("${}\r\n", "A".repeat(BAP_MAX_MESSAGE_LEN * 2));
    let answer = frame(BapCommand::Response, "hashrate", "512.3");
    let mut link = Scripted::new(&[(5, &oversized), (10, &answer)]);
    // Act
    let report = run(
        &Operation::Request(BapParameter::Hashrate),
        &mut link,
        TIMING,
    )
    .expect("run");
    // Assert
    assert_eq!(report.malformed_lines, 1);
    assert_eq!(report.frames.len(), 1);
}

#[test]
fn a_fan_speed_setting_without_a_refusal_is_sent_silently() {
    // Arrange: the reference saves fan_speed without an acknowledgement.
    let mut link = Scripted::new(&[]);
    let operation = Operation::Set {
        parameter: BapParameter::FanSpeed,
        value: "80".into(),
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::SentSilently);
}

#[test]
fn a_refused_fan_speed_setting_is_rejected() {
    // Arrange
    let refused = frame(BapCommand::Error, "fan_speed", "set_failed");
    let mut link = Scripted::new(&[(10, &refused)]);
    let operation = Operation::Set {
        parameter: BapParameter::FanSpeed,
        value: "80".into(),
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::Rejected("set_failed".into()));
}

#[test]
fn another_unanswered_setting_still_times_out() {
    // Arrange
    let mut link = Scripted::new(&[]);
    let operation = Operation::Set {
        parameter: BapParameter::AutoFan,
        value: "1".into(),
    };
    // Act
    let report = run(&operation, &mut link, TIMING).expect("run");
    // Assert
    assert_eq!(report.outcome, Outcome::TimedOut);
}
