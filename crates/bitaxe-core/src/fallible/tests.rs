use super::*;
use std::cell::RefCell;
use std::sync::Once;

thread_local! {
    static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn observe() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let installed = install(Hooks {
            begin: |site| {
                EVENTS.with(|events| events.borrow_mut().push(format!("begin:{}", site.0)))
            },
            end: || EVENTS.with(|events| events.borrow_mut().push("end".to_owned())),
        });
        assert!(installed);
    });
    EVENTS.with(|events| events.borrow_mut().clear());
}

fn events() -> Vec<String> {
    EVENTS.with(|events| events.borrow().clone())
}

#[test]
fn a_reservation_runs_inside_its_named_region() {
    // Arrange
    observe();
    let mut buffer: Vec<u8> = Vec::new();
    // Act
    reserve_exact(&mut buffer, 64, Site("test.buffer")).expect("reserve");
    // Assert
    assert_eq!(events(), ["begin:test.buffer", "end"]);
    assert!(buffer.capacity() >= 64);
}

#[test]
fn a_failed_reservation_still_closes_its_region() {
    // Arrange
    observe();
    let mut buffer: Vec<u8> = Vec::new();
    // Act
    let result = reserve(&mut buffer, usize::MAX, Site("test.overflow"));
    // Assert
    assert!(result.is_err());
    assert_eq!(events(), ["begin:test.overflow", "end"]);
}

#[test]
fn a_recoverable_string_copy_is_exact() {
    // Arrange
    observe();
    // Act
    let copy = try_string("nonce", Site("test.string")).expect("copy");
    // Assert
    assert_eq!(copy, "nonce");
    assert_eq!(events(), ["begin:test.string", "end"]);
}

#[test]
fn hooks_install_once() {
    // Arrange
    observe();
    // Act
    let again = install(Hooks {
        begin: |_| {},
        end: || {},
    });
    // Assert
    assert!(!again);
}
