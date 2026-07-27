//! Bounded newline framing for Stratum V1 JSON messages.

use crate::StratumV1Error;

pub const MAX_STRATUM_JSON_LINE_BYTES: usize = 16 * 1024;

#[derive(Debug, Default)]
pub(crate) struct StratumLineFramer {
    buffered: Vec<u8>,
}

impl StratumLineFramer {
    pub(crate) fn push(&mut self, bytes: &[u8]) -> Result<Vec<String>, StratumV1Error> {
        let mut lines = Vec::new();

        for byte in bytes {
            if *byte != b'\n' {
                if self.buffered.len() == MAX_STRATUM_JSON_LINE_BYTES {
                    return Err(StratumV1Error::InvalidField {
                        field: "stratum_json_line",
                        reason: "JSON line exceeded 16 KiB",
                    });
                }
                self.buffered.push(*byte);
                continue;
            }

            let mut line = std::mem::take(&mut self.buffered);
            if line.ends_with(b"\r") {
                let _removed = line.pop();
            }
            let line = String::from_utf8(line).map_err(|_| StratumV1Error::InvalidField {
                field: "stratum_json_line",
                reason: "JSON line was not valid UTF-8",
            })?;
            if line.len() > MAX_STRATUM_JSON_LINE_BYTES {
                return Err(StratumV1Error::InvalidField {
                    field: "stratum_json_line",
                    reason: "JSON line exceeded 16 KiB",
                });
            }
            lines.push(line);
        }

        Ok(lines)
    }

    pub(crate) fn clear(&mut self) {
        self.buffered.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fragmented_and_coalesced_crlf_input_is_framed_in_order() {
        // Arrange
        let mut framer = StratumLineFramer::default();

        // Act
        let first = framer
            .push(br#"{"id":1"#)
            .expect("fragment should be accepted");
        let second = framer
            .push(b"}\r\n{\"id\":2}\n")
            .expect("completed and coalesced lines should be accepted");

        // Assert
        assert!(first.is_empty());
        assert_eq!(second, [r#"{"id":1}"#, r#"{"id":2}"#]);
    }

    #[test]
    fn oversized_and_invalid_utf8_lines_fail_closed() {
        // Arrange
        let mut oversized = StratumLineFramer::default();
        let mut invalid = StratumLineFramer::default();

        // Act
        let oversized_error = oversized
            .push(&vec![b'x'; MAX_STRATUM_JSON_LINE_BYTES + 1])
            .expect_err("oversized line must fail");
        let invalid_error = invalid
            .push(&[0xff, b'\n'])
            .expect_err("invalid UTF-8 must fail");

        // Assert
        assert!(oversized_error.to_string().contains("16 KiB"));
        assert!(invalid_error.to_string().contains("UTF-8"));
    }
}
