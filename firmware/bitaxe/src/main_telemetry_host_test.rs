//! Exercises the production main-task handoff without ESP-IDF or hardware.
use std::cell::RefCell;
use std::marker::PhantomData;

#[allow(dead_code)]
#[path = "http_api/cadence_owner.rs"]
mod cadence_owner;
#[allow(dead_code)]
#[path = "bwg_worker_usb/startup_diagnostics.rs"]
mod startup_diagnostics;

thread_local! {
    static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    static PROGRESS: startup_diagnostics::StartupProgress = const { startup_diagnostics::StartupProgress::new() };
}

struct EspHttpServer<'a>(PhantomData<&'a ()>);
impl Drop for EspHttpServer<'_> {
    fn drop(&mut self) {
        EVENTS.with_borrow_mut(|events| events.push("server_dropped"));
    }
}
#[allow(non_snake_case)]
mod sys {
    pub const CONFIG_ESP_MAIN_TASK_STACK_SIZE: usize = 16 * 1024;
    pub const CONFIG_PTHREAD_TASK_PRIO_DEFAULT: u32 = 5;
    pub unsafe fn vTaskPrioritySet(task: *mut (), priority: u32) {
        assert!(
            task.is_null(),
            "only the current main task changes priority"
        );
        assert_eq!(priority, 5);
        super::EVENTS.with_borrow_mut(|events| events.push("priority_set"));
    }
}
mod storage_http_diagnostics {
    pub fn http_outcome(ready: bool) {
        assert!(ready);
        super::EVENTS.with_borrow_mut(|events| events.push("http_ready"));
    }
}
mod startup {
    pub fn complete() {
        super::PROGRESS.with(|progress| {
            progress.enter(super::startup_diagnostics::Stage::RuntimeReady);
            progress.complete();
        });
        super::EVENTS.with_borrow_mut(|events| events.push("runtime_complete"));
    }
}
fn live_telemetry_cadence_loop(_owner: &EspHttpServer<'static>) -> ! {
    panic!("the host tests activate ownership without entering an infinite loop")
}

const HTTP_SOURCE: &str = include_str!("http_api.rs");
const WEBSOCKET_SOURCE: &str = include_str!("http_api/websocket.rs");
const MAIN_SOURCE: &str = include_str!("main.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn telemetry_reuses_main_after_startup_returns_without_a_second_stack() {
    // Arrange / Act / Assert
    assert!(!WEBSOCKET_SOURCE.contains("thread::Builder"));
    assert!(!HTTP_SOURCE.contains("LIVE_TELEMETRY_THREAD_STACK_BYTES"));
    assert!(!HTTP_SOURCE.contains("mem::forget(server)"));
    assert!(MAIN_SOURCE.contains("let maybe_http = startup::run()?;"));
    assert!(MAIN_SOURCE.contains("http.run();"));
    assert!(!STARTUP_SOURCE.contains("PROGRESS.complete();\n    Ok("));
    assert!(WEBSOCKET_SOURCE.contains("Duration::from_millis(LIVE_TELEMETRY_CADENCE_MS)"));
    assert!(WEBSOCKET_SOURCE.contains("sys::httpd_queue_work("));
}

#[test]
fn preparation_does_not_publish_readiness_or_change_priority() {
    // Arrange
    let server = EspHttpServer(PhantomData);
    // Act
    let prepared = cadence_owner::PreparedHttpRuntime::new(server);
    // Assert
    EVENTS.with_borrow(|events| assert!(events.is_empty()));
    PROGRESS.with(|progress| assert!(!progress.marker(0).contains("state=complete")));
    drop(prepared);
}

#[test]
fn failed_initialization_drops_server_without_activating_it() {
    // Arrange
    let prepared = cadence_owner::PreparedHttpRuntime::new(EspHttpServer(PhantomData));
    // Act
    let failure: Result<(), &str> = (|| {
        let _owned_until_startup_returns = prepared;
        Err("later initialization failed")
    })();
    // Assert
    assert_eq!(failure, Err("later initialization failed"));
    EVENTS.with_borrow(|events| assert_eq!(events, &["server_dropped"]));
}

#[test]
fn activation_sets_priority_before_readiness_and_keeps_server_alive() {
    // Arrange
    let prepared = cadence_owner::PreparedHttpRuntime::new(EspHttpServer(PhantomData));
    // Act
    let active = prepared.activate();
    // Assert
    EVENTS.with_borrow(|events| {
        assert_eq!(events, &["priority_set", "http_ready", "runtime_complete"]);
    });
    PROGRESS.with(|progress| {
        assert!(progress
            .marker(0)
            .contains("state=complete first_failure=none"))
    });
    drop(active);
    EVENTS.with_borrow(|events| assert_eq!(events.last(), Some(&"server_dropped")));
}

#[test]
fn activation_cannot_erase_an_earlier_required_owner_failure() {
    // Arrange
    PROGRESS.with(|progress| progress.fail(startup_diagnostics::Stage::Statistics));
    let prepared = cadence_owner::PreparedHttpRuntime::new(EspHttpServer(PhantomData));
    // Act
    let _active = prepared.activate();
    // Assert
    PROGRESS.with(|progress| {
        assert!(progress
            .marker(0)
            .contains("state=complete first_failure=statistics"));
    });
}
