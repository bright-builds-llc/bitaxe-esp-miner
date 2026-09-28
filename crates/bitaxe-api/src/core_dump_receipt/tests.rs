use super::*;

#[test]
fn only_returned_stages_publish_their_own_sdk_result() {
    // Arrange
    let expected = [
        None,
        None,
        None,
        Some(11),
        None,
        Some(12),
        None,
        Some(13),
        None,
        Some(14),
        Some(15),
    ];
    for (stage, maybe_result_index) in expected.into_iter().enumerate() {
        let mut words = ready(42, 18, 974848);
        // Act
        checkpoint(&mut words, stage as u32, 0, 0, -1);
        // Assert
        for (index, word) in words.iter().enumerate().take(16).skip(11) {
            assert_eq!(
                *word,
                if maybe_result_index == Some(index) {
                    u32::MAX
                } else {
                    0
                }
            );
        }
        assert_eq!(
            words[8],
            maybe_result_index.map_or(0, |index| 1 << (index - 9))
        );
        assert_eq!(classify(&words, 42, 18), Status::Valid);
    }
}

#[test]
fn source_and_exact_boot_are_separate_admission_checks() {
    // Arrange
    let words = ready(42, 18, 974848);
    // Act / Assert
    assert_eq!(classify(&words, 42, 18), Status::Valid);
    assert_eq!(classify(&words, 43, 18), Status::WrongFirmware);
    assert_eq!(classify(&words, 42, 19), Status::WrongBoot);
}

#[test]
fn every_interrupted_commit_is_rejected() {
    // Arrange
    let previous = ready(42, 18, 974848);
    let mut next = previous;
    checkpoint(&mut next, 5, 10, 999999, -1);
    // Act / Assert
    for completed_writes in 1..=WORDS {
        let mut torn = previous;
        let mut written = 0;
        commit_words(&next, |index, value| {
            if written < completed_writes {
                torn[index] = value;
            }
            written += 1;
        });
        assert_eq!(classify(&torn, 42, 18), Status::Corrupt);
    }
    let mut complete = previous;
    commit_words(&next, |index, value| complete[index] = value);
    assert_eq!(classify(&complete, 42, 18), Status::Valid);
}

#[test]
fn unavailable_is_distinct_from_minus_one_and_outer_error_does_not_replace_inner() {
    // Arrange
    let mut words = ready(42, 18, 974848);
    let initial = marker(&words, 42, 18, Origin::CurrentBoot);
    checkpoint(&mut words, 3, 0, 0, -1);
    checkpoint(&mut words, 10, 0, 0, -2);
    // Act
    let result = marker(&words, 42, 18, Origin::PreviousBoot);
    // Assert
    assert!(initial.contains("init_result=unavailable"));
    assert!(result.contains("init_result=-1"));
    assert!(result.contains("store_result=-2"));
    assert!(result.contains("prepare_result=unavailable"));
    assert!(result.len() < 1024);
}

#[test]
fn changed_payload_and_complement_are_rejected_without_exposure() {
    // Arrange
    let valid = ready(42, 18, 974848);
    // Act / Assert
    for index in 0..WORDS {
        let mut corrupt = valid;
        corrupt[index] ^= 1;
        assert_eq!(classify(&corrupt, 42, 18), Status::Corrupt);
        assert!(!marker(&corrupt, 42, 18, Origin::PreviousBoot).contains("source_hash="));
    }
}
