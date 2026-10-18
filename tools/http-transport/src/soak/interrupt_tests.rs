//! A lease that interrupted local mining (ADR-0038): the run intent survives the lease, and the
//! handover may restart the share counters once.
use super::judge::{judge_with, FinalBaseline, JournalLine};
use super::tests::{clock, complete, expected, ORIGIN};

/// The complete soak, after a lease that interrupted local mining: the run intent shows throughout,
/// and the local run's 40 shares stay counted until the lease's pool session restarts them at -6 s.
fn interrupted() -> Vec<JournalLine> {
    let mut lines = complete();
    for line in &mut lines {
        let active_ms = line.host_unix_ms as i64 - ORIGIN as i64;
        let sample = line.sample.as_mut().expect("sample");
        sample.mining_paused = false;
        if active_ms < -6_000 {
            sample.shares_accepted = 40;
        }
    }
    lines
}

fn judge(lines: &[JournalLine]) -> super::judge::SoakJudgement {
    judge_with(lines, &clock(), &expected(), FinalBaseline::Resumable)
}

#[test]
fn an_interrupting_lease_that_ends_with_the_run_intent_passes() {
    // Arrange
    let lines = interrupted();
    // Act
    let judgement = judge(&lines);
    // Assert
    assert_eq!(judgement.failures, Vec::<&str>::new());
}

#[test]
fn shares_count_from_the_lease_session_after_the_handover_restart() {
    // Arrange
    let lines = interrupted();
    // Act
    let judgement = judge(&lines);
    // Assert
    assert_eq!(judgement.accepted_shares, 6);
}

#[test]
fn an_interrupting_lease_that_ends_paused_fails() {
    // Arrange
    let mut lines = interrupted();
    for line in lines.iter_mut().rev().take(4) {
        line.sample.as_mut().expect("sample").mining_paused = true;
    }
    // Act
    let judgement = judge(&lines);
    // Assert
    assert!(judgement.failures.contains(&"final_state_not_resumable"));
}

/// The complete soak, whose only share restart falls at `restart_ms` instead of the handover.
fn restarted_at(restart_ms: i64) -> Vec<JournalLine> {
    let mut lines = complete();
    for line in &mut lines {
        let active_ms = line.host_unix_ms as i64 - ORIGIN as i64;
        let sample = line.sample.as_mut().expect("sample");
        sample.mining_paused = false;
        if active_ms < restart_ms {
            sample.shares_accepted = 40;
        }
    }
    lines
}

#[test]
fn a_share_restart_while_mining_regresses() {
    // Arrange: the only restart comes in the middle of the lease.
    let lines = restarted_at(300_000);
    // Act
    let judgement = judge(&lines);
    // Assert
    assert!(judgement.failures.contains(&"counter_regressed"));
}

#[test]
fn a_restart_with_the_first_mining_sample_is_the_handover() {
    // Arrange: as on hardware, the first mining sample after the idle samples carries the restart.
    let lines = restarted_at(0);
    // Act
    let judgement = judge(&lines);
    // Assert
    assert_eq!(judgement.failures, Vec::<&str>::new());
    assert_eq!(judgement.accepted_shares, 6);
}

#[test]
fn a_share_restart_after_the_first_window_regresses() {
    // Arrange: mining stops for 60–61 s, and the counters restart as it comes back.
    let mut lines = restarted_at(61_000);
    for line in &mut lines {
        if (60_000..61_000).contains(&(line.host_unix_ms as i64 - ORIGIN as i64)) {
            line.sample.as_mut().expect("sample").mining_active = false;
        }
    }
    // Act
    let judgement = judge(&lines);
    // Assert
    assert!(judgement.failures.contains(&"counter_regressed"));
}

#[test]
fn an_interrupting_lease_that_ends_mining_fails() {
    // Arrange
    let mut lines = interrupted();
    for line in lines.iter_mut().rev().take(4) {
        line.sample.as_mut().expect("sample").mining_active = true;
    }
    // Act
    let judgement = judge(&lines);
    // Assert
    assert!(judgement.failures.contains(&"final_state_not_resumable"));
}

#[test]
fn a_second_share_restart_regresses() {
    // Arrange: the counters restart again in the middle of the lease.
    let mut lines = interrupted();
    for line in &mut lines {
        if line.host_unix_ms as i64 - ORIGIN as i64 >= 300_000 {
            line.sample.as_mut().expect("sample").shares_accepted = 0;
        }
    }
    // Act
    let judgement = judge(&lines);
    // Assert
    assert!(judgement.failures.contains(&"counter_regressed"));
}

#[test]
fn the_paused_baseline_still_refuses_a_handover_restart() {
    // Arrange
    let lines = interrupted();
    // Act
    let judgement = judge_with(&lines, &clock(), &expected(), FinalBaseline::Paused);
    // Assert
    assert!(judgement.failures.contains(&"counter_regressed"));
}
