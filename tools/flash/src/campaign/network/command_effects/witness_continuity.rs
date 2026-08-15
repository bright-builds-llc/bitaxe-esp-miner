use bitaxe_http_transport::{WebSocketRead, WebSocketReadFailureKind};

use super::{observe_websocket_transitions, CommandTransitionWitness};

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
