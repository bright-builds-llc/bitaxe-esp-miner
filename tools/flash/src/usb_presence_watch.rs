//! Receive-nothing presence watcher for one admitted physical USB device (BWG-007 physical checkpoints).
//!
//! It never opens the serial node. It samples the platform USB registry at a bounded interval, keeps the
//! stable physical identity separate from the per-enumeration digest (AGENTS.md, Ultra 205 Serial Session
//! Reuse), and writes one ProtectedOperational JSON line per change. It stops cleanly on stdin EOF.
use crate::*;
use bitaxe_device_session::{sample_usb_presence, UsbPresenceSample};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const SCHEMA: &str = "bwg-usb-presence-watch-v1";
const STOP_POLL: Duration = Duration::from_millis(50);
/// Consecutive probe errors tolerated before the watcher fails (about 2 s at the default interval). A device pulled
/// mid-scan can vanish between the registry read and the holder probe (BWG-007 attempt-002, both-power removal).
const PROBE_FAILURE_LIMIT: u8 = 8;

/// Counts consecutive probe errors. A transient error yields no observation for that tick, so it is never read as
/// absence or presence; only a persistent fault ends the watch.
#[derive(Debug, Default)]
pub(crate) struct ProbeFailures {
    consecutive: u8,
}

impl ProbeFailures {
    /// Records one probe error and reports whether the watcher must now fail.
    pub(crate) fn record_failure(&mut self) -> bool {
        self.consecutive = self.consecutive.saturating_add(1);
        self.consecutive >= PROBE_FAILURE_LIMIT
    }

    pub(crate) fn record_success(&mut self) {
        self.consecutive = 0;
    }
}

#[derive(Debug, Args)]
pub(crate) struct UsbPresenceWatchCommand {
    /// Stable physical identity digest printed by `just detect-ultra205`.
    #[arg(long = "physical-identity", value_parser = parse_physical_identity)]
    physical_identity: String,
    /// Sampling interval; each sample runs one registry and one holder probe.
    #[arg(long = "interval-ms", default_value_t = 250, value_parser = clap::value_parser!(u64).range(100..=5_000))]
    interval_ms: u64,
    /// Continuous ready presence with one enumeration required before `stable`.
    #[arg(long = "stable-ms", default_value_t = 3_000, value_parser = clap::value_parser!(u64).range(500..=60_000))]
    stable_ms: u64,
}

fn parse_physical_identity(value: &str) -> std::result::Result<String, String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        Ok(value.to_owned())
    } else {
        Err("physical identity must be 64 lower-case hex characters".to_owned())
    }
}

/// One registry observation, already reduced to what the tracker needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PresenceObservation {
    Absent,
    Present(PresenceFacts),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PresenceFacts {
    pub(crate) enumeration_sha256: String,
    pub(crate) holder_count: u16,
    /// Accessible, unheld and classified as the Serial/JTAG runtime profile.
    pub(crate) ready: bool,
}

impl From<UsbPresenceSample> for PresenceFacts {
    fn from(sample: UsbPresenceSample) -> Self {
        let ready = sample.accessible
            && sample.holder_count == 0
            && sample.profile == UsbProfile::SerialJtagRuntime;
        Self {
            enumeration_sha256: sample.enumeration_sha256,
            holder_count: sample.holder_count,
            ready,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PresenceEventKind {
    Started,
    Present,
    Absent,
    Reappeared,
    EnumerationChanged,
    Stable,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PresenceEvent {
    schema: &'static str,
    sequence: u64,
    elapsed_ms: u64,
    event: PresenceEventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    enumeration_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enumeration_changed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    holder_count: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presence {
    Unknown,
    Present,
    Absent,
}

/// Pure change detector: physical identity is fixed by the caller, enumeration is tracked here.
#[derive(Debug)]
pub(crate) struct PresenceTracker {
    stable_ms: u64,
    presence: Presence,
    maybe_last_enumeration: Option<String>,
    maybe_ready_since_ms: Option<u64>,
    stable_emitted: bool,
    sequence: u64,
}

impl PresenceTracker {
    pub(crate) const fn new(stable_ms: u64) -> Self {
        Self {
            stable_ms,
            presence: Presence::Unknown,
            maybe_last_enumeration: None,
            maybe_ready_since_ms: None,
            stable_emitted: false,
            sequence: 0,
        }
    }

    pub(crate) fn event(
        &mut self,
        elapsed_ms: u64,
        kind: PresenceEventKind,
        maybe_facts: Option<&PresenceFacts>,
    ) -> PresenceEvent {
        self.sequence += 1;
        PresenceEvent {
            schema: SCHEMA,
            sequence: self.sequence,
            elapsed_ms,
            event: kind,
            enumeration_sha256: maybe_facts.map(|facts| facts.enumeration_sha256.clone()),
            enumeration_changed: None,
            holder_count: maybe_facts.map(|facts| facts.holder_count),
            ready: maybe_facts.map(|facts| facts.ready),
            category: None,
        }
    }

    fn reset_stability(&mut self) {
        self.maybe_ready_since_ms = None;
        self.stable_emitted = false;
    }

    /// Feed one sample and return the change events it causes, in order.
    pub(crate) fn observe(
        &mut self,
        elapsed_ms: u64,
        observation: &PresenceObservation,
    ) -> Vec<PresenceEvent> {
        let PresenceObservation::Present(facts) = observation else {
            if self.presence == Presence::Absent {
                return Vec::new();
            }
            self.presence = Presence::Absent;
            self.reset_stability();
            return vec![self.event(elapsed_ms, PresenceEventKind::Absent, None)];
        };
        let mut events = Vec::new();
        let maybe_previous = self.maybe_last_enumeration.clone();
        let changed = maybe_previous
            .as_ref()
            .map(|previous| previous != &facts.enumeration_sha256);
        match (self.presence, changed) {
            (Presence::Absent, Some(enumeration_changed)) => {
                let mut event = self.event(elapsed_ms, PresenceEventKind::Reappeared, Some(facts));
                event.enumeration_changed = Some(enumeration_changed);
                events.push(event);
                self.reset_stability();
            }
            (Presence::Unknown | Presence::Absent, _) => {
                events.push(self.event(elapsed_ms, PresenceEventKind::Present, Some(facts)));
                self.reset_stability();
            }
            (Presence::Present, Some(true)) => {
                let mut event = self.event(
                    elapsed_ms,
                    PresenceEventKind::EnumerationChanged,
                    Some(facts),
                );
                event.enumeration_changed = Some(true);
                events.push(event);
                self.reset_stability();
            }
            (Presence::Present, _) => {}
        }
        self.presence = Presence::Present;
        self.maybe_last_enumeration = Some(facts.enumeration_sha256.clone());
        if let Some(event) = self.observe_stability(elapsed_ms, facts) {
            events.push(event);
        }
        events
    }

    fn observe_stability(
        &mut self,
        elapsed_ms: u64,
        facts: &PresenceFacts,
    ) -> Option<PresenceEvent> {
        if !facts.ready {
            self.maybe_ready_since_ms = None;
            return None;
        }
        let since = *self.maybe_ready_since_ms.get_or_insert(elapsed_ms);
        if self.stable_emitted || elapsed_ms.saturating_sub(since) < self.stable_ms {
            return None;
        }
        self.stable_emitted = true;
        Some(self.event(elapsed_ms, PresenceEventKind::Stable, Some(facts)))
    }
}

fn emit(event: &PresenceEvent) -> Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, event).context("usb_presence_watch=failed reason=encode")?;
    stdout
        .write_all(b"\n")
        .context("usb_presence_watch=failed reason=stdout")?;
    stdout
        .flush()
        .context("usb_presence_watch=failed reason=stdout")
}

fn stop_on_stdin_eof() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    std::thread::spawn(move || {
        let mut sink = [0_u8; 256];
        let mut stdin = io::stdin().lock();
        // EOF, or any read failure other than an interrupted call, means the owner is gone: stop.
        loop {
            match stdin.read(&mut sink) {
                Ok(count) if count > 0 => {}
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                _ => break,
            }
        }
        flag.store(true, Ordering::SeqCst);
    });
    stop
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Watch until stdin closes; a probe failure is terminal and reported as a typed event.
pub(crate) fn run(command: &UsbPresenceWatchCommand) -> Result<()> {
    let stop = stop_on_stdin_eof();
    let started = Instant::now();
    let mut tracker = PresenceTracker::new(command.stable_ms);
    emit(&tracker.event(0, PresenceEventKind::Started, None))?;
    let interval = Duration::from_millis(command.interval_ms);
    let mut probe_failures = ProbeFailures::default();
    loop {
        if stop.load(Ordering::SeqCst) {
            return emit(&tracker.event(elapsed_ms(started), PresenceEventKind::Stopped, None));
        }
        let maybe_observation = match sample_usb_presence(&command.physical_identity) {
            Ok(maybe_sample) => {
                probe_failures.record_success();
                Some(maybe_sample.map_or(PresenceObservation::Absent, |sample| {
                    PresenceObservation::Present(sample.into())
                }))
            }
            Err(_) if !probe_failures.record_failure() => None,
            Err(_) => {
                let mut failed =
                    tracker.event(elapsed_ms(started), PresenceEventKind::Failed, None);
                failed.category = Some("probe_failed");
                emit(&failed)?;
                bail!("usb_presence_watch=failed reason=probe_failed");
            }
        };
        if let Some(observation) = maybe_observation {
            for event in tracker.observe(elapsed_ms(started), &observation) {
                emit(&event)?;
            }
        }
        let deadline = Instant::now() + interval;
        while Instant::now() < deadline && !stop.load(Ordering::SeqCst) {
            std::thread::sleep(STOP_POLL.min(deadline.saturating_duration_since(Instant::now())));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn present(enumeration: &str, ready: bool) -> PresenceObservation {
        PresenceObservation::Present(PresenceFacts {
            enumeration_sha256: enumeration.repeat(64),
            holder_count: u16::from(!ready),
            ready,
        })
    }

    fn kinds(events: &[PresenceEvent]) -> Vec<PresenceEventKind> {
        events.iter().map(|event| event.event).collect()
    }

    #[test]
    fn a_transient_probe_error_does_not_fail_the_watch() {
        // Arrange
        let mut failures = ProbeFailures::default();
        // Act
        let must_fail = failures.record_failure();
        // Assert
        assert!(!must_fail);
    }

    #[test]
    fn a_persistent_probe_fault_fails_the_watch() {
        // Arrange
        let mut failures = ProbeFailures::default();
        // Act
        let results: Vec<bool> = (0..PROBE_FAILURE_LIMIT)
            .map(|_| failures.record_failure())
            .collect();
        // Assert
        assert_eq!(results.iter().filter(|must_fail| **must_fail).count(), 1);
        assert!(results.last().copied().unwrap_or(false));
    }

    #[test]
    fn a_successful_probe_resets_the_failure_count() {
        // Arrange
        let mut failures = ProbeFailures::default();
        for _ in 1..PROBE_FAILURE_LIMIT {
            failures.record_failure();
        }
        // Act
        failures.record_success();
        let must_fail = failures.record_failure();
        // Assert
        assert!(!must_fail);
    }

    #[test]
    fn first_presence_is_reported_once() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        // Act
        let first = tracker.observe(0, &present("a", false));
        let second = tracker.observe(250, &present("a", false));
        // Assert
        assert_eq!(kinds(&first), [PresenceEventKind::Present]);
        assert!(second.is_empty());
    }

    #[test]
    fn removal_is_reported_once() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", false));
        // Act
        let first = tracker.observe(250, &PresenceObservation::Absent);
        let second = tracker.observe(500, &PresenceObservation::Absent);
        // Assert
        assert_eq!(kinds(&first), [PresenceEventKind::Absent]);
        assert!(second.is_empty());
    }

    #[test]
    fn reappearance_with_a_new_enumeration_is_marked_changed() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", false));
        tracker.observe(250, &PresenceObservation::Absent);
        // Act
        let events = tracker.observe(6_000, &present("b", false));
        // Assert
        assert_eq!(kinds(&events), [PresenceEventKind::Reappeared]);
        assert_eq!(events[0].enumeration_changed, Some(true));
        assert_eq!(
            events[0].enumeration_sha256.as_deref(),
            Some("b".repeat(64).as_str())
        );
    }

    #[test]
    fn reappearance_with_the_same_enumeration_is_marked_unchanged() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", false));
        tracker.observe(250, &PresenceObservation::Absent);
        // Act
        let events = tracker.observe(500, &present("a", false));
        // Assert
        assert_eq!(events[0].enumeration_changed, Some(false));
    }

    #[test]
    fn enumeration_change_between_samples_is_reported_without_absence() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", false));
        // Act
        let events = tracker.observe(250, &present("b", false));
        // Assert
        assert_eq!(kinds(&events), [PresenceEventKind::EnumerationChanged]);
    }

    #[test]
    fn stability_needs_continuous_ready_presence_for_the_whole_bound() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", true));
        // Act
        let early = tracker.observe(999, &present("a", true));
        let due = tracker.observe(1_000, &present("a", true));
        let after = tracker.observe(2_000, &present("a", true));
        // Assert
        assert!(early.is_empty());
        assert_eq!(kinds(&due), [PresenceEventKind::Stable]);
        assert!(after.is_empty());
    }

    #[test]
    fn a_holder_restarts_the_stability_bound() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", true));
        tracker.observe(600, &present("a", false));
        // Act
        let before = tracker.observe(1_200, &present("a", true));
        let due = tracker.observe(2_200, &present("a", true));
        // Assert
        assert!(before.is_empty());
        assert_eq!(kinds(&due), [PresenceEventKind::Stable]);
    }

    #[test]
    fn re_enumeration_restarts_the_stability_bound() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        tracker.observe(0, &present("a", true));
        // Act
        let changed = tracker.observe(900, &present("b", true));
        let early = tracker.observe(1_500, &present("b", true));
        let due = tracker.observe(1_900, &present("b", true));
        // Assert
        assert_eq!(kinds(&changed), [PresenceEventKind::EnumerationChanged]);
        assert!(early.is_empty());
        assert_eq!(kinds(&due), [PresenceEventKind::Stable]);
    }

    #[test]
    fn events_are_sequenced_and_never_name_the_device_node() {
        // Arrange
        let mut tracker = PresenceTracker::new(1_000);
        // Act
        let event = tracker.observe(0, &present("a", false)).remove(0);
        let encoded = serde_json::to_string(&event).expect("event encodes");
        // Assert
        assert_eq!(event.sequence, 1);
        assert!(!encoded.contains("/dev/"));
        assert!(encoded.contains("\"schema\":\"bwg-usb-presence-watch-v1\""));
    }

    #[test]
    fn physical_identity_must_be_a_lower_case_digest() {
        // Arrange / Act / Assert
        assert!(parse_physical_identity(&"a".repeat(64)).is_ok());
        assert!(parse_physical_identity(&"A".repeat(64)).is_err());
        assert!(parse_physical_identity("/dev/cu.usbmodem1").is_err());
    }
}
