//! Reserves one reconnect worker before service allocations; later subscription activation cannot spawn.
use crate::prepared_thread::{self, Prepared};
use bitaxe_core::wifi_reconnect::WifiReconnectEvent;
use std::io;
use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) struct PreparedReconnect {
    sender: Sender<WifiReconnectEvent>,
    thread: Prepared,
}

pub(crate) enum CredentialState<C> {
    Missing,
    Invalid,
    Valid(C),
}

pub(crate) enum PreparedCredentials<C> {
    Missing,
    Invalid,
    Valid {
        credentials: C,
        reconnect: PreparedReconnect,
    },
}

pub(crate) fn prepare_credentials<C>(
    state: CredentialState<C>,
) -> io::Result<PreparedCredentials<C>> {
    prepare_credentials_with(state, prepare)
}

fn prepare_credentials_with<C>(
    state: CredentialState<C>,
    spawn: impl FnOnce() -> io::Result<PreparedReconnect>,
) -> io::Result<PreparedCredentials<C>> {
    match state {
        CredentialState::Missing => Ok(PreparedCredentials::Missing),
        CredentialState::Invalid => Ok(PreparedCredentials::Invalid),
        CredentialState::Valid(credentials) => {
            spawn().map(|reconnect| PreparedCredentials::Valid {
                credentials,
                reconnect,
            })
        }
    }
}

fn prepare() -> io::Result<PreparedReconnect> {
    prepare_with(|receiver| {
        prepared_thread::spawn(
            "wifi-reconnect",
            8_192,
            Box::new(move || super::run(receiver)),
        )
    })
}

fn prepare_with(
    spawn: impl FnOnce(Receiver<WifiReconnectEvent>) -> io::Result<Prepared>,
) -> io::Result<PreparedReconnect> {
    let (sender, receiver) = mpsc::channel();
    let thread = spawn(receiver)?;
    Ok(PreparedReconnect { sender, thread })
}

impl PreparedReconnect {
    pub(super) fn sender(&self) -> &Sender<WifiReconnectEvent> {
        &self.sender
    }

    /// Installs the exact owner subscriptions before waking, and rolls them back if activation fails.
    pub(super) fn activate_subscribed<W, I>(
        &mut self,
        wifi_slot: &mut Option<W>,
        ip_slot: &mut Option<I>,
        wifi: W,
        ip: I,
    ) -> bool {
        if wifi_slot.is_some() || ip_slot.is_some() {
            return false;
        }
        *wifi_slot = Some(wifi);
        *ip_slot = Some(ip);
        if self.thread.activate() {
            return true;
        }
        wifi_slot.take();
        ip_slot.take();
        false
    }
}

#[cfg(test)]
#[path = "preparation_tests.rs"]
mod tests;
