//! The crash-loop guard's shell (ADR-0038 sub-decision 3, owner decisions
//! 2026-10-17). At boot it extends or clears the NVS streak from the reset reason,
//! and starts the device paused after three abnormal resets in a row. The owner
//! loop clears the streak once the boot has been up 10 minutes, and when the
//! operator resumes. The rules are pure (`bitaxe_runtime::boot_guard`).
use bitaxe_core::device_pause::DevicePauseReason;
use bitaxe_runtime::boot_guard::{
    boot_is_stable, next_streak, starts_paused, streak_write, CRASH_STREAK_KEY,
};
use bitaxe_runtime::local_mining::shell::ResumeEdge;
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use std::sync::atomic::{AtomicU32, Ordering};

/// This boot's streak, as saved; the owner loop clears it.
static STREAK: AtomicU32 = AtomicU32::new(0);
/// A failed save waits this long before the owner loop tries again.
const SAVE_RETRY_MS: u64 = 30_000;

/// The boot mining preference after the crash-loop guard: false once it starts
/// the device paused. A failed read counts from zero and a failed save is logged;
/// the guard never stops the device from booting.
pub(crate) fn boot_preference(start_mining_on_boot: bool) -> bool {
    let previous = match crate::device_settings_store::read_u16(CRASH_STREAK_KEY) {
        Ok(maybe_streak) => maybe_streak.unwrap_or(0),
        Err(error) => {
            log::warn!("crash_loop_guard=streak_unreadable error={error}");
            0
        }
    };
    let reset = crate::boot_evidence::operator_snapshot_reset_reason_category();
    let streak = next_streak(previous, reset);
    if streak != previous {
        if let Err(error) = crate::device_settings_store::persist(vec![streak_write(streak)]) {
            log::error!("crash_loop_guard=save_failed error={error}");
        }
    }
    STREAK.store(u32::from(streak), Ordering::Release);
    if !starts_paused(streak) {
        return start_mining_on_boot;
    }
    crate::device_pause::set(DevicePauseReason::CrashLoop);
    log::warn!(
        "crash_loop_guard=paused streak={streak} reset={}",
        reset.label()
    );
    false
}

/// The owner loop's part of the guard and of every device pause.
#[derive(Default)]
pub(crate) struct BootGuardShell {
    resume_edge: ResumeEdge,
    stable_cleared: bool,
    clear_pending: bool,
    maybe_retry_at_ms: Option<u64>,
}

impl BootGuardShell {
    /// One owner tick. `another_owner_mines` hides the pause indication while a
    /// Gate lease or a campaign owns mining.
    pub(crate) fn service(&mut self, now_ms: u64, another_owner_mines: bool) {
        crate::device_pause::set_hidden(another_owner_mines);
        let streak_set = STREAK.load(Ordering::Acquire) != 0;
        if crate::device_pause::held().is_none() && !streak_set {
            return;
        }
        let intent = crate::runtime_snapshot::requested_mining_operator_intent();
        // The operator's resume clears the streak; any run ends the indication,
        // which means something only while paused.
        if self.resume_edge.observe(intent) && streak_set {
            self.clear_pending = true;
        }
        if intent == MiningOperatorIntent::Run && crate::device_pause::take().is_some() {
            log::info!("crash_loop_guard=pause_ended");
        }
        if !self.stable_cleared && boot_is_stable(now_ms) {
            self.clear_pending = true;
            self.stable_cleared = true;
        }
        if self.clear_pending && self.maybe_retry_at_ms.is_none_or(|at| now_ms >= at) {
            self.clear_pending = !clear_streak();
            self.maybe_retry_at_ms = self
                .clear_pending
                .then(|| now_ms.saturating_add(SAVE_RETRY_MS));
        }
    }

    /// The device paused itself on this tick: a resume before the next one is
    /// still seen.
    pub(crate) fn note_device_pause(&mut self) {
        self.resume_edge.note_device_pause();
    }
}

/// Saves a zero streak; returns whether it is saved.
fn clear_streak() -> bool {
    match crate::device_settings_store::persist(vec![streak_write(0)]) {
        Ok(()) => {
            STREAK.store(0, Ordering::Release);
            log::info!("crash_loop_guard=streak_cleared");
            true
        }
        Err(error) => {
            log::error!("crash_loop_guard=clear_failed error={error}");
            false
        }
    }
}
