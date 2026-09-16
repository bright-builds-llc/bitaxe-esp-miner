//! Qualified entropy seeds one fallibly allocated, work-item-local generator.
use rand::{rngs::StdRng, SeedableRng};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    Allocation,
    Entropy,
}

#[inline(never)]
pub(crate) fn prepare<E>(
    fill: impl FnOnce(&mut [u8]) -> Result<(), E>,
) -> Result<Vec<StdRng>, Failure> {
    prepare_with(fill, |slot| {
        slot.try_reserve_exact(1).map_err(|_| Failure::Allocation)
    })
}
fn prepare_with<E>(
    fill: impl FnOnce(&mut [u8]) -> Result<(), E>,
    reserve: impl FnOnce(&mut Vec<StdRng>) -> Result<(), Failure>,
) -> Result<Vec<StdRng>, Failure> {
    let mut slot = Vec::new();
    reserve(&mut slot)?;
    if slot.capacity() == 0 {
        return Err(Failure::Allocation);
    }
    let mut seed = Zeroizing::new([0_u8; 32]);
    fill(seed.as_mut()).map_err(|_| Failure::Entropy)?;
    slot.push(StdRng::from_seed(*seed));
    Ok(slot)
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    #[test]
    fn qualified_seed_produces_the_same_local_generator() {
        // Arrange
        let mut expected = StdRng::from_seed([7; 32]);
        // Act
        let mut actual = prepare(|bytes| {
            bytes.fill(7);
            Ok::<_, ()>(())
        })
        .expect("generator");
        // Assert
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].next_u64(), expected.next_u64());
    }
    #[test]
    fn allocation_failure_stops_before_entropy_access() {
        // Arrange
        let called = std::cell::Cell::new(false);
        // Act
        let result = prepare_with(
            |_| {
                called.set(true);
                Ok::<_, ()>(())
            },
            |slot| {
                slot.try_reserve_exact(usize::MAX)
                    .map_err(|_| Failure::Allocation)
            },
        );
        // Assert
        assert!(matches!(result, Err(Failure::Allocation)));
        assert!(!called.get());
    }
    #[test]
    fn entropy_failure_never_produces_a_fallback_generator() {
        // Arrange / Act
        let result = prepare(|bytes| {
            bytes.fill(9);
            Err::<(), _>(())
        });
        // Assert
        assert!(matches!(result, Err(Failure::Entropy)));
    }
}
