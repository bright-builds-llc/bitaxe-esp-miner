//! Cooperative progress boundaries for the production-owner feedback loop.

use std::collections::VecDeque;

use bitaxe_core::runtime_health::TaskWatchdogOwnerSubphase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerProgressBoundary {
    EventStarted,
    EventHandled,
    EffectStarted,
    EffectHeartbeat(TaskWatchdogOwnerSubphase),
    EffectCompleted,
}

pub(crate) fn drive_feedback<Event, Effect, HandleError>(
    initial_event: Event,
    mut handle: impl FnMut(Event) -> Result<Vec<Effect>, HandleError>,
    mut execute: impl FnMut(Effect, &mut dyn FnMut(TaskWatchdogOwnerSubphase)) -> Option<Event>,
    mut progress: impl FnMut(OwnerProgressBoundary, Option<&Effect>),
) -> Result<(), HandleError> {
    let mut events = VecDeque::from([initial_event]);
    while let Some(event) = events.pop_front() {
        progress(OwnerProgressBoundary::EventStarted, None);
        let effects = handle(event)?;
        progress(OwnerProgressBoundary::EventHandled, None);
        for effect in effects {
            progress(OwnerProgressBoundary::EffectStarted, Some(&effect));
            let mut heartbeat =
                |subphase| progress(OwnerProgressBoundary::EffectHeartbeat(subphase), None);
            let maybe_feedback = execute(effect, &mut heartbeat);
            progress(OwnerProgressBoundary::EffectCompleted, None);
            if let Some(feedback) = maybe_feedback {
                events.push_back(feedback);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn feedback_cascade_reports_each_completed_event_and_effect() {
        // Arrange
        let mut progress = Vec::new();

        // Act
        drive_feedback(
            0_u8,
            |event| Ok::<Vec<u8>, &'static str>(if event == 0 { vec![10, 11] } else { vec![] }),
            |effect, _| (effect == 10).then_some(1),
            |boundary, _| progress.push(boundary),
        )
        .expect("feedback cascade should complete");

        // Assert
        assert_eq!(
            progress,
            [
                OwnerProgressBoundary::EventStarted,
                OwnerProgressBoundary::EventHandled,
                OwnerProgressBoundary::EffectStarted,
                OwnerProgressBoundary::EffectCompleted,
                OwnerProgressBoundary::EffectStarted,
                OwnerProgressBoundary::EffectCompleted,
                OwnerProgressBoundary::EventStarted,
                OwnerProgressBoundary::EventHandled,
            ]
        );
    }

    #[test]
    fn effect_progress_is_reported_only_after_execution_returns() {
        // Arrange
        let effect_returned = Cell::new(false);
        let effect_progress_observed = Cell::new(false);

        // Act
        drive_feedback(
            (),
            |_| Ok::<Vec<()>, &'static str>(vec![()]),
            |_, _| {
                effect_returned.set(true);
                None
            },
            |boundary, _| {
                if boundary == OwnerProgressBoundary::EffectCompleted {
                    effect_progress_observed.set(effect_returned.get());
                }
            },
        )
        .expect("single effect should complete");

        // Assert
        assert!(effect_progress_observed.get());
    }

    #[test]
    fn entry_progress_precedes_handler_and_effect_execution() {
        // Arrange
        let event_entry_observed = Cell::new(false);
        let effect_entry_observed = Cell::new(false);

        // Act
        drive_feedback(
            (),
            |_| {
                assert!(event_entry_observed.get());
                Ok::<Vec<()>, &'static str>(vec![()])
            },
            |_, _| {
                assert!(effect_entry_observed.get());
                None
            },
            |boundary, _| match boundary {
                OwnerProgressBoundary::EventStarted => event_entry_observed.set(true),
                OwnerProgressBoundary::EffectStarted => effect_entry_observed.set(true),
                OwnerProgressBoundary::EventHandled
                | OwnerProgressBoundary::EffectHeartbeat(_)
                | OwnerProgressBoundary::EffectCompleted => {}
            },
        )
        .expect("entry boundaries should complete");

        // Assert
        assert!(event_entry_observed.get());
        assert!(effect_entry_observed.get());
    }

    #[test]
    fn entry_progress_resets_inherited_age_but_blocking_work_can_still_become_stale() {
        // Arrange
        const TIMEOUT_MS: u64 = 5_000;
        let now_ms = Cell::new(4_999_u64);
        let last_feed_ms = Cell::new(0_u64);
        let blocking_effect_became_stale = Cell::new(false);

        // Act
        drive_feedback(
            (),
            |_| {
                now_ms.set(now_ms.get().saturating_add(2));
                assert!(now_ms.get().saturating_sub(last_feed_ms.get()) < TIMEOUT_MS);
                Ok::<Vec<()>, &'static str>(vec![()])
            },
            |_, heartbeat| {
                assert_eq!(last_feed_ms.get(), now_ms.get());
                heartbeat(TaskWatchdogOwnerSubphase::Unavailable);
                now_ms.set(now_ms.get().saturating_add(TIMEOUT_MS + 1));
                blocking_effect_became_stale
                    .set(now_ms.get().saturating_sub(last_feed_ms.get()) > TIMEOUT_MS);
                None
            },
            |_, _| last_feed_ms.set(now_ms.get()),
        )
        .expect("production-shaped feedback should complete");

        // Assert
        assert!(blocking_effect_became_stale.get());
    }

    #[test]
    fn failed_event_reports_no_completed_progress() {
        // Arrange
        let mut progress = Vec::new();

        // Act
        let result = drive_feedback(
            (),
            |_| Err::<Vec<()>, _>("event_failed"),
            |_, _| None,
            |boundary, _| progress.push(boundary),
        );

        // Assert
        assert_eq!(result, Err("event_failed"));
        assert_eq!(progress, [OwnerProgressBoundary::EventStarted]);
    }

    #[test]
    fn effect_heartbeats_are_reported_before_completion() {
        // Arrange
        let mut progress = Vec::new();

        // Act
        drive_feedback(
            (),
            |_| Ok::<Vec<()>, &'static str>(vec![()]),
            |_, heartbeat| {
                heartbeat(TaskWatchdogOwnerSubphase::SafeStopStopDispatch);
                heartbeat(TaskWatchdogOwnerSubphase::SafeStopAssertControlLineLow);
                None
            },
            |boundary, _| progress.push(boundary),
        )
        .expect("heartbeat effect should complete");

        // Assert
        assert_eq!(
            progress,
            [
                OwnerProgressBoundary::EventStarted,
                OwnerProgressBoundary::EventHandled,
                OwnerProgressBoundary::EffectStarted,
                OwnerProgressBoundary::EffectHeartbeat(
                    TaskWatchdogOwnerSubphase::SafeStopStopDispatch,
                ),
                OwnerProgressBoundary::EffectHeartbeat(
                    TaskWatchdogOwnerSubphase::SafeStopAssertControlLineLow,
                ),
                OwnerProgressBoundary::EffectCompleted,
            ]
        );
    }
}
