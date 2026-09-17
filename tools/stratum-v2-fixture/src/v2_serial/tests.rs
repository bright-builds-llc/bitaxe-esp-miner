use super::*;
fn bytes<const N: usize>(hex: &str) -> [u8; N] {
    assert_eq!(hex.len(), N * 2);
    std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).expect("synthetic hex"))
}
#[test]
fn target_1024_and_independent_genesis_oracle() {
    // Arrange: public Bitcoin genesis is a unit oracle, never this fixture's hardware job.
    let header = oracle::header(
        1,
        [0; 32],
        bytes("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a"),
        1231006505,
        0x1d00ffff,
        2083236893,
    );
    // Act
    let hash = oracle::hash(&header);
    // Assert
    assert_eq!(
        oracle::hex(&hash),
        "6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"
    );
    assert_eq!(
        oracle::hex(&oracle::TARGET),
        "000000000000000000000000000000000000000000000000c0ff3f0000000000"
    );
    assert!(oracle::meets_target(&hash, &oracle::TARGET));
}
#[test]
fn asymmetric_nonzero_version_header_matches_independent_negative_oracle() {
    // Arrange: checked-in independent Python/OpenSSL vector, copied as non-secret test bytes.
    let previous = std::array::from_fn(|i| i as u8);
    let merkle = std::array::from_fn(|i| (i + 32) as u8);
    // Act
    let header = oracle::header(
        536870916 | 4218880,
        previous,
        merkle,
        1694564867,
        486604799,
        305419896,
    );
    // Assert
    assert_eq!(
        oracle::hex(&oracle::hash(&header)),
        "67dfaeb3b4bb6ab4b0ab13aa9b6abb8b9b3263c5b9ee034056e8434fddd861bb"
    );
    assert!(!oracle::meets_target(
        &oracle::hash(&header),
        &oracle::TARGET
    ));
}
#[test]
fn target_equality_and_next_integer_are_distinct() {
    let mut above = oracle::TARGET;
    above[0] = 1;
    assert!(oracle::meets_target(&oracle::TARGET, &oracle::TARGET));
    assert!(!oracle::meets_target(&above, &oracle::TARGET));
}
#[test]
fn rolling_version_is_only_existing_hardware_subset() {
    assert!(oracle::valid_version(oracle::VERSION | oracle::MASK));
    assert!(!oracle::valid_version(oracle::VERSION | 1));
    assert!(!oracle::valid_version(oracle::MASK));
}

mod boundary;
mod protocol;
