//! Cooperative progress boundaries for the production-owner feedback loop.

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerProgressBoundary {
    EventHandled,
    EffectCompleted,
}

pub(crate) fn drive_feedback<Event, Effect, HandleError>(
    initial_event: Event,
    mut handle: impl FnMut(Event) -> Result<Vec<Effect>, HandleError>,
    mut execute: impl FnMut(Effect) -> Option<Event>,
    mut progress: impl FnMut(OwnerProgressBoundary),
) -> Result<(), HandleError> {
    let mut events = VecDeque::from([initial_event]);
    while let Some(event) = events.pop_front() {
        let effects = handle(event)?;
        progress(OwnerProgressBoundary::EventHandled);
        for effect in effects {
            let maybe_feedback = execute(effect);
            progress(OwnerProgressBoundary::EffectCompleted);
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
            |effect| (effect == 10).then_some(1),
            |boundary| progress.push(boundary),
        )
        .expect("feedback cascade should complete");

        // Assert
        assert_eq!(
            progress,
            [
                OwnerProgressBoundary::EventHandled,
                OwnerProgressBoundary::EffectCompleted,
                OwnerProgressBoundary::EffectCompleted,
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
            |_| {
                effect_returned.set(true);
                None
            },
            |boundary| {
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
    fn failed_event_reports_no_completed_progress() {
        // Arrange
        let mut progress = Vec::new();

        // Act
        let result = drive_feedback(
            (),
            |_| Err::<Vec<()>, _>("event_failed"),
            |_| None,
            |boundary| progress.push(boundary),
        );

        // Assert
        assert_eq!(result, Err("event_failed"));
        assert!(progress.is_empty());
    }
}
