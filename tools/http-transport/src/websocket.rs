use std::io;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use tungstenite::client::client_with_config;
use tungstenite::protocol::{Message, WebSocket, WebSocketConfig};
use tungstenite::Error;

use crate::{maybe_connect, Origin, Scheme};

const MAX_WEBSOCKET_BYTES: usize = 65_536;
const IO_RETRY_INTERVAL: Duration = Duration::from_millis(25);

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
    read_timeout: Duration,
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
        let (mut socket, _response) = client_with_config(request, stream, Some(config))
            .context("strict WebSocket handshake failed")?;
        socket.get_mut().set_read_timeout(None)?;
        socket.get_mut().set_write_timeout(None)?;
        socket.get_mut().set_nonblocking(true)?;
        Ok(Self {
            socket,
            read_timeout: io_timeout,
        })
    }

    /// Reads one bounded message, handling control frames internally.
    pub fn read(&mut self) -> Result<WebSocketRead> {
        let deadline = Instant::now()
            .checked_add(self.read_timeout)
            .context("strict WebSocket read deadline overflowed")?;
        loop {
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    return Ok(WebSocketRead::Text(text.as_bytes().to_vec()));
                }
                Ok(Message::Binary(_)) => bail!("strict WebSocket received binary payload"),
                Ok(Message::Close(_)) => return Ok(WebSocketRead::Closed),
                Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {
                    if !self.flush_until(deadline)? {
                        return Ok(WebSocketRead::Timeout);
                    }
                }
                Err(Error::Io(error))
                    if classify_io_error(&error) == IoErrorDisposition::Retryable =>
                {
                    if !wait_for_retry(deadline) {
                        return Ok(WebSocketRead::Timeout);
                    }
                }
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

    fn flush_until(&mut self, deadline: Instant) -> Result<bool> {
        Ok(retry_would_block_until(deadline, || self.socket.flush())?.is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IoErrorDisposition {
    Retryable,
    Fatal,
}

fn classify_io_error(error: &io::Error) -> IoErrorDisposition {
    match error.kind() {
        io::ErrorKind::WouldBlock => IoErrorDisposition::Retryable,
        _ => IoErrorDisposition::Fatal,
    }
}

fn retry_would_block_until<T>(
    deadline: Instant,
    mut operation: impl FnMut() -> std::result::Result<T, Error>,
) -> Result<Option<T>> {
    loop {
        match operation() {
            Ok(value) => return Ok(Some(value)),
            Err(Error::Io(error)) if classify_io_error(&error) == IoErrorDisposition::Retryable => {
                if !wait_for_retry(deadline) {
                    return Ok(None);
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn wait_for_retry(deadline: Instant) -> bool {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return false;
    }
    thread::sleep(remaining.min(IO_RETRY_INTERVAL));
    Instant::now() < deadline
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

    #[test]
    fn only_would_block_is_a_reusable_tungstenite_io_error() {
        // Arrange
        let would_block = io::Error::from(io::ErrorKind::WouldBlock);
        let timed_out = io::Error::from(io::ErrorKind::TimedOut);
        let reset = io::Error::from(io::ErrorKind::ConnectionReset);

        // Act
        let dispositions = [would_block, timed_out, reset].map(|error| classify_io_error(&error));

        // Assert
        assert_eq!(
            dispositions,
            [
                IoErrorDisposition::Retryable,
                IoErrorDisposition::Fatal,
                IoErrorDisposition::Fatal,
            ]
        );
    }

    #[test]
    fn control_frame_flush_retries_a_temporary_would_block() {
        // Arrange
        let mut attempts = 0_u8;
        let deadline = Instant::now() + Duration::from_millis(100);

        // Act
        let result = retry_would_block_until(deadline, || {
            attempts = attempts.saturating_add(1);
            if attempts == 1 {
                Err(Error::Io(io::Error::from(io::ErrorKind::WouldBlock)))
            } else {
                Ok(())
            }
        })
        .expect("bounded retry");

        // Assert
        assert_eq!(result, Some(()));
        assert_eq!(attempts, 2);
    }

    #[test]
    fn one_connection_survives_109_idle_observation_intervals_then_reads_text() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let (release_sender, release_receiver) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("single client connection");
            let mut socket = tungstenite::accept(stream).expect("server handshake");
            release_receiver.recv().expect("release idle server");
            socket
                .send(Message::Text("after-idle".into()))
                .expect("server text send");
        });
        let origin = format!("http://{address}");
        let mut client = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_millis(2),
        )
        .expect("client handshake");

        // Act
        let idle_started = Instant::now();
        for _ in 0..109 {
            assert_eq!(
                client.read().expect("idle read outcome"),
                WebSocketRead::Timeout
            );
        }
        let idle_elapsed = idle_started.elapsed();
        release_sender.send(()).expect("release server");
        let deadline = Instant::now() + Duration::from_secs(1);
        let message = loop {
            match client.read().expect("post-idle read outcome") {
                WebSocketRead::Text(bytes) => break bytes,
                WebSocketRead::Timeout if Instant::now() < deadline => {}
                outcome => panic!("unexpected post-idle outcome: {outcome:?}"),
            }
        };

        // Assert
        assert_eq!(message, b"after-idle");
        assert!(idle_elapsed >= Duration::from_millis(150));
        assert!(idle_elapsed < Duration::from_secs(2));
        server.join().expect("server join");
    }

    #[test]
    fn nonblocking_ping_flushes_pong_and_continues_to_text() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("client connection");
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .expect("server read timeout");
            let mut socket = tungstenite::accept(stream).expect("server handshake");
            socket
                .send(Message::Ping(b"health".to_vec().into()))
                .expect("server ping");
            assert!(matches!(socket.read(), Ok(Message::Pong(_))));
            socket
                .send(Message::Text("after-pong".into()))
                .expect("server text send");
        });
        let origin = format!("http://{address}");
        let mut client = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("client handshake");

        // Act
        let message = client.read().expect("client read");

        // Assert
        assert_eq!(message, WebSocketRead::Text(b"after-pong".to_vec()));
        server.join().expect("server join");
    }

    #[test]
    fn message_limit_accepts_exactly_64_kib_and_rejects_one_byte_more() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("client connection");
            let mut socket = tungstenite::accept(stream).expect("server handshake");
            socket
                .send(Message::Text("a".repeat(MAX_WEBSOCKET_BYTES).into()))
                .expect("bounded server text");
            socket
                .send(Message::Text("b".repeat(MAX_WEBSOCKET_BYTES + 1).into()))
                .expect("oversized server text");
        });
        let origin = format!("http://{address}");
        let mut client = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("client handshake");

        // Act
        let bounded = client.read().expect("bounded client read");
        let oversized = client.read();

        // Assert
        assert_eq!(
            bounded,
            WebSocketRead::Text(vec![b'a'; MAX_WEBSOCKET_BYTES])
        );
        assert!(oversized.is_err());
        server.join().expect("server join");
    }

    #[test]
    fn peer_close_releases_the_socket_for_a_fresh_connection() {
        // Arrange
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let server = thread::spawn(move || {
            let (first_stream, _) = listener.accept().expect("first client connection");
            let mut first = tungstenite::accept(first_stream).expect("first server handshake");
            first.close(None).expect("server close");
            drop(first);

            let (second_stream, _) = listener.accept().expect("second client connection");
            let mut second = tungstenite::accept(second_stream).expect("second server handshake");
            second
                .send(Message::Text("reconnected".into()))
                .expect("second server text");
        });
        let origin = format!("http://{address}");
        let mut first = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("first client handshake");

        // Act
        let closed = first.read().expect("first client close");
        drop(first);
        let mut second = PlainWebSocket::connect(
            &origin,
            "/api/ws/live",
            Duration::from_secs(1),
            Duration::from_secs(1),
        )
        .expect("second client handshake");
        let message = second.read().expect("second client read");

        // Assert
        assert_eq!(closed, WebSocketRead::Closed);
        assert_eq!(message, WebSocketRead::Text(b"reconnected".to_vec()));
        server.join().expect("server join");
    }
}
