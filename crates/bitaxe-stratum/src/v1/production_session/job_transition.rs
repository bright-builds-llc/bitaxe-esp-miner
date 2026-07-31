use crate::v1::production_work::PoolSessionGeneration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JobTransitionState {
    #[default]
    NotObserved,
    ReplacementQueued,
    ReplacementDispatched,
    ReplacementResultCorrelated,
}

impl JobTransitionState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotObserved => "not_observed",
            Self::ReplacementQueued => "replacement_queued",
            Self::ReplacementDispatched => "replacement_dispatched",
            Self::ReplacementResultCorrelated => "replacement_result_correlated",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JobTransitionEvidence {
    pub pool_notify_count: u64,
    pub clean_jobs_notify_count: u64,
    pub previous_block_change_count: u64,
    pub new_block_generation_count: u64,
    pub replacement_dispatch_count: u64,
    pub replacement_result_count: u64,
    pub completed_transition_count: u64,
    pub stale_generation_result_discard_count: u64,
    pub stale_generation_submit_count: u64,
    pub reconnect_count: u64,
    pub latest_state: JobTransitionState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct JobTransitionTracker {
    evidence: JobTransitionEvidence,
    maybe_transition_generation: Option<PoolSessionGeneration>,
    current_transition_completed: bool,
}

impl JobTransitionTracker {
    pub(super) const fn evidence(&self) -> JobTransitionEvidence {
        self.evidence
    }

    pub(super) fn note_notify(
        &mut self,
        clean_jobs: bool,
        previous_block_changed: bool,
        generation_advanced: bool,
        generation: PoolSessionGeneration,
    ) {
        self.evidence.pool_notify_count = self.evidence.pool_notify_count.saturating_add(1);
        if clean_jobs {
            self.evidence.clean_jobs_notify_count =
                self.evidence.clean_jobs_notify_count.saturating_add(1);
        }
        if !previous_block_changed {
            if generation_advanced
                && self.maybe_transition_generation.is_some()
                && !self.current_transition_completed
            {
                self.maybe_transition_generation = Some(generation);
                self.evidence.latest_state = JobTransitionState::ReplacementQueued;
            }
            return;
        }

        self.evidence.previous_block_change_count =
            self.evidence.previous_block_change_count.saturating_add(1);
        if generation_advanced {
            self.evidence.new_block_generation_count =
                self.evidence.new_block_generation_count.saturating_add(1);
        }
        self.evidence.latest_state = JobTransitionState::ReplacementQueued;
        self.maybe_transition_generation = Some(generation);
        self.current_transition_completed = false;
    }

    pub(super) fn note_dispatch(&mut self, generation: PoolSessionGeneration) {
        if self.maybe_transition_generation != Some(generation) {
            return;
        }
        self.evidence.replacement_dispatch_count =
            self.evidence.replacement_dispatch_count.saturating_add(1);
        if !self.current_transition_completed {
            self.evidence.latest_state = JobTransitionState::ReplacementDispatched;
        }
    }

    pub(super) fn note_correlated_result(&mut self, generation: PoolSessionGeneration) {
        if self.maybe_transition_generation != Some(generation) {
            return;
        }
        self.evidence.replacement_result_count =
            self.evidence.replacement_result_count.saturating_add(1);
        self.evidence.latest_state = JobTransitionState::ReplacementResultCorrelated;
        if !self.current_transition_completed {
            self.evidence.completed_transition_count =
                self.evidence.completed_transition_count.saturating_add(1);
            self.current_transition_completed = true;
        }
    }

    pub(super) fn note_stale_generation_result(&mut self) {
        self.evidence.stale_generation_result_discard_count = self
            .evidence
            .stale_generation_result_discard_count
            .saturating_add(1);
    }

    pub(super) fn note_reconnect(&mut self) {
        self.evidence.reconnect_count = self.evidence.reconnect_count.saturating_add(1);
        self.maybe_transition_generation = None;
        self.current_transition_completed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_block_chain_completes_only_after_replacement_result() {
        // Arrange
        let mut tracker = JobTransitionTracker::default();
        let generation = PoolSessionGeneration::initial().next();

        // Act
        tracker.note_notify(true, true, true, generation);
        tracker.note_dispatch(generation);
        tracker.note_correlated_result(generation);

        // Assert
        let evidence = tracker.evidence();
        assert_eq!(evidence.completed_transition_count, 1);
        assert_eq!(
            evidence.latest_state,
            JobTransitionState::ReplacementResultCorrelated
        );
    }

    #[test]
    fn same_block_refresh_never_starts_a_new_block_transition() {
        // Arrange
        let mut tracker = JobTransitionTracker::default();

        // Act
        tracker.note_notify(false, false, false, PoolSessionGeneration::initial());

        // Assert
        let evidence = tracker.evidence();
        assert_eq!(evidence.pool_notify_count, 1);
        assert_eq!(evidence.completed_transition_count, 0);
        assert_eq!(evidence.latest_state, JobTransitionState::NotObserved);
    }

    #[test]
    fn same_block_generation_refresh_preserves_an_inflight_transition() {
        // Arrange
        let mut tracker = JobTransitionTracker::default();
        let new_block_generation = PoolSessionGeneration::initial().next();
        let refreshed_generation = new_block_generation.next();
        tracker.note_notify(true, true, true, new_block_generation);
        tracker.note_dispatch(new_block_generation);

        // Act
        tracker.note_notify(true, false, true, refreshed_generation);
        tracker.note_dispatch(refreshed_generation);
        tracker.note_correlated_result(refreshed_generation);

        // Assert
        let evidence = tracker.evidence();
        assert_eq!(evidence.previous_block_change_count, 1);
        assert_eq!(evidence.new_block_generation_count, 1);
        assert_eq!(evidence.replacement_dispatch_count, 2);
        assert_eq!(evidence.replacement_result_count, 1);
        assert_eq!(evidence.completed_transition_count, 1);
        assert_eq!(
            evidence.latest_state,
            JobTransitionState::ReplacementResultCorrelated
        );
    }

    #[test]
    fn multiple_transitions_and_reconnects_remain_closed_aggregate_evidence() {
        // Arrange
        let mut tracker = JobTransitionTracker::default();
        let first = PoolSessionGeneration::initial().next();
        let second = first.next();

        // Act
        tracker.note_notify(true, true, true, first);
        tracker.note_dispatch(first);
        tracker.note_correlated_result(first);
        tracker.note_reconnect();
        tracker.note_notify(true, true, true, second);
        tracker.note_dispatch(second);
        tracker.note_stale_generation_result();
        tracker.note_correlated_result(second);

        // Assert
        let evidence = tracker.evidence();
        assert_eq!(evidence.completed_transition_count, 2);
        assert_eq!(evidence.replacement_dispatch_count, 2);
        assert_eq!(evidence.replacement_result_count, 2);
        assert_eq!(evidence.stale_generation_result_discard_count, 1);
        assert_eq!(evidence.stale_generation_submit_count, 0);
        assert_eq!(evidence.reconnect_count, 1);
    }

    #[test]
    fn reconnect_cannot_complete_a_pre_reconnect_transition() {
        // Arrange
        let mut tracker = JobTransitionTracker::default();
        let generation = PoolSessionGeneration::initial().next();
        tracker.note_notify(true, true, true, generation);
        tracker.note_dispatch(generation);

        // Act
        tracker.note_reconnect();
        tracker.note_correlated_result(generation);

        // Assert
        let evidence = tracker.evidence();
        assert_eq!(evidence.completed_transition_count, 0);
        assert_eq!(evidence.replacement_result_count, 0);
        assert_eq!(evidence.reconnect_count, 1);
    }
}
