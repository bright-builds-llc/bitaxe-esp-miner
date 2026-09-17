use super::*;

/// Actual decoded ACK fields. Settlement retains evidence, never work authority.
#[derive(Clone, Copy)]
pub struct ObservedAcknowledgement {
    pub channel_id: u32,
    pub last_sequence: u32,
    pub accepted_count: u32,
    pub shares_sum: u64,
}

impl V2Record {
    /// Joins a received ACK to one actual completed write, independently of the
    /// ordinary work runtime's later retirement or delivery of its inbox event.
    pub fn observe_acknowledgement(
        &mut self,
        ack: ObservedAcknowledgement,
        digest: String,
        now: Option<u64>,
    ) -> bool {
        let time = self.time(now);
        let pending = self
            .record
            .share_facts
            .iter()
            .filter(|f| {
                f.submission_sequence <= ack.last_sequence && f.maybe_ack_last_sequence.is_none()
            })
            .count();
        let valid = self.record.scope == Scope::Share
            && !self.record.resources.socket_closed
            && !self.record.resources.worker_quiescent
            && self.record.maybe_outcome.is_none()
            && ack.accepted_count == 1
            && ack.shares_sum == 1024
            && pending == 1;
        let Some(fact) = self.share_mut(ack.last_sequence) else {
            self.fail(Stage::Accepted, FailureCategory::Evidence, time);
            return false;
        };
        if !valid
            || fact.channel_id != ack.channel_id
            || fact.maybe_write_completed_at_device_us.is_none()
            || fact.maybe_ack_last_sequence.is_some()
        {
            self.fail(Stage::Accepted, FailureCategory::Evidence, time);
            return false;
        }
        fact.maybe_ack_at_device_us = time;
        fact.maybe_ack_last_sequence = Some(ack.last_sequence);
        fact.maybe_ack_accepted_count = Some(ack.accepted_count);
        fact.maybe_ack_shares_sum = Some(ack.shares_sum);
        fact.maybe_matched_submit_count = Some(1);
        let job = fact.job_id;
        self.event(
            Stage::Accepted,
            time,
            Some(ack.channel_id),
            Some(job),
            Some(ack.last_sequence),
            Some(digest),
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prepared() -> V2Record {
        let observation = CurrentObservation {
            boot_ordinal: 2,
            worker_generation: 7,
            serial_transport_epoch: 4,
            maybe_observed_at_us: Some(1000),
            clock_valid: true,
            maybe_station_ipv4: None,
            wifi_connected: true,
            maybe_socket: None,
        };
        let mut r =
            V2Record::admit(Scope::Share, "test".into(), &observation, None).expect("record");
        assert!(r.budget_armed(2, 180_000, Some(2000)));
        assert!(r.add_share(ShareFact {
            dispatch_sequence: 1,
            asic_job_id: 0,
            work_fields_sha256: "0".repeat(64),
            dispatched_at_device_us: 2100,
            nonce_at_device_us: 2200,
            maybe_write_started_at_device_us: Some(2300),
            maybe_write_completed_at_device_us: Some(2400),
            nonce: 1,
            version_bits: 0,
            asic_index: 0,
            core_id: 0,
            small_core_id: 0,
            channel_id: 9,
            job_id: 7,
            submission_sequence: 0,
            ntime: 1000,
            version: 0x20000000,
            maybe_ack_at_device_us: None,
            maybe_ack_last_sequence: None,
            maybe_ack_accepted_count: None,
            maybe_ack_shares_sum: None,
            maybe_matched_submit_count: None
        }));
        r
    }
    fn ack() -> ObservedAcknowledgement {
        ObservedAcknowledgement {
            channel_id: 9,
            last_sequence: 0,
            accepted_count: 1,
            shares_sum: 1024,
        }
    }
    #[test]
    fn actual_ack_survives_revocation_and_owner_retirement_without_authority_change() {
        // Arrange
        let mut r = prepared();
        r.event(Stage::Revoked, Some(2500), None, None, None, None);
        r.event(Stage::Shutdown, Some(2501), None, None, None, None);
        let binding = r.binding();
        let deadline = r.snapshot().maybe_authority_deadline_device_us;
        // Act: actual receive settles before a delayed ordinary inbox is retired.
        assert!(r.observe_acknowledgement(ack(), "1".repeat(64), Some(2600)));
        r.socket_closed(Some(2700));
        r.joined(Some(2800), false);
        // Assert
        let result = r.snapshot();
        assert_eq!(r.binding(), binding);
        assert_eq!(result.maybe_authority_deadline_device_us, deadline);
        assert_eq!(result.share_facts[0].maybe_ack_at_device_us, Some(2600));
        assert_eq!(
            result
                .events
                .iter()
                .filter(|e| e.kind == Stage::Accepted)
                .count(),
            1
        );
        assert!(result.maybe_first_failure.is_none());
        assert!(result.resources.fence_retained);
        assert!(result.maybe_outcome.is_none());
    }
    #[test]
    fn repeated_ack_never_gains_second_credit() {
        // Arrange
        let mut r = prepared();
        assert!(r.observe_acknowledgement(ack(), "1".repeat(64), Some(2600)));
        // Act
        assert!(!r.observe_acknowledgement(ack(), "1".repeat(64), Some(2601)));
        // Assert
        let result = r.snapshot();
        assert_eq!(result.share_facts[0].maybe_ack_at_device_us, Some(2600));
        assert_eq!(
            result
                .events
                .iter()
                .filter(|e| e.kind == Stage::Accepted)
                .count(),
            1
        );
        assert_eq!(
            result.maybe_first_failure.expect("duplicate").category,
            FailureCategory::Evidence
        );
    }
    #[test]
    fn incomplete_write_cannot_be_settled_by_received_ack() {
        // Arrange
        let mut r = prepared();
        r.share_mut(0)
            .expect("fact")
            .maybe_write_completed_at_device_us = None;
        // Act
        assert!(!r.observe_acknowledgement(ack(), "1".repeat(64), Some(2600)));
        // Assert
        assert!(r.snapshot().share_facts[0]
            .maybe_ack_last_sequence
            .is_none());
    }
    #[test]
    fn later_ack_cannot_skip_an_earlier_unsettled_submission() {
        // Arrange
        let mut r = prepared();
        let mut second = r.snapshot().share_facts[0].clone();
        second.submission_sequence = 1;
        assert!(r.add_share(second));
        // Act
        assert!(!r.observe_acknowledgement(
            ObservedAcknowledgement {
                last_sequence: 1,
                ..ack()
            },
            "1".repeat(64),
            Some(2600)
        ));
        // Assert
        assert!(r
            .snapshot()
            .share_facts
            .iter()
            .all(|f| f.maybe_ack_last_sequence.is_none()));
    }
    #[test]
    fn ack_must_match_channel_count_and_actual_difficulty_sum() {
        // Arrange
        for invalid in [
            ObservedAcknowledgement {
                channel_id: 8,
                ..ack()
            },
            ObservedAcknowledgement {
                accepted_count: 2,
                ..ack()
            },
            ObservedAcknowledgement {
                shares_sum: 1,
                ..ack()
            },
        ] {
            let mut r = prepared();
            // Act / Assert
            assert!(!r.observe_acknowledgement(invalid, "1".repeat(64), Some(2600)));
            assert!(r.snapshot().share_facts[0]
                .maybe_ack_last_sequence
                .is_none());
        }
    }
}
