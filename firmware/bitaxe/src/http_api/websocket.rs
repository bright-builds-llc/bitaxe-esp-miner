use super::*;

enum LiveCadenceIssueError {
    SerializeFrame,
}

struct WebSocketSendFailure {
    lease: websocket_api::WebSocketClientLease,
    error: sys::esp_err_t,
}

struct QueuedWebSocketFrame {
    server: sys::httpd_handle_t,
    lease: websocket_api::WebSocketClientLease,
    frame_type: sys::httpd_ws_type_t,
    payload: Box<[u8]>,
}

/// Starts the HTTP route shell and intentionally leaks the server so ESP-IDF's
pub(super) fn start_live_telemetry_cadence_task(server: sys::httpd_handle_t) -> anyhow::Result<()> {
    let server_addr = server as usize;
    std::thread::Builder::new()
        .name("axeos-live-ws".to_owned())
        .stack_size(LIVE_TELEMETRY_THREAD_STACK_BYTES)
        .spawn(move || live_telemetry_cadence_loop(server_addr))?;
    Ok(())
}

pub(super) fn live_telemetry_cadence_loop(server_addr: usize) {
    let server = server_addr as sys::httpd_handle_t;
    loop {
        std::thread::sleep(Duration::from_millis(LIVE_TELEMETRY_CADENCE_MS));
        broadcast_live_telemetry_cadence(server);
        broadcast_raw_log_chunks(server);
        prune_stale_websocket_sessions(server);
    }
}

pub(super) fn broadcast_live_telemetry_cadence(server: sys::httpd_handle_t) {
    let result =
        publish_projected_live_telemetry_payload(crate::runtime_uptime::millis(), |current| {
            let Some(frame) = websocket_api::live_cadence_frame(current) else {
                return Ok(Vec::new());
            };
            let body =
                serde_json::to_string(&frame).map_err(|_| LiveCadenceIssueError::SerializeFrame)?;
            Ok(broadcast_websocket_text_frame(
                server,
                WebSocketRouteKind::LiveTelemetry,
                &body,
            ))
        });
    match result {
        Ok(failures) => handle_websocket_send_failures(WebSocketRouteKind::LiveTelemetry, failures),
        Err(OperatorSnapshotPublishError::Issuance {
            source: LiveCadenceIssueError::SerializeFrame,
            ..
        }) => log::warn!("axeos_websocket_live_cadence=skipped reason=serialize_frame"),
        Err(_) => {
            log::warn!("axeos_websocket_live_cadence=skipped reason=snapshot_publication")
        }
    }
}

pub(super) fn broadcast_raw_log_chunks(server: sys::httpd_handle_t) {
    let buffer = log_buffer::retained_log_buffer();
    for chunk in websocket_api::raw_log_chunks(&buffer) {
        let failures = broadcast_websocket_text_frame(server, WebSocketRouteKind::Logs, &chunk);
        handle_websocket_send_failures(WebSocketRouteKind::Logs, failures);
    }
}

pub(super) fn prune_stale_websocket_sessions(server: sys::httpd_handle_t) {
    ping_websocket_route(server, WebSocketRouteKind::Logs);
    ping_websocket_route(server, WebSocketRouteKind::LiveTelemetry);
}

pub(super) fn ping_websocket_route(server: sys::httpd_handle_t, route: WebSocketRouteKind) {
    for lease in websocket_api::client_leases(route) {
        let result = send_websocket_ping_frame_async(server, lease);
        if result == sys::ESP_OK {
            continue;
        }

        log::warn!(
            "axeos_websocket_ping=unregistering_stale route={route:?} session={} error={result}",
            lease.session()
        );
        websocket_api::unregister_if_current(lease);
    }
}

fn broadcast_websocket_text_frame(
    server: sys::httpd_handle_t,
    route: WebSocketRouteKind,
    body: &str,
) -> Vec<WebSocketSendFailure> {
    let mut failures = Vec::new();
    for lease in websocket_api::client_leases(route) {
        let result = send_websocket_text_frame_async(server, lease, body);
        if result == sys::ESP_OK {
            continue;
        }
        failures.push(WebSocketSendFailure {
            lease,
            error: result,
        });
    }
    failures
}

fn handle_websocket_send_failures(route: WebSocketRouteKind, failures: Vec<WebSocketSendFailure>) {
    for failure in failures {
        log::warn!(
            "axeos_websocket_broadcast=unregistering_stale route={route:?} session={} error={}",
            failure.lease.session(),
            failure.error
        );
        websocket_api::unregister_if_current(failure.lease);
    }
}

pub(super) fn register_websocket_handlers(
    server: &mut EspHttpServer<'static>,
) -> anyhow::Result<()> {
    register_websocket_handler(server, API_WS_PATH, API_WS_ROUTE, websocket_logs_handler)?;
    register_websocket_handler(
        server,
        API_WS_LIVE_PATH,
        API_WS_LIVE_ROUTE,
        websocket_live_handler,
    )?;
    Ok(())
}

unsafe extern "C" fn websocket_logs_handler(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    handle_websocket_upgrade(request, WebSocketRouteKind::Logs)
}

unsafe extern "C" fn websocket_live_handler(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    handle_websocket_upgrade(request, WebSocketRouteKind::LiveTelemetry)
}

pub(super) fn register_websocket_handler(
    server: &mut EspHttpServer<'static>,
    path: &'static [u8],
    display_path: &'static str,
    handler: unsafe extern "C" fn(*mut sys::httpd_req_t) -> sys::esp_err_t,
) -> anyhow::Result<()> {
    let uri = sys::httpd_uri_t {
        uri: path.as_ptr().cast(),
        method: sys::http_method_HTTP_GET,
        handler: Some(handler),
        user_ctx: ptr::null_mut(),
        is_websocket: true,
        ..Default::default()
    };
    let result = unsafe { sys::httpd_register_uri_handler(server.handle(), &uri) };
    if result == sys::ESP_OK {
        return Ok(());
    }

    Err(anyhow::anyhow!(
        "failed to register websocket route {display_path}: esp_err={result}"
    ))
}

pub(super) fn handle_websocket_upgrade(
    request: *mut sys::httpd_req_t,
    route: WebSocketRouteKind,
) -> sys::esp_err_t {
    if unsafe { (*request).method } != sys::http_method_HTTP_GET as i32 {
        return handle_websocket_frame(request);
    }

    if !is_websocket_upgrade_request(request) {
        log::warn!("axeos_websocket_upgrade=rejected route={route:?} reason=no_upgrade");
        return send_raw_public_response(request, websocket_upgrade_required_response());
    }

    let input = access_input_from_raw(request);
    match plan_websocket_upgrade(input, route) {
        WebSocketUpgradeDecision::Accept(plan) => {
            let session = unsafe { sys::httpd_req_to_sockfd(request) };
            if session < 0 {
                log::warn!("axeos_websocket_upgrade=rejected route={route:?} reason=no_session");
                return sys::ESP_FAIL;
            }

            match websocket_api::register_client(session, plan.route) {
                websocket_api::WebSocketRegisterOutcome::Accepted {
                    active_clients,
                    lease,
                } => {
                    install_websocket_session_context(request, lease);
                    log::info!(
                        "axeos_websocket_upgrade=accepted route={:?} active_clients={active_clients}",
                        plan.route
                    );
                    let result = send_websocket_connect_frames(request, plan.route);
                    if result != sys::ESP_OK {
                        log::warn!(
                            "axeos_websocket_upgrade=connect_send_failed route={:?} session={session} error={result}",
                            plan.route
                        );
                        websocket_api::unregister_if_current(lease);
                    }

                    result
                }
                websocket_api::WebSocketRegisterOutcome::RejectedMaxClients { max_clients } => {
                    log::warn!(
                        "axeos_websocket_upgrade=rejected route={route:?} reason=max_clients max_clients={max_clients}"
                    );
                    send_raw_public_response(
                        request,
                        PublicHttpResponse {
                            status: 400,
                            body: SettingsPatchPublicError::WrongApiInput.body(),
                            content_type: Some("text/plain"),
                        },
                    )
                }
            }
        }
        WebSocketUpgradeDecision::Reject(response) => {
            log_access_denied("websocket", websocket_route_path(route), input);
            log::warn!("axeos_websocket_upgrade=rejected route={route:?}");
            send_raw_public_response(request, response)
        }
    }
}

pub(super) fn is_websocket_upgrade_request(request: *mut sys::httpd_req_t) -> bool {
    raw_header_matches(request, UPGRADE_HEADER, |value| {
        value.eq_ignore_ascii_case("websocket")
    }) && raw_header_matches(request, CONNECTION_HEADER, |value| {
        value
            .split(',')
            .any(|part| part.trim().eq_ignore_ascii_case("upgrade"))
    })
}

pub(super) fn raw_header_matches(
    request: *mut sys::httpd_req_t,
    name: &[u8],
    predicate: impl FnOnce(&str) -> bool,
) -> bool {
    let mut buffer = [0; ORIGIN_HEADER_BUFFER_BYTES];
    let result = unsafe {
        sys::httpd_req_get_hdr_value_str(
            request,
            name.as_ptr().cast(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };
    if result != sys::ESP_OK {
        return false;
    }

    let Ok(value) = (unsafe { CStr::from_ptr(buffer.as_ptr()) }).to_str() else {
        return false;
    };

    predicate(value)
}

const fn websocket_upgrade_required_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 400,
        body: WEBSOCKET_UPGRADE_REQUIRED_BODY,
        content_type: Some("text/plain"),
    }
}

pub(super) fn request_path_without_query(uri: &str) -> &str {
    uri.split_once('?')
        .map(|(path, _query)| path)
        .unwrap_or(uri)
}

pub(super) fn websocket_route_path(route: WebSocketRouteKind) -> &'static str {
    match route {
        WebSocketRouteKind::Logs => "/api/ws",
        WebSocketRouteKind::LiveTelemetry => "/api/ws/live",
    }
}

pub(super) fn log_access_denied(kind: &str, path: &str, input: RouteAccessInput) {
    log::warn!(
        "axeos_access_gate=denied kind={kind} path={path} ap_mode_enabled={} request_ip={} origin={:?}",
        input.ap_mode_enabled,
        input.request_ip,
        input.origin
    );
}

pub(super) fn handle_websocket_frame(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t::default();
    let result = unsafe { sys::httpd_ws_recv_frame(request, &mut frame, 0) };
    if result != sys::ESP_OK {
        unregister_request_websocket_session(request, "recv_error");
        return result;
    }

    if frame.type_ == sys::httpd_ws_type_t_HTTPD_WS_TYPE_CLOSE {
        unregister_request_websocket_session(request, "close_frame");
    }

    sys::ESP_OK
}

pub(super) fn unregister_request_websocket_session(request: *mut sys::httpd_req_t, reason: &str) {
    let maybe_lease = websocket_lease_from_request(request);
    let Some(lease) = maybe_lease else {
        log::warn!("axeos_websocket_session=unregister_skipped reason={reason} session=missing");
        return;
    };

    websocket_api::unregister_if_current(lease);
    log::info!(
        "axeos_websocket_session=unregistered reason={reason} session={}",
        lease.session()
    );
}

pub(super) fn install_websocket_session_context(
    request: *mut sys::httpd_req_t,
    lease: websocket_api::WebSocketClientLease,
) {
    let lease_ptr = Box::into_raw(Box::new(lease));
    unsafe {
        (*request).sess_ctx = lease_ptr.cast::<c_void>();
        (*request).free_ctx = Some(free_websocket_session_context);
        (*request).ignore_sess_ctx_changes = false;
    }
}

pub(super) fn websocket_lease_from_request(
    request: *mut sys::httpd_req_t,
) -> Option<websocket_api::WebSocketClientLease> {
    let context = unsafe { (*request).sess_ctx };
    if context.is_null() {
        return None;
    }

    Some(unsafe { *context.cast::<websocket_api::WebSocketClientLease>() })
}

unsafe extern "C" fn free_websocket_session_context(context: *mut c_void) {
    if context.is_null() {
        return;
    }
    let lease = unsafe { Box::from_raw(context.cast::<websocket_api::WebSocketClientLease>()) };
    websocket_api::unregister_if_current(*lease);
}

pub(super) fn send_websocket_connect_frames(
    request: *mut sys::httpd_req_t,
    route: WebSocketRouteKind,
) -> sys::esp_err_t {
    match route {
        WebSocketRouteKind::Logs => {
            let buffer = log_buffer::retained_log_buffer();
            websocket_api::log_client_connected(&buffer);
            log_buffer::append_runtime_log_line("axeos_websocket_logs=connected");
            let buffer = log_buffer::retained_log_buffer();
            for chunk in websocket_api::raw_log_chunks(&buffer) {
                let result = send_websocket_text_frame(request, &chunk);
                if result != sys::ESP_OK {
                    return result;
                }
            }
            sys::ESP_OK
        }
        WebSocketRouteKind::LiveTelemetry => {
            match publish_projected_live_telemetry_payload(
                crate::runtime_uptime::millis(),
                |current| {
                    let Some(frame) = websocket_api::live_connect_frame(current) else {
                        return Err(sys::ESP_FAIL);
                    };
                    let body = serde_json::to_string(&frame).map_err(|_| sys::ESP_FAIL)?;
                    let result = send_websocket_text_frame(request, &body);
                    if result == sys::ESP_OK {
                        return Ok(result);
                    }
                    Err(result)
                },
            ) {
                Ok(result) => result,
                Err(OperatorSnapshotPublishError::Issuance { source, .. }) => source,
                Err(_) => sys::ESP_FAIL,
            }
        }
    }
}

pub(super) fn send_websocket_text_frame(
    request: *mut sys::httpd_req_t,
    body: &str,
) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t {
        final_: true,
        fragmented: false,
        type_: sys::httpd_ws_type_t_HTTPD_WS_TYPE_TEXT,
        payload: body.as_ptr() as *mut u8,
        len: body.len(),
    };
    unsafe { sys::httpd_ws_send_frame(request, &mut frame) }
}

pub(super) fn send_websocket_text_frame_async(
    server: sys::httpd_handle_t,
    lease: websocket_api::WebSocketClientLease,
    body: &str,
) -> sys::esp_err_t {
    queue_websocket_frame(
        server,
        lease,
        sys::httpd_ws_type_t_HTTPD_WS_TYPE_TEXT,
        body.as_bytes(),
    )
}

pub(super) fn send_websocket_ping_frame_async(
    server: sys::httpd_handle_t,
    lease: websocket_api::WebSocketClientLease,
) -> sys::esp_err_t {
    queue_websocket_frame(server, lease, sys::httpd_ws_type_t_HTTPD_WS_TYPE_PING, &[])
}

pub(super) fn queue_websocket_frame(
    server: sys::httpd_handle_t,
    lease: websocket_api::WebSocketClientLease,
    frame_type: sys::httpd_ws_type_t,
    payload: &[u8],
) -> sys::esp_err_t {
    let queued = Box::new(QueuedWebSocketFrame {
        server,
        lease,
        frame_type,
        payload: payload.to_vec().into_boxed_slice(),
    });
    let queued_ptr = Box::into_raw(queued);
    let result = unsafe {
        sys::httpd_queue_work(
            server,
            Some(send_queued_websocket_frame),
            queued_ptr.cast::<c_void>(),
        )
    };
    if result != sys::ESP_OK {
        drop(unsafe { Box::from_raw(queued_ptr) });
    }
    result
}

unsafe extern "C" fn send_queued_websocket_frame(argument: *mut c_void) {
    if argument.is_null() {
        return;
    }
    let mut queued = unsafe { Box::from_raw(argument.cast::<QueuedWebSocketFrame>()) };
    if !websocket_api::is_current(queued.lease) {
        return;
    }

    let session_is_websocket = unsafe {
        sys::httpd_ws_get_fd_info(queued.server, queued.lease.session())
            == sys::httpd_ws_client_info_t_HTTPD_WS_CLIENT_WEBSOCKET
    };
    if !session_is_websocket {
        websocket_api::unregister_if_current(queued.lease);
        return;
    }

    let payload = if queued.payload.is_empty() {
        ptr::null_mut()
    } else {
        queued.payload.as_mut_ptr()
    };
    let mut frame = sys::httpd_ws_frame_t {
        final_: true,
        fragmented: false,
        type_: queued.frame_type,
        payload,
        len: queued.payload.len(),
    };
    let result = unsafe {
        sys::httpd_ws_send_frame_async(queued.server, queued.lease.session(), &mut frame)
    };
    if result != sys::ESP_OK {
        websocket_api::unregister_if_current(queued.lease);
    }
}
