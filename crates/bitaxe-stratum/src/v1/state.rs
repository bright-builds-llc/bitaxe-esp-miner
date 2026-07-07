use crate::v1::messages::PoolDifficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolLifecycleStatus {
    Disconnected,
    Connecting,
    Subscribed,
    Authorized,
    Active,
    Reconnecting,
    FallbackActive,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkSubmissionGate {
    Blocked,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningActivityStatus {
    Paused,
    Active,
    SafeBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HashrateInputs {
    pub hashes_done: u64,
    pub elapsed_ms: u64,
    pub rolling_hashrate_hs: f64,
}

impl Default for HashrateInputs {
    fn default() -> Self {
        Self {
            hashes_done: 0,
            elapsed_ms: 0,
            rolling_hashrate_hs: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShareDifficulty(f64);

impl ShareDifficulty {
    pub const fn new(difficulty: f64) -> Self {
        Self(difficulty)
    }

    pub const fn raw(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShareCounters {
    pub accepted: u64,
    pub rejected: u64,
    pub rejected_reasons: Vec<String>,
    pub maybe_best_difficulty: Option<ShareDifficulty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningRuntimeState {
    pub lifecycle: PoolLifecycleStatus,
    pub counters: ShareCounters,
    pub maybe_pool_difficulty: Option<PoolDifficulty>,
    pub fallback_active: bool,
    pub work_submission: WorkSubmissionGate,
    pub hashrate_inputs: HashrateInputs,
    pub mining_activity: MiningActivityStatus,
    pub maybe_blocked_reason: Option<&'static str>,
}

impl Default for MiningRuntimeState {
    fn default() -> Self {
        Self {
            lifecycle: PoolLifecycleStatus::Disconnected,
            counters: ShareCounters::default(),
            maybe_pool_difficulty: None,
            fallback_active: false,
            work_submission: WorkSubmissionGate::Blocked,
            hashrate_inputs: HashrateInputs::default(),
            mining_activity: MiningActivityStatus::Paused,
            maybe_blocked_reason: None,
        }
    }
}

impl MiningRuntimeState {
    pub fn record_accepted_share(&mut self, difficulty: ShareDifficulty) {
        self.counters.accepted += 1;
        let should_update_best = self
            .counters
            .maybe_best_difficulty
            .map(|best| difficulty.raw() > best.raw())
            .unwrap_or(true);
        if should_update_best {
            self.counters.maybe_best_difficulty = Some(difficulty);
        }
    }

    pub fn record_rejected_share(&mut self, reason: impl Into<String>) {
        self.counters.rejected += 1;
        self.counters.rejected_reasons.push(reason.into());
    }

    pub fn set_lifecycle(&mut self, lifecycle: PoolLifecycleStatus) {
        self.lifecycle = lifecycle;
    }

    pub fn set_pool_difficulty(&mut self, difficulty: PoolDifficulty) {
        self.maybe_pool_difficulty = Some(difficulty);
    }

    pub fn set_fallback_active(&mut self, active: bool) {
        self.fallback_active = active;
        if active {
            self.lifecycle = PoolLifecycleStatus::FallbackActive;
        }
    }

    pub fn record_hashrate_inputs(&mut self, inputs: HashrateInputs) {
        self.hashrate_inputs = inputs;
    }

    pub fn set_mining_activity(&mut self, activity: MiningActivityStatus) {
        self.mining_activity = activity;
    }

    pub fn block_work_submission(&mut self, reason: &'static str) {
        self.work_submission = WorkSubmissionGate::Blocked;
        self.mining_activity = MiningActivityStatus::SafeBlocked;
        self.maybe_blocked_reason = Some(reason);
    }

    pub fn clear_blocked_reason(&mut self) {
        self.maybe_blocked_reason = None;
    }

    pub fn allow_work_submission(&mut self) {
        self.work_submission = WorkSubmissionGate::Ready;
        self.clear_blocked_reason();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::messages::PoolDifficulty;

    #[test]
    fn runtime_state_defaults_to_disconnected_and_blocked() {
        // Arrange
        let state = MiningRuntimeState::default();

        // Act
        let lifecycle = state.lifecycle;

        // Assert
        assert_eq!(lifecycle, PoolLifecycleStatus::Disconnected);
        assert_eq!(state.counters.accepted, 0);
        assert_eq!(state.counters.rejected, 0);
        assert!(!state.fallback_active);
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
    }

    #[test]
    fn runtime_state_blocks_work_submission_with_exact_reason() {
        // Arrange
        let mut state = MiningRuntimeState::default();

        // Act
        state.block_work_submission("voltage_observation_stale");

        // Assert
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
        assert_eq!(state.mining_activity, MiningActivityStatus::SafeBlocked);
        assert_eq!(
            state.maybe_blocked_reason,
            Some("voltage_observation_stale")
        );
    }

    #[test]
    fn runtime_state_clears_blocked_reason_when_work_submission_is_allowed() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        state.block_work_submission("voltage_observation_stale");

        // Act
        state.allow_work_submission();

        // Assert
        assert_eq!(state.work_submission, WorkSubmissionGate::Ready);
        assert_eq!(state.maybe_blocked_reason, None);
    }

    #[test]
    fn runtime_state_records_accepted_share_and_best_difficulty() {
        // Arrange
        let mut state = MiningRuntimeState::default();

        // Act
        state.record_accepted_share(ShareDifficulty::new(42.0));

        // Assert
        assert_eq!(state.counters.accepted, 1);
        assert_eq!(
            state.counters.maybe_best_difficulty,
            Some(ShareDifficulty::new(42.0))
        );
    }

    #[test]
    fn runtime_state_records_rejected_share_reason() {
        // Arrange
        let mut state = MiningRuntimeState::default();

        // Act
        state.record_rejected_share("low difficulty");

        // Assert
        assert_eq!(state.counters.rejected, 1);
        assert_eq!(
            state.counters.rejected_reasons,
            vec!["low difficulty".to_owned()]
        );
    }

    #[test]
    fn runtime_state_sets_fallback_lifecycle_status() {
        // Arrange
        let mut state = MiningRuntimeState::default();

        // Act
        state.set_fallback_active(true);

        // Assert
        assert!(state.fallback_active);
        assert_eq!(state.lifecycle, PoolLifecycleStatus::FallbackActive);
    }

    #[test]
    fn runtime_state_records_hashrate_inputs_and_activity_status() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        let inputs = HashrateInputs {
            hashes_done: 1_000,
            elapsed_ms: 2_000,
            rolling_hashrate_hs: 500.0,
        };

        // Act
        state.record_hashrate_inputs(inputs);
        state.set_mining_activity(MiningActivityStatus::Active);
        let active = state.mining_activity;
        state.set_mining_activity(MiningActivityStatus::SafeBlocked);
        let safe_blocked = state.mining_activity;
        state.set_mining_activity(MiningActivityStatus::Paused);

        // Assert
        assert_eq!(state.hashrate_inputs, inputs);
        assert_eq!(active, MiningActivityStatus::Active);
        assert_eq!(safe_blocked, MiningActivityStatus::SafeBlocked);
        assert_eq!(state.mining_activity, MiningActivityStatus::Paused);
    }

    #[test]
    fn runtime_state_records_pool_difficulty_and_work_submission_gate() {
        // Arrange
        let mut state = MiningRuntimeState::default();
        let difficulty = PoolDifficulty { difficulty: 1638.0 };

        // Act
        state.set_pool_difficulty(difficulty);
        state.allow_work_submission();

        // Assert
        assert_eq!(state.maybe_pool_difficulty, Some(difficulty));
        assert_eq!(state.work_submission, WorkSubmissionGate::Ready);
    }
}
