//! No socket or UART write runs inside an operator-snapshot publication: the global
//! publication lock is held through `issue`, so a slow client there would stall every
//! other publication (HTTP, WebSocket cadence, BAP). Publication closures serialize; the
//! callers write after the lock is released.

const HANDLERS_SOURCE: &str = include_str!("http_api/handlers.rs");
const WEBSOCKET_SOURCE: &str = include_str!("http_api/websocket.rs");
const BAP_SOURCE: &str = include_str!("bap_adapter.rs");

/// Each top-level function's text, split at `fn ` declarations at the start of a line.
fn functions(source: &str) -> Vec<&str> {
    let mut starts: Vec<usize> = source
        .match_indices('\n')
        .map(|(index, _)| index + 1)
        .filter(|&start| {
            let line = &source[start..];
            ["fn ", "pub fn ", "pub(super) fn ", "pub(crate) fn ", "unsafe extern \"C\" fn "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect();
    starts.push(source.len());
    starts.windows(2).map(|pair| &source[pair[0]..pair[1]]).collect()
}

fn publishing_functions(source: &str) -> Vec<&str> {
    functions(source)
        .into_iter()
        .filter(|function| function.contains("publish_projected_"))
        .collect()
}

#[test]
fn http_handlers_publish_only_inside_request_free_body_functions() {
    // Arrange
    let publishing = publishing_functions(HANDLERS_SOURCE);

    // Act / Assert
    assert_eq!(publishing.len(), 2, "system info and ASIC settings");
    for function in publishing {
        let signature = function.lines().next().unwrap_or_default();
        assert!(signature.contains("_body() ->"), "{signature}");
        assert!(!function.contains("send_json"), "{signature}");
        assert!(!function.contains("request"), "{signature}");
    }
}

#[test]
fn the_websocket_connect_frame_is_sent_after_its_publication() {
    // Arrange
    let publishing: Vec<&str> = publishing_functions(WEBSOCKET_SOURCE)
        .into_iter()
        .filter(|function| function.contains("publish_projected_live_telemetry_payload("))
        .collect();

    // Act / Assert: the only connect-path publication is the request-free body.
    assert_eq!(publishing.len(), 1);
    assert!(publishing[0].starts_with("fn live_connect_frame_body()"));
    assert!(!publishing[0].contains("send_websocket_text_frame"));
    assert!(!publishing[0].contains("request"));
}

#[test]
fn bap_publications_never_write_to_the_uart() {
    // Arrange
    let publishing = publishing_functions(BAP_SOURCE);

    // Act / Assert
    assert_eq!(publishing.len(), 2, "request snapshot and subscription frames");
    assert!(publishing
        .iter()
        .all(|function| !function.contains("write_frame") && !function.contains("driver")));
}
