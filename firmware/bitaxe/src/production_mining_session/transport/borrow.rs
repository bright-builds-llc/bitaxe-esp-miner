//! Exclusive borrowing of an already idle pool worker; never a new thread.
use super::PoolTransportCommand;
use std::sync::{
    mpsc::{SyncSender, TrySendError},
    Arc, Mutex,
};

#[derive(Clone, Copy)]
struct Callbacks {
    run: fn(),
    complete: fn(),
}
#[derive(Default)]
enum Loan {
    #[default]
    Free,
    Reserved,
    Queued(Callbacks),
    Running,
    Completed,
}
#[derive(Default)]
struct State {
    ready: bool,
    queued: usize,
    in_flight: bool,
    connected: bool,
    loan: Loan,
}
#[derive(Clone)]
pub(crate) struct NoiseBorrowHandle {
    sender: SyncSender<PoolTransportCommand>,
    state: Arc<Mutex<State>>,
}
impl NoiseBorrowHandle {
    pub(super) fn new(sender: SyncSender<PoolTransportCommand>) -> Self {
        Self {
            sender,
            state: Arc::new(Mutex::new(State::default())),
        }
    }
    pub(crate) fn is_idle(&self) -> bool {
        self.state.try_lock().is_ok_and(|s| {
            s.ready && s.queued == 0 && !s.in_flight && !s.connected && matches!(s.loan, Loan::Free)
        })
    }
    pub(crate) fn reserve(&self) -> bool {
        let Ok(mut state) = self.state.try_lock() else {
            return false;
        };
        if !state.ready
            || state.queued != 0
            || state.in_flight
            || state.connected
            || !matches!(state.loan, Loan::Free)
        {
            return false;
        }
        state.loan = Loan::Reserved;
        true
    }
    pub(crate) fn dispatch(&self, run: fn(), complete: fn()) -> Result<(), ()> {
        let Ok(mut state) = self.state.lock() else {
            return Err(());
        };
        if !matches!(state.loan, Loan::Reserved) {
            return Err(());
        }
        self.sender
            .try_send(PoolTransportCommand::Diagnostic)
            .map_err(|_| ())?;
        state.queued += 1;
        state.loan = Loan::Queued(Callbacks { run, complete });
        Ok(())
    }
    pub(crate) fn release(&self) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if !matches!(state.loan, Loan::Reserved | Loan::Completed) {
            return false;
        }
        state.loan = Loan::Free;
        true
    }
    pub(super) fn send(
        &self,
        command: PoolTransportCommand,
    ) -> Result<(), TrySendError<PoolTransportCommand>> {
        let Ok(mut state) = self.state.lock() else {
            return Err(TrySendError::Disconnected(command));
        };
        if !matches!(state.loan, Loan::Free) || matches!(command, PoolTransportCommand::Diagnostic)
        {
            return Err(TrySendError::Full(command));
        }
        self.sender.try_send(command)?;
        state.queued += 1;
        Ok(())
    }
    pub(super) fn worker(&self) -> NoiseBorrowWorker {
        NoiseBorrowWorker {
            state: Arc::clone(&self.state),
        }
    }
}
pub(super) struct NoiseBorrowWorker {
    state: Arc<Mutex<State>>,
}
impl NoiseBorrowWorker {
    pub(super) fn ready(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.ready = true;
        }
    }
    pub(super) fn begin_command(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.queued = state.queued.saturating_sub(1);
            state.in_flight = true;
        }
    }
    pub(super) fn finish_command(&self, connected: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.in_flight = false;
            state.connected = connected;
        }
    }
    pub(super) fn connection(&self, connected: bool) {
        if let Ok(mut state) = self.state.lock() {
            state.connected = connected;
        }
    }
    /// Both callbacks run outside the metadata lock; completion follows scoped return.
    #[inline(never)]
    pub(super) fn run_job(&self) -> bool {
        let callbacks = {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            if state.connected {
                return false;
            }
            match std::mem::replace(&mut state.loan, Loan::Running) {
                Loan::Queued(callbacks) => callbacks,
                other => {
                    state.loan = other;
                    return false;
                }
            }
        };
        (callbacks.run)();
        // This caller observes actual job return, after every job-local owner dropped.
        if let Ok(mut state) = self.state.lock() {
            state.loan = Loan::Completed;
        }
        (callbacks.complete)();
        true
    }
    pub(super) fn unavailable(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.ready = false;
        }
    }
}
