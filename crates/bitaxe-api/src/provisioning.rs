//! Pure configuration-network contracts for the provisioning access point.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/connect/connect.c:wifi_init_softap`
//! - `reference/esp-miner/components/dns_server/dns_server.c:start_dns_server`

use std::net::Ipv4Addr;

use thiserror::Error;

/// UDP port used by the configuration-network DNS owner.
pub const CAPTIVE_DNS_PORT: u16 = 53;
/// Maximum accepted request and emitted response size.
pub const CAPTIVE_DNS_PACKET_BYTES: usize = 256;
/// TTL used for wildcard configuration-network IPv4 answers.
pub const CAPTIVE_DNS_TTL_SECONDS: u32 = 300;
/// Recurring redaction-safe proof that all configuration-network owners are ready.
pub const PROVISIONING_NETWORK_READY_MARKER: &str =
    "provisioning_network_ready schema_version=1 ap=ready dhcp=ready dns=ready redacted=true";

const DNS_HEADER_BYTES: usize = 12;
const DNS_ANSWER_BYTES: usize = 16;
const DNS_MAX_QUESTIONS: u16 = 16;
const DNS_FLAG_RESPONSE: u16 = 0x8000;
const DNS_FLAG_RECURSION_DESIRED: u16 = 0x0100;
const DNS_OPCODE_MASK: u16 = 0x7800;
const DNS_TYPE_A: u16 = 1;
const DNS_CLASS_IN: u16 = 1;

/// Closed malformed or bounded captive-DNS request categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CaptiveDnsError {
    #[error("DNS packet length is outside the supported bound")]
    PacketLength,
    #[error("DNS query has no questions")]
    NoQuestions,
    #[error("DNS query exceeds the supported question count")]
    TooManyQuestions,
    #[error("DNS question is truncated")]
    TruncatedQuestion,
    #[error("compressed DNS question names are unsupported")]
    CompressedQuestionName,
    #[error("DNS question label exceeds the wire-format bound")]
    QuestionLabelTooLong,
    #[error("DNS question name exceeds the wire-format bound")]
    QuestionNameTooLong,
    #[error("DNS response exceeds the supported packet bound")]
    ResponseTooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AnswerPlan {
    name_offset: u16,
    answer_ipv4: bool,
}

/// Derives the upstream-shaped configuration SSID from the AP MAC address.
#[must_use]
pub fn configuration_ap_ssid(ap_mac: [u8; 6]) -> String {
    format!("Bitaxe_{:02X}{:02X}", ap_mac[4], ap_mac[5])
}

/// Requires two complete recurring configuration-network readiness samples.
#[must_use]
pub fn has_recurring_provisioning_network_ready(log: &str) -> bool {
    log.lines()
        .filter(|line| line.trim_end().ends_with(PROVISIONING_NETWORK_READY_MARKER))
        .count()
        >= 2
}

/// Builds a bounded wildcard DNS response for configuration-network clients.
///
/// Standard IN/A questions receive the supplied AP address. Other standard
/// question types receive a valid response with no answers so clients may
/// continue their normal A-query fallback. Responses and nonstandard opcodes
/// are ignored.
pub fn build_captive_dns_response(
    request: &[u8],
    ap_ipv4: Ipv4Addr,
) -> Result<Option<Vec<u8>>, CaptiveDnsError> {
    if !(DNS_HEADER_BYTES..=CAPTIVE_DNS_PACKET_BYTES).contains(&request.len()) {
        return Err(CaptiveDnsError::PacketLength);
    }

    let request_flags = read_u16(request, 2);
    if request_flags & DNS_FLAG_RESPONSE != 0 || request_flags & DNS_OPCODE_MASK != 0 {
        return Ok(None);
    }

    let question_count = read_u16(request, 4);
    if question_count == 0 {
        return Err(CaptiveDnsError::NoQuestions);
    }
    if question_count > DNS_MAX_QUESTIONS {
        return Err(CaptiveDnsError::TooManyQuestions);
    }

    let mut cursor = DNS_HEADER_BYTES;
    let mut answers = Vec::with_capacity(usize::from(question_count));
    for _ in 0..question_count {
        let name_offset = u16::try_from(cursor).map_err(|_| CaptiveDnsError::ResponseTooLarge)?;
        cursor = question_name_end(request, cursor)?;
        let question_end = cursor
            .checked_add(4)
            .ok_or(CaptiveDnsError::TruncatedQuestion)?;
        if question_end > request.len() {
            return Err(CaptiveDnsError::TruncatedQuestion);
        }

        let question_type = read_u16(request, cursor);
        let question_class = read_u16(request, cursor + 2);
        answers.push(AnswerPlan {
            name_offset,
            answer_ipv4: question_type == DNS_TYPE_A && question_class == DNS_CLASS_IN,
        });
        cursor = question_end;
    }

    let answer_count = answers.iter().filter(|answer| answer.answer_ipv4).count();
    let response_len = cursor
        .checked_add(
            answer_count
                .checked_mul(DNS_ANSWER_BYTES)
                .ok_or(CaptiveDnsError::ResponseTooLarge)?,
        )
        .ok_or(CaptiveDnsError::ResponseTooLarge)?;
    if response_len > CAPTIVE_DNS_PACKET_BYTES {
        return Err(CaptiveDnsError::ResponseTooLarge);
    }

    let mut response = request[..cursor].to_vec();
    write_u16(
        &mut response,
        2,
        DNS_FLAG_RESPONSE | (request_flags & DNS_FLAG_RECURSION_DESIRED),
    );
    write_u16(&mut response, 6, answer_count as u16);
    write_u16(&mut response, 8, 0);
    write_u16(&mut response, 10, 0);

    for answer in answers.iter().filter(|answer| answer.answer_ipv4) {
        response.extend_from_slice(&(0xc000 | answer.name_offset).to_be_bytes());
        response.extend_from_slice(&DNS_TYPE_A.to_be_bytes());
        response.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
        response.extend_from_slice(&CAPTIVE_DNS_TTL_SECONDS.to_be_bytes());
        response.extend_from_slice(&4_u16.to_be_bytes());
        response.extend_from_slice(&ap_ipv4.octets());
    }

    Ok(Some(response))
}

fn question_name_end(request: &[u8], mut cursor: usize) -> Result<usize, CaptiveDnsError> {
    let mut name_bytes = 0_usize;
    loop {
        let Some(&label_len) = request.get(cursor) else {
            return Err(CaptiveDnsError::TruncatedQuestion);
        };
        if label_len & 0xc0 == 0xc0 {
            return Err(CaptiveDnsError::CompressedQuestionName);
        }
        if label_len > 63 {
            return Err(CaptiveDnsError::QuestionLabelTooLong);
        }

        cursor += 1;
        name_bytes += 1;
        if label_len == 0 {
            return Ok(cursor);
        }

        let label_len = usize::from(label_len);
        let label_end = cursor
            .checked_add(label_len)
            .ok_or(CaptiveDnsError::QuestionNameTooLong)?;
        name_bytes = name_bytes
            .checked_add(label_len)
            .ok_or(CaptiveDnsError::QuestionNameTooLong)?;
        if name_bytes > u8::MAX as usize {
            return Err(CaptiveDnsError::QuestionNameTooLong);
        }
        if label_end > request.len() {
            return Err(CaptiveDnsError::TruncatedQuestion);
        }
        cursor = label_end;
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(questions: &[(&[&str], u16)], additional_count: u16) -> Vec<u8> {
        let mut request = vec![0x12, 0x34, 0x01, 0x00];
        request.extend_from_slice(&(questions.len() as u16).to_be_bytes());
        request.extend_from_slice(&0_u16.to_be_bytes());
        request.extend_from_slice(&0_u16.to_be_bytes());
        request.extend_from_slice(&additional_count.to_be_bytes());
        for (labels, question_type) in questions {
            for label in *labels {
                request.push(label.len() as u8);
                request.extend_from_slice(label.as_bytes());
            }
            request.push(0);
            request.extend_from_slice(&question_type.to_be_bytes());
            request.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
        }
        request
    }

    #[test]
    fn configuration_ssid_uses_uppercase_final_ap_mac_octets() {
        // Arrange / Act
        let ssid = configuration_ap_ssid([0x02, 0, 0, 0, 0xab, 0xcd]);

        // Assert
        assert_eq!(ssid, "Bitaxe_ABCD");
    }

    #[test]
    fn recurring_readiness_requires_two_complete_redacted_samples() {
        // Arrange
        let one = format!("I ready: {PROVISIONING_NETWORK_READY_MARKER}\n");
        let two = format!("{one}{one}");

        // Act / Assert
        assert!(!has_recurring_provisioning_network_ready(&one));
        assert!(has_recurring_provisioning_network_ready(&two));
        assert!(!has_recurring_provisioning_network_ready(
            "provisioning_network_ready schema_version=1 ap=ready dhcp=ready dns=missing redacted=true",
        ));
    }

    #[test]
    fn wildcard_a_query_returns_ap_address_and_pinned_ttl() {
        // Arrange
        let request = query(&[(&["connectivity", "check"], DNS_TYPE_A)], 0);

        // Act
        let response = build_captive_dns_response(&request, Ipv4Addr::new(192, 0, 2, 1))
            .expect("valid query")
            .expect("standard query response");

        // Assert
        assert_eq!(&response[0..2], &[0x12, 0x34]);
        assert_eq!(read_u16(&response, 2), 0x8100);
        assert_eq!(read_u16(&response, 4), 1);
        assert_eq!(read_u16(&response, 6), 1);
        assert_eq!(read_u16(&response, 8), 0);
        assert_eq!(read_u16(&response, 10), 0);
        assert_eq!(read_u16(&response, request.len()), 0xc00c);
        assert_eq!(read_u16(&response, request.len() + 2), DNS_TYPE_A);
        assert_eq!(
            u32::from_be_bytes(
                response[request.len() + 6..request.len() + 10]
                    .try_into()
                    .expect("TTL bytes"),
            ),
            CAPTIVE_DNS_TTL_SECONDS
        );
        assert_eq!(&response[response.len() - 4..], &[192, 0, 2, 1]);
    }

    #[test]
    fn non_a_query_receives_empty_standard_response() {
        // Arrange
        let request = query(&[(&["ipv6", "test"], 28)], 0);

        // Act
        let response = build_captive_dns_response(&request, Ipv4Addr::LOCALHOST)
            .expect("valid query")
            .expect("standard query response");

        // Assert
        assert_eq!(response.len(), request.len());
        assert_eq!(read_u16(&response, 6), 0);
        assert_eq!(read_u16(&response, 2), 0x8100);
    }

    #[test]
    fn multiple_questions_answer_only_in_a_and_use_each_name_offset() {
        // Arrange
        let request = query(
            &[
                (&["first", "test"], DNS_TYPE_A),
                (&["second", "test"], 28),
                (&["third", "test"], DNS_TYPE_A),
            ],
            0,
        );

        // Act
        let response = build_captive_dns_response(&request, Ipv4Addr::new(192, 0, 2, 2))
            .expect("valid query")
            .expect("standard query response");

        // Assert
        assert_eq!(read_u16(&response, 6), 2);
        assert_eq!(read_u16(&response, request.len()), 0xc00c);
        let third_offset = 12 + (1 + 5) + (1 + 4) + 1 + 4 + (1 + 6) + (1 + 4) + 1 + 4;
        assert_eq!(
            read_u16(&response, request.len() + DNS_ANSWER_BYTES),
            0xc000 | third_offset as u16
        );
    }

    #[test]
    fn response_drops_unparsed_additional_records_and_clears_count() {
        // Arrange
        let mut request = query(&[(&["portal", "test"], DNS_TYPE_A)], 1);
        let question_end = request.len();
        request.extend_from_slice(&[0, 0, 41, 4, 208, 0, 0, 0, 0, 0, 0]);

        // Act
        let response = build_captive_dns_response(&request, Ipv4Addr::new(192, 0, 2, 3))
            .expect("valid query")
            .expect("standard query response");

        // Assert
        assert_eq!(read_u16(&response, 10), 0);
        assert_eq!(response.len(), question_end + DNS_ANSWER_BYTES);
    }

    #[test]
    fn response_and_nonstandard_opcode_are_ignored() {
        // Arrange
        let mut response_packet = query(&[(&["portal", "test"], DNS_TYPE_A)], 0);
        response_packet[2] = 0x81;
        let mut inverse_query = query(&[(&["portal", "test"], DNS_TYPE_A)], 0);
        inverse_query[2] = 0x09;

        // Act / Assert
        assert_eq!(
            build_captive_dns_response(&response_packet, Ipv4Addr::LOCALHOST),
            Ok(None)
        );
        assert_eq!(
            build_captive_dns_response(&inverse_query, Ipv4Addr::LOCALHOST),
            Ok(None)
        );
    }

    #[test]
    fn malformed_and_oversized_queries_fail_closed() {
        // Arrange
        let too_short = [0_u8; DNS_HEADER_BYTES - 1];
        let too_long = [0_u8; CAPTIVE_DNS_PACKET_BYTES + 1];
        let mut no_questions = [0_u8; DNS_HEADER_BYTES];
        no_questions[2] = 1;
        let mut compressed = query(&[(&["portal"], DNS_TYPE_A)], 0);
        compressed[DNS_HEADER_BYTES] = 0xc0;
        let mut truncated = query(&[(&["portal"], DNS_TYPE_A)], 0);
        truncated.truncate(truncated.len() - 1);
        let mut overlong_label = query(&[(&["portal"], DNS_TYPE_A)], 0);
        overlong_label[DNS_HEADER_BYTES] = 64;

        // Act / Assert
        assert_eq!(
            build_captive_dns_response(&too_short, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::PacketLength)
        );
        assert_eq!(
            build_captive_dns_response(&too_long, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::PacketLength)
        );
        assert_eq!(
            build_captive_dns_response(&no_questions, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::NoQuestions)
        );
        assert_eq!(
            build_captive_dns_response(&compressed, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::CompressedQuestionName)
        );
        assert_eq!(
            build_captive_dns_response(&truncated, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::TruncatedQuestion)
        );
        assert_eq!(
            build_captive_dns_response(&overlong_label, Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::QuestionLabelTooLong)
        );
    }

    #[test]
    fn bounded_question_and_response_limits_fail_closed() {
        // Arrange
        let root_labels: &[&str] = &[];
        let seventeen = vec![(root_labels, DNS_TYPE_A); 17];
        let sixteen = vec![(root_labels, DNS_TYPE_A); 16];

        // Act / Assert
        assert_eq!(
            build_captive_dns_response(&query(&seventeen, 0), Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::TooManyQuestions)
        );
        assert_eq!(
            build_captive_dns_response(&query(&sixteen, 0), Ipv4Addr::LOCALHOST),
            Err(CaptiveDnsError::ResponseTooLarge)
        );
    }
}
