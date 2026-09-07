use super::*;
fn receipt(sequence: u32, step: u32, outcome: Outcome) -> Receipt {
    Receipt {
        source_hash: 0x1234,
        boot_ordinal: 7,
        generation: 3,
        sequence,
        uptime_ms: 1_000 + u64::from(sequence),
        last_completed_step: if outcome == Outcome::Completed {
            step
        } else {
            step - 1
        },
        current_step: step,
        outcome,
        failure: if outcome == Outcome::Failed {
            Failure::AsicFailed
        } else {
            Failure::None
        },
        maybe_heap_free: Some(10_000),
        maybe_heap_largest: Some(5_000),
        maybe_stack_free: Some(1_000),
    }
}
#[test]
fn interruption_at_every_word_preserves_only_verified_progress() {
    for step in 1..=9 {
        // Arrange
        let prior = receipt(step * 2 - 1, step, Outcome::Started);
        let next = receipt(step * 2, step, Outcome::Completed);
        let prior_words = prior.encode().expect("valid prior");
        let next_words = next.encode().expect("valid next");
        for cut in 0..=WORDS + 1 {
            let mut slots = [prior_words, [0; WORDS]];
            let mut writes = 0;
            // Act: simulate native abort without invoking any Rust panic hook.
            commit_words(next_words, |index, value| {
                if writes < cut {
                    slots[1][index] = value;
                }
                writes += 1;
            });
            let snapshot = recover(&slots, prior.source_hash);
            // Assert
            assert_eq!(snapshot.status, Status::Valid);
            assert_eq!(
                snapshot.maybe_receipt,
                Some(if cut == WORDS + 1 { next } else { prior })
            );
            assert_eq!(snapshot.interrupted, cut > 0 && cut < WORDS + 1);
        }
    }
}
#[test]
fn every_payload_bit_corruption_is_rejected() {
    // Arrange
    let valid = receipt(1, 1, Outcome::Started)
        .encode()
        .expect("valid receipt");
    for word in 0..WORDS {
        for bit in 0..32 {
            let mut corrupt = valid;
            corrupt[word] ^= 1 << bit;
            // Act
            let snapshot = recover(&[corrupt, [0; WORDS]], 0x1234);
            // Assert
            assert_ne!(snapshot.status, Status::Valid);
            assert!(snapshot.maybe_receipt.is_none());
        }
    }
}
#[test]
fn wrong_firmware_has_no_payload() {
    // Arrange
    let slot = receipt(1, 1, Outcome::Started)
        .encode()
        .expect("valid receipt");
    // Act
    let snapshot = recover(&[slot, [0; WORDS]], 0x9876);
    // Assert
    assert_eq!(snapshot.status, Status::WrongFirmware);
    assert!(snapshot.maybe_receipt.is_none());
    assert_eq!(snapshot.marker(true),"worker_preparation_receipt schema=v1 origin=previous_boot status=wrong_firmware redacted=true");
}
#[test]
fn invalid_generation_or_progress_cannot_be_encoded() {
    for mutate in [
        |r: &mut Receipt| r.generation = 0,
        |r: &mut Receipt| r.sequence = 0,
        |r: &mut Receipt| r.current_step = 10,
        |r: &mut Receipt| r.last_completed_step = 1,
    ] {
        // Arrange
        let mut value = receipt(1, 1, Outcome::Started);
        mutate(&mut value);
        // Act / Assert
        assert!(value.encode().is_none());
    }
}
#[test]
fn previous_snapshot_survives_current_boot_rewrites() {
    // Arrange
    let old = receipt(3, 2, Outcome::Started);
    let mut slots = [old.encode().expect("old"), [0; WORDS]];
    let previous = recover(&slots, old.source_hash).for_boot(7);
    // Act
    for sequence in 1..=6 {
        let mut next = receipt(sequence, 1, Outcome::Started);
        next.boot_ordinal = 8;
        commit_words(next.encode().expect("new"), |index, value| {
            slots[(sequence % 2) as usize][index] = value
        });
    }
    // Assert
    assert_eq!(previous.maybe_receipt, Some(old));
    assert_eq!(
        recover(&slots, old.source_hash)
            .maybe_receipt
            .expect("current")
            .boot_ordinal,
        8
    );
    assert_eq!(previous.for_boot(8).status, Status::Unavailable);
}
#[test]
fn incomplete_and_corrupt_without_valid_slot_have_no_payload() {
    // Arrange
    let mut incomplete = [0; WORDS];
    incomplete[1] = 1;
    let mut corrupt = incomplete;
    corrupt[0] = 123;
    // Act / Assert
    assert_eq!(
        recover(&[incomplete, [0; WORDS]], 0x1234).status,
        Status::Incomplete
    );
    assert_eq!(
        recover(&[corrupt, [0; WORDS]], 0x1234).status,
        Status::Corrupt
    );
    assert_eq!(
        recover(&[[0; WORDS]; 2], 0x1234).status,
        Status::Unavailable
    );
}
#[test]
fn normal_failure_retains_current_step_and_prior_completion() {
    // Arrange
    let failed = receipt(7, 4, Outcome::Failed);
    // Act
    let snapshot = recover(
        &[failed.encode().expect("failed receipt"), [0; WORDS]],
        0x1234,
    );
    // Assert
    assert_eq!(snapshot.maybe_receipt, Some(failed));
    assert!(snapshot
        .marker(false)
        .contains("last_completed_step=3 current_step=4 outcome=failed"));
}

#[test]
fn read_commit_word_twice_prevents_accepting_uncommitted_complete_payload() {
    // Arrange: old validity was read, then the writer replaced all fields but has not committed.
    let complete = receipt(2, 1, Outcome::Completed)
        .encode()
        .expect("valid candidate");
    let mut header_reads = 0;
    // Act
    let observed = read_slot(|index| {
        if index != 0 {
            return complete[index];
        }
        header_reads += 1;
        if header_reads == 1 {
            VALID
        } else {
            0
        }
    });
    // Assert
    assert_eq!(
        recover(&[observed, [0; WORDS]], 0x1234).status,
        Status::Incomplete
    );
}

#[test]
fn interruption_while_reusing_older_slot_cannot_publish_new_step() {
    for cut in 1..WORDS + 1 {
        // Arrange
        let older = receipt(1, 1, Outcome::Started);
        let latest = receipt(2, 1, Outcome::Completed);
        let next = receipt(3, 2, Outcome::Started);
        let mut slots = [
            older.encode().expect("older"),
            latest.encode().expect("latest"),
        ];
        let mut writes = 0;
        // Act
        commit_words(next.encode().expect("next"), |index, value| {
            if writes < cut {
                slots[0][index] = value;
            }
            writes += 1;
        });
        let observed = recover(&slots, 0x1234);
        // Assert
        assert_eq!(observed.maybe_receipt, Some(latest));
        assert!(observed.interrupted);
    }
}

#[test]
fn checksummed_generation_regression_or_ambiguous_sequence_is_corrupt() {
    for same_sequence in [false, true] {
        // Arrange
        let older = receipt(1, 1, Outcome::Started);
        let mut newer = receipt(if same_sequence { 1 } else { 2 }, 1, Outcome::Started);
        newer.generation = 2;
        // Act
        let snapshot = recover(
            &[
                older.encode().expect("older"),
                newer.encode().expect("newer"),
            ],
            0x1234,
        );
        // Assert
        assert_eq!(snapshot.status, Status::Corrupt);
        assert!(snapshot.maybe_receipt.is_none());
    }
}

#[test]
fn typed_failure_codes_round_trip_and_unknown_codes_are_corrupt() {
    for raw in 1..=12 {
        // Arrange
        let mut value = receipt(raw, 4, Outcome::Failed);
        value.failure = Failure::parse(raw).expect("defined closed code");
        // Act
        let snapshot = recover(
            &[value.encode().expect("typed failure"), [0; WORDS]],
            0x1234,
        );
        // Assert
        assert_eq!(snapshot.maybe_receipt, Some(value));
        assert!(snapshot
            .marker(true)
            .contains(&format!("failure={}", value.failure.label())));
    }
    // Arrange: valid checksum cannot admit an undefined failure discriminator.
    let mut invalid = receipt(1, 4, Outcome::Failed).encode().expect("failure");
    invalid[12] = 3 | (13 << 8);
    invalid[17] = checksum(&invalid[1..17]);
    // Act / Assert
    assert_eq!(
        recover(&[invalid, [0; WORDS]], 0x1234).status,
        Status::Corrupt
    );
}
#[test]
fn incomplete_progress_cannot_fabricate_a_typed_failure() {
    // Arrange
    let mut started = receipt(1, 1, Outcome::Started);
    started.failure = Failure::AsicFailed;
    let mut failed = receipt(2, 1, Outcome::Failed);
    failed.failure = Failure::None;
    // Act / Assert
    assert!(started.encode().is_none());
    assert!(failed.encode().is_none());
}

#[test]
fn late_admission_failure_preserves_already_completed_step() {
    // Arrange
    let completed = receipt(4, 2, Outcome::Completed);
    let late = Receipt {
        sequence: 5,
        outcome: Outcome::Failed,
        failure: Failure::Cancelled,
        ..completed
    };
    // Act
    let observed = recover(
        &[
            completed.encode().expect("completed"),
            late.encode().expect("late failure"),
        ],
        0x1234,
    );
    // Assert
    assert_eq!(observed.maybe_receipt, Some(late));
}
