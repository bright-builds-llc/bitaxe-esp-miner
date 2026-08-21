//! Pure scheduling and admission core for the firmware BAP owner.

use bitaxe_core::bap::{
    plan_command, BapAdmission, BapConnectionMode, BapEffect, BapFrame, BapFrameError, BapIngress,
    BapParameter, BapPlanError, BapRequestSnapshot, BapRestartPolicy, BapSettingIntent,
    BAP_MAX_MESSAGE_LEN,
};

pub const BAP_AP_ANNOUNCEMENT_INTERVAL_MS: u64 = 5_000;

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
}

impl BapDispatch {
    fn no_response() -> Self {
        Self {
            responses: Vec::new(),
            maybe_setting: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Subscription {
    parameter: BapParameter,
    interval_ms: u64,
    next_due_ms: u64,
    expires_at_ms: u64,
}

#[derive(Debug, Clone)]
pub struct BapRuntime {
    ingress: BapIngress,
    mode: BapConnectionMode,
    subscriptions: Vec<Subscription>,
    next_ap_announcement_ms: u64,
}

impl Default for BapRuntime {
    fn default() -> Self {
        Self {
            ingress: BapIngress::default(),
            mode: BapConnectionMode::AccessPoint,
            subscriptions: Vec::new(),
            next_ap_announcement_ms: 0,
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

    pub fn admit(
        &mut self,
        input: &[u8],
        now_ms: u64,
        maybe_snapshot: Option<&BapRequestSnapshot>,
    ) -> Result<BapDispatch, BapRuntimeError> {
        let admission = self.ingress.admit(input, now_ms)?;
        let BapAdmission::Accepted(frame) = admission else {
            return Ok(BapDispatch::no_response());
        };
        let plan = plan_command(&frame, self.mode, maybe_snapshot)?;
        let mut dispatch = BapDispatch {
            responses: plan.responses().to_vec(),
            maybe_setting: None,
        };
        let Some(effect) = plan.effect() else {
            return Ok(dispatch);
        };
        match effect {
            BapEffect::Subscribe {
                parameter,
                interval_ms,
                timeout_ms,
            } => self.subscribe(*parameter, *interval_ms, *timeout_ms, now_ms),
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

        let mut actions = Vec::new();
        self.subscriptions.retain_mut(|subscription| {
            if now_ms >= subscription.expires_at_ms {
                actions.push(BapRuntimeAction::SubscriptionTimedOut(
                    subscription.parameter,
                ));
                return false;
            }
            if now_ms >= subscription.next_due_ms {
                actions.push(BapRuntimeAction::PublishSubscription(
                    subscription.parameter,
                ));
                subscription.next_due_ms =
                    coalesced_deadline(subscription.next_due_ms, subscription.interval_ms, now_ms);
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
        let interval_ms = u64::from(interval_ms);
        let subscription = Subscription {
            parameter,
            interval_ms,
            next_due_ms: now_ms.saturating_add(interval_ms),
            expires_at_ms: now_ms.saturating_add(u64::from(timeout_ms)),
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
    use bitaxe_core::bap::{BapCommand, BAP_SUBSCRIPTION_TIMEOUT_MS};

    fn snapshot() -> BapRequestSnapshot {
        BapRequestSnapshot {
            device_model: "Ultra 205".to_owned(),
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

    #[test]
    fn runtime_owns_subscription_renewal_cadence_timeout_and_unsubscribe() {
        // Arrange
        let mut runtime = BapRuntime::default();
        runtime.set_mode(BapConnectionMode::Connected, 100);

        // Act
        let subscribed = runtime
            .admit(b"$BAP,SUB,hashrate,1000*00\r\n", 100, Some(&snapshot()))
            .expect("compatible subscription plans");
        let before = runtime.poll(1_099);
        let due = runtime.poll(1_100);
        let coalesced = runtime.poll(5_100);
        let timeout = runtime.poll(100 + u64::from(BAP_SUBSCRIPTION_TIMEOUT_MS));
        let unsubscribed = runtime
            .admit(b"$BAP,UNSUB,hashrate\r\n", 400_000, Some(&snapshot()))
            .expect("compatible unsubscribe plans");

        // Assert
        assert_eq!(subscribed.responses.len(), 1);
        assert!(before.is_empty());
        assert_eq!(
            due,
            vec![BapRuntimeAction::PublishSubscription(
                BapParameter::Hashrate
            )]
        );
        assert_eq!(
            coalesced,
            vec![BapRuntimeAction::PublishSubscription(
                BapParameter::Hashrate
            )]
        );
        assert_eq!(
            timeout,
            vec![BapRuntimeAction::SubscriptionTimedOut(
                BapParameter::Hashrate
            )]
        );
        assert_eq!(unsubscribed.responses.len(), 1);
        assert!(runtime.poll(500_000).is_empty());
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
            .admit(frame.as_bytes(), 0, Some(&snapshot()))
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
