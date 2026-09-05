//! Transfers the prepared server to the existing main task without another stack.
use super::{sys, EspHttpServer};

const _: () = assert!(sys::CONFIG_ESP_MAIN_TASK_STACK_SIZE == 16 * 1024);
const _: () = assert!(sys::CONFIG_PTHREAD_TASK_PRIO_DEFAULT == 5);

/// Owns the server until main activates cadence or failed initialization drops it.
pub(crate) struct PreparedHttpRuntime {
    server: EspHttpServer<'static>,
}

pub(super) struct ActiveHttpRuntime {
    server: EspHttpServer<'static>,
}

impl PreparedHttpRuntime {
    pub(super) fn new(server: EspHttpServer<'static>) -> Self {
        Self { server }
    }

    pub(super) fn activate(self) -> ActiveHttpRuntime {
        // Preserve CPU0 affinity; only the former telemetry owner's priority is
        // adopted, after initialization has unwound back to main.
        unsafe { sys::vTaskPrioritySet(std::ptr::null_mut(), 5) };
        let active = ActiveHttpRuntime {
            server: self.server,
        };
        crate::storage_http_diagnostics::http_outcome(true);
        crate::startup::complete();
        active
    }

    /// Runs only after startup returns, reusing main's full 16 KiB stack.
    pub(crate) fn run(self) -> ! {
        let active = self.activate();
        super::live_telemetry_cadence_loop(&active.server)
    }
}
