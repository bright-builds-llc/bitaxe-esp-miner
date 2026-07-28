use crate::*;

pub(crate) fn sanitize_evidence_text(text: &str, redaction_mode: EvidenceRedactionMode) -> String {
    const NEVER_PERSIST_FIELDS: &[&str] = &[
        "wifiPass",
        "wifipass",
        "wifi_password",
        "password",
        "pass",
        "token",
        "apiKey",
        "api_key",
        "pool_password",
        "poolPassword",
        "stratumPassword",
        "nvsSecret",
        "secret",
        "poolURL",
        "poolPort",
        "poolUser",
        "poolWorker",
        "worker",
        "ownerAddress",
        "btcAddress",
    ];
    let without_secret_json_fields = redact_json_string_fields(text, NEVER_PERSIST_FIELDS);
    let without_secret_json_scalars =
        redact_json_scalar_fields(&without_secret_json_fields, NEVER_PERSIST_FIELDS);
    let without_secret_tokens =
        redact_key_value_tokens(&without_secret_json_scalars, NEVER_PERSIST_FIELDS);

    if redaction_mode == EvidenceRedactionMode::DeveloperRaw {
        return without_secret_tokens;
    }

    let without_network_json_fields =
        redact_json_string_fields(&without_secret_tokens, &["ssid", "hostname", "hostName"]);
    let without_urls = redact_urls(&without_network_json_fields);
    let without_macs = redact_mac_addresses(&without_urls);
    let without_ips = redact_ipv4_addresses(&without_macs);
    let without_wifi_driver_ssids = redact_wifi_driver_connected_ssids(&without_ips);
    let without_operational_tokens = redact_key_value_tokens(
        &without_wifi_driver_ssids,
        &[
            "ssid",
            "SSID",
            "hostname",
            "hostName",
            "pid",
            "pgid",
            "USB_serial",
            "usb_serial",
            "USB-serial",
        ],
    );
    let without_local_paths = redact_local_paths(&without_operational_tokens);
    redact_http_metadata(&without_local_paths)
}

pub(crate) fn redact_json_scalar_fields(text: &str, fields: &[&str]) -> String {
    fields.iter().fold(text.to_owned(), |sanitized, field| {
        redact_json_scalar_field(&sanitized, field)
    })
}

pub(crate) fn redact_json_scalar_field(text: &str, field: &str) -> String {
    let pattern = format!("\"{field}\"");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let Some(relative_start) = text[index..].find(&pattern) else {
            output.push_str(&text[index..]);
            break;
        };
        let field_start = index + relative_start;
        let field_end = field_start + pattern.len();
        output.push_str(&text[index..field_end]);
        let mut cursor = field_end;
        while text
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if text.as_bytes().get(cursor) != Some(&b':') {
            index = field_end;
            continue;
        }
        cursor += 1;
        while text
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if text.as_bytes().get(cursor) == Some(&b'"') {
            index = field_end;
            continue;
        }
        output.push_str(&text[field_end..cursor]);
        output.push_str("\"[redacted]\"");
        while let Some(byte) = text.as_bytes().get(cursor) {
            if matches!(byte, b',' | b'}' | b']') || byte.is_ascii_whitespace() {
                break;
            }
            cursor += 1;
        }
        index = cursor;
    }
    output
}

pub(crate) fn redact_local_paths(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let is_unix_path = ["/Users/", "/home/", "/dev/cu", "/dev/tty"]
            .iter()
            .any(|prefix| rest.starts_with(prefix));
        let is_windows_path = rest.as_bytes().get(1) == Some(&b':')
            && rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
            && matches!(rest.as_bytes().get(2), Some(b'\\'));
        if is_unix_path || is_windows_path {
            output.push_str("[redacted-path]");
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if character.is_whitespace() || matches!(character, '"' | '\'' | ',' | '}') {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }
        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }
    output
}

pub(crate) fn redact_http_metadata(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let line_without_protocol = if let Some(index) = line.find("HTTP/") {
            let protocol_end = line[index..]
                .find(char::is_whitespace)
                .map(|end| index + end)
                .unwrap_or(line.len());
            format!("{}[redacted-http]{}", &line[..index], &line[protocol_end..])
        } else {
            line.to_owned()
        };
        if line_without_protocol
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("host:")
        {
            let leading = line_without_protocol.len() - line_without_protocol.trim_start().len();
            let newline = if line_without_protocol.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            output.push_str(&line_without_protocol[..leading]);
            output.push_str("Host: [redacted]");
            output.push_str(newline);
        } else {
            output.push_str(&line_without_protocol);
        }
    }
    output
}

pub(crate) fn redact_wifi_driver_connected_ssids(text: &str) -> String {
    const MARKER: &str = "wifi:connected with ";
    const AID_DELIMITER: &str = ", aid =";

    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let Some(relative_start) = text[index..].find(MARKER) else {
            output.push_str(&text[index..]);
            break;
        };

        let marker_start = index + relative_start;
        let ssid_start = marker_start + MARKER.len();
        output.push_str(&text[index..ssid_start]);
        output.push_str("[redacted-ssid]");

        let remaining = &text[ssid_start..];
        let relative_end = remaining
            .find(AID_DELIMITER)
            .or_else(|| remaining.find('\n'))
            .unwrap_or(remaining.len());
        index = ssid_start + relative_end;
    }

    output
}

pub(crate) fn redact_json_string_fields(text: &str, fields: &[&str]) -> String {
    fields.iter().fold(text.to_owned(), |sanitized, field| {
        redact_json_string_field(&sanitized, field)
    })
}

pub(crate) fn redact_json_string_field(text: &str, field: &str) -> String {
    let pattern = format!("\"{field}\"");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let Some(relative_start) = text[index..].find(&pattern) else {
            output.push_str(&text[index..]);
            break;
        };

        let field_start = index + relative_start;
        let field_end = field_start + pattern.len();
        output.push_str(&text[index..field_start]);

        let Some((value_open, value_close)) = maybe_json_string_value_bounds(text, field_end)
        else {
            output.push_str(&text[field_start..field_end]);
            index = field_end;
            continue;
        };

        output.push_str(&text[field_start..=value_open]);
        output.push_str("[redacted]");
        output.push('"');
        index = value_close + 1;
    }

    output
}

pub(crate) fn maybe_json_string_value_bounds(
    text: &str,
    after_field: usize,
) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut cursor = after_field;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if bytes.get(cursor) != Some(&b':') {
        return None;
    }
    cursor += 1;

    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if bytes.get(cursor) != Some(&b'"') {
        return None;
    }
    let value_open = cursor;
    cursor += 1;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'"' => return Some((value_open, cursor)),
            _ => cursor += 1,
        }
    }

    None
}

pub(crate) fn redact_urls(text: &str) -> String {
    const URL_SCHEMES: [&str; 4] = ["http://", "https://", "ws://", "wss://"];

    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if let Some(scheme) = URL_SCHEMES.iter().find(|scheme| rest.starts_with(**scheme)) {
            output.push_str("[redacted-url]");
            index += scheme.len();
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if is_url_delimiter(character) {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }

        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

pub(crate) fn is_url_delimiter(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '"' | '\'' | '<' | '>' | ')' | '(' | '[' | ']' | '{' | '}'
        )
}

pub(crate) fn redact_ipv4_addresses(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
                index += 1;
            }
            let token = &text[start..index];
            if is_ipv4_address(token) {
                output.push_str("[redacted-ip]");
            } else {
                output.push_str(token);
            }
            continue;
        }

        let character = text[index..].chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

pub(crate) fn is_ipv4_address(token: &str) -> bool {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        !part.is_empty()
            && part.len() <= 3
            && part.chars().all(|character| character.is_ascii_digit())
            && part.parse::<u8>().is_ok()
    })
}

pub(crate) fn redact_mac_addresses(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < bytes.len() {
        if is_mac_address_at(bytes, index) {
            output.push_str("[redacted-mac]");
            index += 17;
            continue;
        }

        let character = text[index..].chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

pub(crate) fn is_mac_address_at(bytes: &[u8], index: usize) -> bool {
    if index + 17 > bytes.len() {
        return false;
    }

    if index > 0 && bytes[index - 1].is_ascii_hexdigit() {
        return false;
    }

    if index + 17 < bytes.len() && bytes[index + 17].is_ascii_hexdigit() {
        return false;
    }

    for offset in 0..17 {
        let byte = bytes[index + offset];
        if matches!(offset, 2 | 5 | 8 | 11 | 14) {
            if byte != b':' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

pub(crate) fn redact_key_value_tokens(text: &str, keys: &[&str]) -> String {
    keys.iter().fold(text.to_owned(), |sanitized, key| {
        redact_key_value_token(&sanitized, key)
    })
}

pub(crate) fn redact_key_value_token(text: &str, key: &str) -> String {
    let pattern = format!("{key}=");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with(&pattern) {
            output.push_str(&pattern);
            output.push_str("[redacted]");
            index += pattern.len();
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if character.is_whitespace() {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }

        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}
