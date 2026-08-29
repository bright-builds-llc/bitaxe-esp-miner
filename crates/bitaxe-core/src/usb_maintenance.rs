//! Pure maintenance-handoff state for the single native USB PHY.

const HANDOFF_WINDOW_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceEvent {
    LineCoding { bit_rate: u32 },
    LineState { dtr: bool, rts: bool },
    SafeStopComplete,
    SafeStopFailed,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceAction {
    None,
    RequestSafeStop,
    EmitReady,
    CommitRestart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaintenancePhase {
    Idle,
    DtrAsserted,
    SafeStopPending,
    Ready,
    Committed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbMaintenanceState {
    phase: MaintenancePhase,
    deadline_ms: Option<u64>,
}

impl Default for UsbMaintenanceState {
    fn default() -> Self {
        Self {
            phase: MaintenancePhase::Idle,
            deadline_ms: None,
        }
    }
}

impl UsbMaintenanceState {
    pub fn observe(&mut self, event: MaintenanceEvent, now_ms: u64) -> MaintenanceAction {
        if self.deadline_ms.is_some_and(|deadline| now_ms >= deadline) {
            self.disarm();
        }
        match (self.phase, event) {
            (MaintenancePhase::Idle, MaintenanceEvent::LineState { dtr: true, .. }) => {
                self.phase = MaintenancePhase::DtrAsserted;
                self.deadline_ms = now_ms.checked_add(HANDOFF_WINDOW_MS);
                MaintenanceAction::None
            }
            (MaintenancePhase::DtrAsserted, MaintenanceEvent::LineCoding { bit_rate: 1_200 }) => {
                self.phase = MaintenancePhase::SafeStopPending;
                MaintenanceAction::RequestSafeStop
            }
            (MaintenancePhase::SafeStopPending, MaintenanceEvent::SafeStopComplete) => {
                self.phase = MaintenancePhase::Ready;
                MaintenanceAction::EmitReady
            }
            (MaintenancePhase::Ready, MaintenanceEvent::LineState { dtr: false, .. }) => {
                self.phase = MaintenancePhase::Committed;
                self.deadline_ms = None;
                MaintenanceAction::CommitRestart
            }
            (_, MaintenanceEvent::Detached | MaintenanceEvent::SafeStopFailed) => {
                self.disarm();
                MaintenanceAction::None
            }
            (_, MaintenanceEvent::LineCoding { .. } | MaintenanceEvent::LineState { .. }) => {
                self.disarm();
                MaintenanceAction::None
            }
            (MaintenancePhase::Committed, _) => MaintenanceAction::None,
            _ => {
                self.disarm();
                MaintenanceAction::None
            }
        }
    }

    pub fn expire(&mut self, now_ms: u64) {
        if self.deadline_ms.is_some_and(|deadline| now_ms >= deadline) {
            self.disarm();
        }
    }

    fn disarm(&mut self) {
        self.phase = MaintenancePhase::Idle;
        self.deadline_ms = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dtr(dtr: bool) -> MaintenanceEvent {
        MaintenanceEvent::LineState { dtr, rts: false }
    }

    #[test]
    fn exact_control_sequence_acknowledges_one_committed_restart() {
        // Arrange
        let mut state = UsbMaintenanceState::default();

        // Act / Assert
        assert_eq!(state.observe(dtr(true), 10), MaintenanceAction::None);
        assert_eq!(
            state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 11),
            MaintenanceAction::RequestSafeStop
        );
        assert_eq!(
            state.observe(MaintenanceEvent::SafeStopComplete, 12),
            MaintenanceAction::EmitReady
        );
        assert_eq!(
            state.observe(dtr(false), 13),
            MaintenanceAction::CommitRestart
        );
        assert_eq!(state.observe(dtr(false), 14), MaintenanceAction::None);
    }

    #[test]
    fn wrong_order_disarms_without_restart() {
        // Arrange
        let mut state = UsbMaintenanceState::default();

        // Act
        let action = state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 1);

        // Assert
        assert_eq!(action, MaintenanceAction::None);
        assert_eq!(state.observe(dtr(false), 2), MaintenanceAction::None);
    }

    #[test]
    fn duplicate_ready_event_disarms_without_restart() {
        // Arrange
        let mut state = UsbMaintenanceState::default();
        let _action = state.observe(dtr(true), 1);
        let _action = state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 2);
        let _action = state.observe(MaintenanceEvent::SafeStopComplete, 3);

        // Act
        let duplicate = state.observe(MaintenanceEvent::SafeStopComplete, 4);

        // Assert
        assert_eq!(duplicate, MaintenanceAction::None);
        assert_eq!(state.observe(dtr(false), 5), MaintenanceAction::None);
    }

    #[test]
    fn detach_before_commit_disarms_without_restart() {
        // Arrange
        let mut state = UsbMaintenanceState::default();
        let _action = state.observe(dtr(true), 1);
        let _action = state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 2);
        let _action = state.observe(MaintenanceEvent::SafeStopComplete, 3);

        // Act
        let detached = state.observe(MaintenanceEvent::Detached, 4);

        // Assert
        assert_eq!(detached, MaintenanceAction::None);
        assert_eq!(state.observe(dtr(false), 5), MaintenanceAction::None);
    }

    #[test]
    fn accepted_commit_stays_single_after_later_detach() {
        // Arrange
        let mut state = UsbMaintenanceState::default();
        let _action = state.observe(dtr(true), 1);
        let _action = state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 2);
        let _action = state.observe(MaintenanceEvent::SafeStopComplete, 3);

        // Act
        let committed = state.observe(dtr(false), 4);
        let detached = state.observe(MaintenanceEvent::Detached, 5);

        // Assert
        assert_eq!(committed, MaintenanceAction::CommitRestart);
        assert_eq!(detached, MaintenanceAction::None);
        assert_eq!(state.observe(dtr(false), 6), MaintenanceAction::None);
    }

    #[test]
    fn failed_safe_stop_disarms_without_restart() {
        // Arrange
        let mut state = UsbMaintenanceState::default();
        let _action = state.observe(dtr(true), 1);
        let _action = state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 2);

        // Act
        let rejected = state.observe(MaintenanceEvent::SafeStopFailed, 3);

        // Assert
        assert_eq!(rejected, MaintenanceAction::None);
        assert_eq!(state.observe(dtr(false), 4), MaintenanceAction::None);
    }

    #[test]
    fn disconnect_and_exact_deadline_disarm_without_restart() {
        for event in [MaintenanceEvent::Detached, dtr(false)] {
            // Arrange
            let mut state = UsbMaintenanceState::default();
            let _action = state.observe(dtr(true), 0);

            // Act
            let action = state.observe(event, HANDOFF_WINDOW_MS);

            // Assert
            assert_eq!(action, MaintenanceAction::None);
        }
    }
}
