use std::io;
use std::net::TcpStream;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use tungstenite::client::client_with_config;
use tungstenite::protocol::{Message, WebSocket, WebSocketConfig};
use tungstenite::Error;

use crate::{maybe_connect, Origin, Scheme};

const MAX_WEBSOCKET_BYTES: usize = 65_536;

/// One bounded plain-text WebSocket read outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketRead {
    Text(Vec<u8>),
    Timeout,
    Closed,
}

/// Plain `ws://` connection over one strictly admitted HTTP origin.
pub struct PlainWebSocket {
    socket: WebSocket<TcpStream>,
}

impl PlainWebSocket {
    /// Connects to one path without TLS, an Origin header, or an unbounded frame size.
    pub fn connect(
        origin: &str,
        route: &str,
        connect_timeout: Duration,
        io_timeout: Duration,
    ) -> Result<Self> {
        if route.is_empty()
            || !route.starts_with('/')
            || route.contains(char::is_whitespace)
            || route.contains(['?', '#'])
        {
            bail!("strict WebSocket route is invalid");
        }
        let origin = Origin::parse(origin)?;
        if origin.scheme != Scheme::Http {
            bail!("strict WebSocket transport requires plain HTTP origin");
        }
        let started = Instant::now();
        let deadline = started + connect_timeout;
        let stream = maybe_connect(&origin, started, deadline, connect_timeout)
            .context("strict WebSocket TCP connection failed")?;
        stream.set_read_timeout(Some(io_timeout))?;
        stream.set_write_timeout(Some(io_timeout))?;
        let request = format!("ws://{}{route}", origin.authority);
        let config = WebSocketConfig::default()
            .read_buffer_size(8_192)
            .write_buffer_size(8_192)
            .max_message_size(Some(MAX_WEBSOCKET_BYTES))
            .max_frame_size(Some(MAX_WEBSOCKET_BYTES));
        let (socket, _response) = client_with_config(request, stream, Some(config))
            .context("strict WebSocket handshake failed")?;
        Ok(Self { socket })
    }

    /// Reads one bounded message, handling control frames internally.
    pub fn read(&mut self) -> Result<WebSocketRead> {
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    return Ok(WebSocketRead::Text(text.as_bytes().to_vec()));
                }
                Ok(Message::Binary(_)) => bail!("strict WebSocket received binary payload"),
                Ok(Message::Close(_)) => return Ok(WebSocketRead::Closed),
                Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {
                    self.socket.flush()?;
                }
                Err(Error::Io(error)) if is_timeout(&error) => return Ok(WebSocketRead::Timeout),
                Err(Error::ConnectionClosed | Error::AlreadyClosed) => {
                    return Ok(WebSocketRead::Closed);
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    /// Requests a clean close without waiting indefinitely for the peer.
    pub fn close(&mut self) {
        let _result = self.socket.close(None);
    }
}

fn is_timeout(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    use tungstenite::accept_hdr;

    use super::*;

    #[test]
    fn plain_handshake_omits_origin_and_reads_bounded_text() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let (header_sender, header_receiver) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("client connection");
            let mut socket = accept_hdr(
                stream,
                |request: &tungstenite::handshake::server::Request, response| {
                    header_sender
                        .send(request.headers().contains_key("origin"))
                        .expect("header result");
                    Ok(response)
                },
            )
            .expect("server handshake");
            socket
                .send(Message::Text("bounded".into()))
                .expect("server send");
        });
        let origin = format!("http://{address}");

        // Act
        let mut client = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("client handshake");
        let message = client.read().expect("client read");

        // Assert
        assert_eq!(message, WebSocketRead::Text(b"bounded".to_vec()));
        assert!(!header_receiver.recv().expect("origin result"));
        server.join().expect("server join");
    }

    #[test]
    fn tls_and_path_bearing_routes_fail_before_connecting() {
        // Arrange
        let cases = [
            ("https://127.0.0.1", "/api/ws/live"),
            ("http://127.0.0.1", "api/ws/live"),
            ("http://127.0.0.1", "/api/ws/live?token=secret"),
        ];

        // Act
        let outcomes = cases.map(|(origin, route)| {
            PlainWebSocket::connect(
                origin,
                route,
                Duration::from_millis(1),
                Duration::from_millis(1),
            )
        });

        // Assert
        assert!(outcomes.into_iter().all(|outcome| outcome.is_err()));
    }
}
