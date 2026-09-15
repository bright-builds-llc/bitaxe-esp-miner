use super::initialize;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Reservation {
    heap: Rc<RefCell<Vec<usize>>>,
    block: usize,
    bytes: usize,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.heap.borrow_mut()[self.block] += self.bytes;
    }
}

fn reserve(heap: &Rc<RefCell<Vec<usize>>>, bytes: usize) -> Result<Reservation, &'static str> {
    let mut blocks = heap.borrow_mut();
    let block = blocks
        .iter()
        .position(|free| *free >= bytes)
        .ok_or("allocation")?;
    blocks[block] -= bytes;
    Ok(Reservation {
        heap: Rc::clone(heap),
        block,
        bytes,
    })
}

#[test]
fn larger_owner_first_avoids_the_reproduced_fragmentation_failure() {
    // Arrange: a bounded allocator model, not a measurement of ESP-IDF heap placement.
    let heap = Rc::new(RefCell::new(vec![20_000, 10_000]));
    let old_deferred = reserve(&heap, 8192).expect("smaller owner fits first");
    assert!(reserve(&heap, 16384).is_err());
    drop(old_deferred);
    let deferred = RefCell::new(None);

    // Act: invoke the production orchestration boundary with the same allocations.
    let server = initialize(
        || reserve(&heap, 16384),
        || {
            deferred.replace(Some(reserve(&heap, 8192)?));
            Ok(())
        },
    )
    .expect("both owners fit in the changed order");

    // Assert
    assert_eq!(heap.borrow().iter().sum::<usize>(), 30_000 - 16384 - 8192);
    drop(server);
    deferred.take();
    assert_eq!(*heap.borrow(), vec![20_000, 10_000]);
}

#[test]
fn server_failure_never_starts_the_deferred_worker() {
    // Arrange
    let started = Cell::new(false);
    // Act
    let result = initialize::<(), _>(
        || Err("server allocation"),
        || {
            started.set(true);
            Ok(())
        },
    );
    // Assert
    assert_eq!(result, Err("server allocation"));
    assert!(!started.get());
}

#[test]
fn deferred_failure_releases_server_and_preserves_the_original_error() {
    // Arrange
    let heap = Rc::new(RefCell::new(vec![16384]));
    // Act
    let result = initialize(|| reserve(&heap, 16384), || Err("deferred allocation"));
    // Assert
    assert_eq!(result.err(), Some("deferred allocation"));
    assert_eq!(*heap.borrow(), vec![16384]);
}
