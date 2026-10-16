//! Every blocking send on the HTTP server task is timed by `http_send_timing`, so the
//! soak observer sees each slow write (`task-http-server-task-stall-diagnostics`).

const SOURCES: [(&str, &str); 3] = [
    ("response.rs", include_str!("http_api/response.rs")),
    ("handlers.rs", include_str!("http_api/handlers.rs")),
    ("websocket.rs", include_str!("http_api/websocket.rs")),
];
const SEND_CALLS: [&str; 4] = [
    ".write_all(",
    "sys::httpd_resp_send(",
    "sys::httpd_ws_send_frame(",
    "sys::httpd_ws_send_frame_async(",
];
/// How far back the enclosing `timed` call may start (one wrapped statement).
const STATEMENT_WINDOW: usize = 320;

#[test]
fn every_server_task_send_runs_inside_a_timing_wrapper() {
    // Arrange
    let mut untimed = Vec::new();

    // Act
    for (file, source) in SOURCES {
        for call in SEND_CALLS {
            for (index, _) in source.match_indices(call) {
                let start = index.saturating_sub(STATEMENT_WINDOW);
                let start = (start..=index).find(|&i| source.is_char_boundary(i)).unwrap_or(index);
                if !source[start..index].contains("http_send_timing::timed") {
                    untimed.push(format!("{file}: {call}"));
                }
            }
        }
    }

    // Assert
    assert!(untimed.is_empty(), "untimed sends: {untimed:?}");
}

#[test]
fn the_runtime_health_projection_carries_the_send_durations() {
    // Arrange / Act
    let adapter = include_str!("runtime_health_adapter.rs");
    // Assert
    assert!(adapter.contains(".with_http_server_sends(crate::http_send_timing::snapshot())"));
}
