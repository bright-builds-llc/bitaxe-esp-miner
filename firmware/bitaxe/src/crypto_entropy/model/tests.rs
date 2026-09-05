use super::*;

struct Source {
    events: Vec<&'static str>,
    fail: bool,
    seed: u8,
}
impl StartupEntropySource for Source {
    fn enable(&mut self) {
        self.events.push("enable");
    }
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), EntropyError> {
        self.events.push("fill");
        seed.fill(self.seed);
        if self.fail {
            Err(EntropyError::Source)
        } else {
            Ok(())
        }
    }
    fn disable(&mut self) {
        self.events.push("disable");
    }
}
fn source(fail: bool) -> Source {
    Source {
        events: vec![],
        fail,
        seed: 17,
    }
}

#[test]
fn runtime_generation_uses_seeded_stream_without_reentering_hardware() {
    // Arrange
    let owner = EntropyOwner::new();
    let mut source = source(false);
    owner.initialize(&mut source).expect("startup entropy");
    let mut first = [0u8; 32];
    let mut second = [0u8; 32];
    // Act
    owner.fill(&mut first).expect("first logical session");
    owner
        .fill(&mut second)
        .expect("later session while RF unavailable");
    // Assert
    assert_ne!(first, second);
    assert_eq!(source.events, ["enable", "fill", "disable"]);
}

#[test]
fn failed_seed_disables_hardware_and_seals_admission() {
    // Arrange
    let owner = EntropyOwner::new();
    let mut source = source(true);
    let mut bytes = [255u8; 32];
    // Act
    assert_eq!(owner.initialize(&mut source), Err(EntropyError::Source));
    assert_eq!(
        owner.initialize(&mut source),
        Err(EntropyError::AlreadyInitialized)
    );
    // Assert
    assert_eq!(owner.fill(&mut bytes), Err(EntropyError::Unavailable));
    assert_eq!(bytes, [0u8; 32]);
    assert_eq!(source.events, ["enable", "fill", "disable"]);
}

#[test]
fn unavailable_or_contended_owner_never_returns_prior_nonce_bytes() {
    // Arrange
    let owner = EntropyOwner::new();
    let mut bytes = [255u8; 32];
    assert_eq!(owner.fill(&mut bytes), Err(EntropyError::Unavailable));
    owner
        .initialize(&mut source(false))
        .expect("startup entropy");
    let _held = owner.maybe_rng.lock().expect("fixture contention");
    bytes.fill(255);
    // Act / Assert
    assert_eq!(owner.fill(&mut bytes), Err(EntropyError::Unavailable));
    assert_eq!(bytes, [0u8; 32]);
}

#[test]
fn a_new_boot_seed_changes_the_session_stream() {
    // Arrange
    let first = EntropyOwner::new();
    let second = EntropyOwner::new();
    first.initialize(&mut source(false)).expect("first boot");
    let mut next_source = source(false);
    next_source.seed = 18;
    second.initialize(&mut next_source).expect("next boot");
    let mut a = [0u8; 32];
    let mut b = [0u8; 32];
    // Act
    first.fill(&mut a).expect("first stream");
    second.fill(&mut b).expect("second stream");
    // Assert
    assert_ne!(a, b);
}
