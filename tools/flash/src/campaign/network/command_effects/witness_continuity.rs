use bitaxe_http_transport::{WebSocketRead, WebSocketReadFailureKind};

use super::super::serial::observe_command_transition_lines;
use super::CommandTransitionWitness;

pub(super) fn consume_optional_websocket_read(
    observation: Result<WebSocketRead, WebSocketReadFailureKind>,
    expected_session: &str,
    pending: &mut Vec<u8>,
    witness: &mut CommandTransitionWitness,
) -> Result<bool, ()> {
    match observation {
        Ok(WebSocketRead::Text(bytes)) => {
            observe_websocket_transitions(&bytes, expected_session, pending, witness)?;
            Ok(true)
        }
        Ok(WebSocketRead::Timeout) => Ok(true),
        Ok(WebSocketRead::Closed) | Err(WebSocketReadFailureKind::Io) => {
            pending.clear();
            Ok(false)
        }
        Err(
            WebSocketReadFailureKind::Protocol
            | WebSocketReadFailureKind::Capacity
            | WebSocketReadFailureKind::Other,
        ) => Err(()),
    }
}

fn observe_websocket_transitions(
    bytes: &[u8],
    expected_session: &str,
    pending: &mut Vec<u8>,
    witness: &mut CommandTransitionWitness,
) -> Result<(), ()> {
    const MAX_PENDING_BYTES: usize = 65_536;
    pending.extend_from_slice(bytes);
    while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
        let line = pending.drain(..=newline).collect::<Vec<_>>();
        if line
            .windows(b"command_status_transition ".len())
            .any(|window| window == b"command_status_transition ")
            && !observe_command_transition_lines(&line, expected_session, witness)
        {
            return Err(());
        }
    }
    if pending.len() > MAX_PENDING_BYTES {
        return Err(());
    }
    Ok(())
}
