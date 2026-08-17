//! ESP Task WDT ownership for the production-mining task.

use bitaxe_core::runtime_health::TaskWatchdogObservation;
use esp_idf_svc::sys;
use std::ptr;

use crate::task_watchdog_observation::{record, record_owner_progress};
use bitaxe_core::runtime_health::TaskWatchdogOwnerSubphase;

pub(super) struct ProductionTaskWatchdog {
    owns_subscription: bool,
    failure_latched: bool,
    feed_sequence: u64,
}

impl ProductionTaskWatchdog {
    pub(super) fn subscribe(now_millis: u64) -> Self {
        let mut owner = Self {
            owns_subscription: false,
            failure_latched: false,
            feed_sequence: 0,
        };
        let result = unsafe { sys::esp_task_wdt_add(ptr::null_mut()) };
        if result != sys::ESP_OK {
            owner.failure_latched = true;
            record(TaskWatchdogObservation::SubscriptionFailed);
            log::error!("production_task_watchdog=not_participating reason=subscription_failed error={result}");
            return owner;
        }

        owner.owns_subscription = true;
        owner.feed(now_millis);
        owner
    }

    pub(super) fn feed(&mut self, now_millis: u64) {
        let Some(observation) = self.maybe_feed_observation(now_millis) else {
            return;
        };
        record(observation);
    }

    pub(super) fn feed_owner_progress(
        &mut self,
        now_millis: u64,
        subphase: TaskWatchdogOwnerSubphase,
    ) {
        let maybe_observation = self.maybe_feed_observation(now_millis);
        record_owner_progress(subphase, maybe_observation);
    }

    fn maybe_feed_observation(&mut self, now_millis: u64) -> Option<TaskWatchdogObservation> {
        if !self.owns_subscription || self.failure_latched {
            return None;
        }
        let result = unsafe { sys::esp_task_wdt_reset() };
        if result != sys::ESP_OK {
            self.failure_latched = true;
            log::error!(
                "production_task_watchdog=not_participating reason=feed_failed error={result}"
            );
            return Some(TaskWatchdogObservation::FeedFailed);
        }

        self.feed_sequence = self.feed_sequence.saturating_add(1);
        Some(TaskWatchdogObservation::fed(self.feed_sequence, now_millis))
    }
}

impl Drop for ProductionTaskWatchdog {
    fn drop(&mut self) {
        if !self.owns_subscription {
            return;
        }
        let result = unsafe { sys::esp_task_wdt_delete(ptr::null_mut()) };
        if result != sys::ESP_OK {
            if !self.failure_latched {
                record(TaskWatchdogObservation::UnsubscriptionFailed);
            }
            log::error!("production_task_watchdog=cleanup_failed error={result}");
            return;
        }
        if !self.failure_latched {
            record(TaskWatchdogObservation::Unsubscribed);
        }
    }
}
