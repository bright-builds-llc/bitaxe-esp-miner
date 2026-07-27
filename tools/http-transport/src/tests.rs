use super::*;

#[test]
fn incomplete_write_is_never_complete() {
    struct FailingWriter {
        remaining: usize,
    }

    impl Write for FailingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "synthetic"));
            }
            let count = self.remaining.min(buffer.len());
            self.remaining -= count;
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Arrange
    let mut writer = FailingWriter { remaining: 7 };

    // Act
    let observation = send_request(&mut writer, b"complete request");

    // Assert
    assert_eq!(observation.bytes_written, 7);
    assert!(!observation.complete);
}

#[test]
fn invalid_origins_fail_closed() {
    for origin in [
        "ftp://example.invalid",
        "http://",
        "http://user@example.invalid",
        "http://example.invalid/path",
        "http://::1",
    ] {
        assert!(StrictHttpClient::new(origin).is_err());
    }
}

#[test]
fn chunked_body_requires_terminal_chunk() {
    assert_eq!(
        decode_chunked(b"4\r\ntest\r\n0\r\n\r\n"),
        Some(b"test".to_vec())
    );
    assert_eq!(decode_chunked(b"4\r\ntest\r\n"), None);
}
