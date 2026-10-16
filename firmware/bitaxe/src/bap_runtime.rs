//! Pure scheduling and admission core for the firmware BAP owner.

use bitaxe_core::bap::{
    plan_command, BapAdmission, BapCommand, BapConnectionMode, BapEffect, BapFrame, BapFrameError,
    BapIngress, BapParameter, BapPendingSetting, BapPlanError, BapRequestSnapshot,
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
    /// A setting awaiting its effect; its acknowledgement is sent only after the
    /// effect succeeds.
    pub maybe_setting: Option<BapPendingSetting>,
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
                dispatch.maybe_setting = Some(BapPendingSetting {
                    setting: setting.clone(),
                    restart: *restart,
                    acknowledgement: std::mem::take(&mut dispatch.responses),
                });
            }
        }
        Ok(dispatch)
    }

    /// The reference runs its subscription task in every mode, so existing
    /// subscriptions keep publishing and timing out after a Wi-Fi loss, beside
    /// the access-point announcements.
    pub fn poll(&mut self, now_ms: u64) -> Vec<BapRuntimeAction> {
        let mut actions = Vec::new();
        if self.mode == BapConnectionMode::AccessPoint && now_ms >= self.next_ap_announcement_ms {
            self.next_ap_announcement_ms = coalesced_deadline(
                self.next_ap_announcement_ms,
                BAP_AP_ANNOUNCEMENT_INTERVAL_MS,
                now_ms,
            );
            actions.push(BapRuntimeAction::AnnounceAccessPoint);
        }
        actions.extend(self.poll_subscriptions(now_ms));
        actions
    }

    fn poll_subscriptions(&mut self, now_ms: u64) -> Vec<BapRuntimeAction> {
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
        // The reference walks its subscription table in `BAP_PARAM` order, so
        // several updates due on one tick go out in that order.
        let rank = reference_rank(parameter);
        let index = self
            .subscriptions
            .partition_point(|existing| reference_rank(existing.parameter) < rank);
        self.subscriptions.insert(index, subscription);
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

/// The last value sent for each update token, like the reference's `last_values`:
/// `BAP_send_if_changed` compares and resends each token on its own, so a
/// `chipTemp` change never resends an unchanged `vrTemp`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BapLastValues {
    sent: Vec<(BapParameter, String, Option<String>)>,
}

impl BapLastValues {
    /// Keeps only the frames whose value changed since this token was last sent,
    /// and records them as sent.
    pub fn changed(&mut self, parameter: BapParameter, frames: Vec<BapFrame>) -> Vec<BapFrame> {
        frames
            .into_iter()
            .filter(|frame| {
                let token = frame.parameter_token();
                let value = frame.value().map(str::to_owned);
                let slot = self
                    .sent
                    .iter_mut()
                    .find(|(owner, sent, _)| *owner == parameter && sent == token);
                match slot {
                    Some((_, _, last)) if *last == value => false,
                    Some((_, _, last)) => {
                        *last = value;
                        true
                    }
                    None => {
                        self.sent.push((parameter, token.to_owned(), value));
                        true
                    }
                }
            })
            .collect()
    }

    /// Forgets a parameter's values, as each SUB does in the reference. Clearing them
    /// on timeout as well is equivalent: only a new SUB can publish that parameter again.
    pub fn reset(&mut self, parameter: BapParameter) {
        self.sent.retain(|(owner, _, _)| *owner != parameter);
    }
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

fn reference_rank(parameter: BapParameter) -> usize {
    BapParameter::ALL
        .iter()
        .position(|candidate| *candidate == parameter)
        .unwrap_or(BapParameter::ALL.len())
}

#[cfg(test)]
#[path = "bap_runtime/tests.rs"]
mod tests;
