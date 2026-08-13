use bitaxe_api::MiningOperatorIntentEffect;
use bitaxe_stratum::v1::state::MiningOperatorIntent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RequestedOperatorIntent(MiningOperatorIntent);

impl Default for RequestedOperatorIntent {
    fn default() -> Self {
        Self(MiningOperatorIntent::Paused)
    }
}

impl RequestedOperatorIntent {
    pub(crate) const fn current(self) -> MiningOperatorIntent {
        self.0
    }

    pub(crate) fn set(&mut self, intent: MiningOperatorIntent) {
        self.0 = intent;
    }

    pub(crate) fn apply_boot_preference(
        &mut self,
        start_mining_on_boot: bool,
    ) -> MiningOperatorIntent {
        let intent = if start_mining_on_boot {
            MiningOperatorIntent::Run
        } else {
            MiningOperatorIntent::Paused
        };
        self.set(intent);
        intent
    }

    pub(crate) fn apply(&mut self, effect: MiningOperatorIntentEffect) {
        self.set(effect.next_intent);
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::state::{MiningOperatorIntent, MiningRuntimeState};

    use super::*;

    #[test]
    fn pause_request_survives_an_interleaved_stale_session_publication() {
        // Arrange
        let mut requested = RequestedOperatorIntent::default();
        requested.set(MiningOperatorIntent::Run);
        let mut visible_session = MiningRuntimeState::default();
        visible_session.set_operator_intent(MiningOperatorIntent::Run);
        let stale_publication = visible_session.clone();

        // Act
        requested.apply(MiningOperatorIntentEffect {
            next_intent: MiningOperatorIntent::Paused,
        });
        visible_session = stale_publication;

        // Assert
        assert_eq!(requested.current(), MiningOperatorIntent::Paused);
        assert_eq!(visible_session.operator_intent, MiningOperatorIntent::Run);
    }

    #[test]
    fn the_latest_command_replaces_only_requested_intent() {
        // Arrange
        let mut requested = RequestedOperatorIntent::default();

        // Act
        requested.apply(MiningOperatorIntentEffect {
            next_intent: MiningOperatorIntent::Paused,
        });
        requested.apply(MiningOperatorIntentEffect {
            next_intent: MiningOperatorIntent::Run,
        });

        // Assert
        assert_eq!(requested.current(), MiningOperatorIntent::Run);
    }

    #[test]
    fn boot_preference_initializes_requested_intent() {
        // Arrange
        let mut requested = RequestedOperatorIntent::default();

        // Act
        let intent = requested.apply_boot_preference(true);

        // Assert
        assert_eq!(intent, MiningOperatorIntent::Run);
        assert_eq!(requested.current(), MiningOperatorIntent::Run);
    }
}
