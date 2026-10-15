//! Pure scheduling and admission core for the firmware BAP owner.

use bitaxe_core::bap::{
    plan_command, BapAdmission, BapCommand, BapConnectionMode, BapEffect, BapFrame, BapFrameError,
    BapIngress, BapParameter, BapPlanError, BapRequestSnapshot, BapRestartPolicy, BapSettingIntent,
    BAP_MAX_MESSAGE_LEN,
};

pub const BAP_AP_ANNOUNCEMENT_INTERVAL_MS: u64 = 5_000;
/// The reference subscription task wakes once a second (`vTaskDelay(1000)`), so no
/// subscription publishes more often than this, whatever interval it asked for.
pub const BAP_SUBSCRIPTION_TICK_MS: u64 = 1_000;

#[derive(Debug, Clone, PartialEq)]
pub enum BapRuntimeAction {
    PublishSubscription(BapParameter),
    SubscriptionTimedOut(BapParameter),
    AnnounceAccessPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BapDispatch {
    pub responses: Vec<BapFrame>,
    pub maybe_setting: Option<(BapSettingIntent, BapRestartPolicy)>,
    /// A (re)subscription: the reference clears this parameter's last-sent values,
    /// so its next update is sent even when unchanged.
    pub maybe_subscribed: Option<BapParameter>,
}

impl BapDispatch {
    fn no_response() -> Self {
        Self {
            responses: Vec::new(),
            maybe_setting: None,
            maybe_subscribed: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Subscription {
    parameter: BapParameter,
    interval_ms: u64,
    subscribed_at_ms: u64,
    /// `None` until the first update, like the reference's `last_response = 0`.
    maybe_last_response_ms: Option<u64>,
    timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct BapRuntime {
    ingress: BapIngress,
    mode: BapConnectionMode,
    subscriptions: Vec<Subscription>,
    next_ap_announcement_ms: u64,
    next_subscription_tick_ms: u64,
}

impl Default for BapRuntime {
    fn default() -> Self {
        Self {
            ingress: BapIngress::default(),
            mode: BapConnectionMode::AccessPoint,
            subscriptions: Vec::new(),
            next_ap_announcement_ms: 0,
            next_subscription_tick_ms: 0,
        }
    }
}

impl BapRuntime {
    pub fn set_mode(&mut self, mode: BapConnectionMode, now_ms: u64) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        if mode == BapConnectionMode::AccessPoint {
            self.next_ap_announcement_ms = now_ms;
        }
    }

    /// Admits one inbound frame. `snapshot` runs only for an accepted request, the
    /// one command that answers from runtime values: each call takes the global
    /// operator-snapshot publication lock that HTTP and WebSocket also hold.
    pub fn admit(
        &mut self,
        input: &[u8],
        now_ms: u64,
        snapshot: impl FnOnce() -> Option<BapRequestSnapshot>,
    ) -> Result<BapDispatch, BapRuntimeError> {
        let admission = self.ingress.admit(input, now_ms)?;
        let BapAdmission::Accepted(frame) = admission else {
            return Ok(BapDispatch::no_response());
        };
        let maybe_snapshot = (frame.command() == BapCommand::Request)
            .then(snapshot)
            .flatten();
        let plan = plan_command(&frame, self.mode, maybe_snapshot.as_ref())?;
        let mut dispatch = BapDispatch {
            responses: plan.responses().to_vec(),
            maybe_setting: None,
            maybe_subscribed: None,
        };
        let Some(effect) = plan.effect() else {
            return Ok(dispatch);
        };
        match effect {
            BapEffect::Subscribe {
                parameter,
                interval_ms,
                timeout_ms,
            } => {
                self.subscribe(*parameter, *interval_ms, *timeout_ms, now_ms);
                dispatch.maybe_subscribed = Some(*parameter);
            }
            BapEffect::Unsubscribe { parameter } => self.unsubscribe(*parameter),
            BapEffect::ApplySetting { setting, restart } => {
                dispatch.responses.clear();
                dispatch.maybe_setting = Some((setting.clone(), *restart));
            }
        }
        Ok(dispatch)
    }

    pub fn poll(&mut self, now_ms: u64) -> Vec<BapRuntimeAction> {
        if self.mode == BapConnectionMode::AccessPoint {
            if now_ms < self.next_ap_announcement_ms {
                return Vec::new();
            }
            self.next_ap_announcement_ms = coalesced_deadline(
                self.next_ap_announcement_ms,
                BAP_AP_ANNOUNCEMENT_INTERVAL_MS,
                now_ms,
            );
            return vec![BapRuntimeAction::AnnounceAccessPoint];
        }

        if now_ms < self.next_subscription_tick_ms {
            return Vec::new();
        }
        self.next_subscription_tick_ms = now_ms.saturating_add(BAP_SUBSCRIPTION_TICK_MS);
        let mut actions = Vec::new();
        self.subscriptions.retain_mut(|subscription| {
            // The reference times out strictly after the window since the last SUB.
            if now_ms.saturating_sub(subscription.subscribed_at_ms) > subscription.timeout_ms {
                actions.push(BapRuntimeAction::SubscriptionTimedOut(
                    subscription.parameter,
                ));
                return false;
            }
            let due = subscription
                .maybe_last_response_ms
                .is_none_or(|last| now_ms.saturating_sub(last) >= subscription.interval_ms);
            if due {
                subscription.maybe_last_response_ms = Some(now_ms);
                actions.push(BapRuntimeAction::PublishSubscription(
                    subscription.parameter,
                ));
            }
            true
        });
        actions
    }

    fn subscribe(
        &mut self,
        parameter: BapParameter,
        interval_ms: u32,
        timeout_ms: u32,
        now_ms: u64,
    ) {
        let subscription = Subscription {
            parameter,
            interval_ms: u64::from(interval_ms),
            subscribed_at_ms: now_ms,
            maybe_last_response_ms: None,
            timeout_ms: u64::from(timeout_ms),
        };
        if let Some(existing) = self
            .subscriptions
            .iter_mut()
            .find(|existing| existing.parameter == parameter)
        {
            *existing = subscription;
            return;
        }
        self.subscriptions.push(subscription);
    }

    fn unsubscribe(&mut self, parameter: BapParameter) {
        self.subscriptions
            .retain(|subscription| subscription.parameter != parameter);
    }
}

fn coalesced_deadline(deadline_ms: u64, interval_ms: u64, now_ms: u64) -> u64 {
    let elapsed = now_ms.saturating_sub(deadline_ms);
    let slots = elapsed / interval_ms;
    deadline_ms.saturating_add(interval_ms.saturating_mul(slots.saturating_add(1)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BapRuntimeError {
    Frame(BapFrameError),
    Plan(BapPlanError),
}

impl From<BapFrameError> for BapRuntimeError {
    fn from(error: BapFrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<BapPlanError> for BapRuntimeError {
    fn from(error: BapPlanError) -> Self {
        Self::Plan(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BapFrameAccumulator {
    bytes: Vec<u8>,
    in_frame: bool,
    discarding: bool,
}

impl Default for BapFrameAccumulator {
    fn default() -> Self {
        Self {
            bytes: Vec::with_capacity(BAP_MAX_MESSAGE_LEN),
            in_frame: false,
            discarding: false,
        }
    }
}

impl BapFrameAccumulator {
    pub fn push(&mut self, input: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        for byte in input {
            if *byte == b'$' {
                self.bytes.clear();
                self.bytes.push(*byte);
                self.in_frame = true;
                self.discarding = false;
                continue;
            }
            if !self.in_frame {
                continue;
            }
            if self.discarding {
                if matches!(*byte, b'\r' | b'\n') {
                    self.reset();
                }
                continue;
            }
            if self.bytes.len().saturating_add(1) >= BAP_MAX_MESSAGE_LEN {
                self.bytes.clear();
                self.discarding = true;
                continue;
            }
            self.bytes.push(*byte);
            if matches!(*byte, b'\r' | b'\n') {
                frames.push(std::mem::take(&mut self.bytes));
                self.in_frame = false;
            }
        }
        frames
    }

    fn reset(&mut self) {
        self.bytes.clear();
        self.in_frame = false;
        self.discarding = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitaxe_core::bap::BAP_SUBSCRIPTION_TIMEOUT_MS;

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
            Some((BapSettingIntent::WifiPassword(_), BapRestartPolicy::Always))
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
}
