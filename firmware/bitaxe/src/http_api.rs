//! ESP-IDF HTTP shell for the Phase 05 AxeOS API route table.

use std::ffi::{CStr, CString};
use std::net::Ipv4Addr;
use std::ptr;
use std::time::Duration;

use bitaxe_api::{
    asic_settings_from_snapshot, block_found_dismiss_plan, empty_statistics_response,
    execute_settings_persistence_plan, identify_plan, log_download_headers,
    origin_gate_from_header, pause_mining_plan, phase05_routes, plan_http_access,
    plan_settings_patch_body, plan_settings_patch_body_size, plan_websocket_upgrade, restart_plan,
    resume_mining_plan, scoreboard_response, system_info_from_snapshot, unknown_api_route_response,
    unsupported_update_response, BlockFoundNotificationState, CommandEffect, CommandPlan,
    HttpAccessDecision, IdentifyMode, IdentifyModeEffect, OriginGate, PublicHttpResponse,
    RouteAccessInput, SettingsPatchBodyDecision, SettingsPatchPublicError,
    SettingsPersistenceEffect, SettingsPersistencePlan, SettingsPublicResponse, WebSocketRouteKind,
    WebSocketUpgradeDecision, LIVE_TELEMETRY_CADENCE_MS,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::sys;
use serde::Serialize;

use crate::runtime_snapshot::collect_api_snapshot;
use crate::{log_buffer, settings_adapter, websocket_api};

type ApiRequest<'request, 'connection> = Request<&'request mut EspHttpConnection<'connection>>;

const API_WS_PATH: &[u8] = b"/api/ws\0";
const API_WS_LIVE_PATH: &[u8] = b"/api/ws/live\0";
const APPLICATION_JSON_CSTR: &[u8] = b"application/json\0";
const ORIGIN_HEADER: &[u8] = b"Origin\0";
const ORIGIN_HEADER_BUFFER_BYTES: usize = 128;
const TEXT_PLAIN_CSTR: &[u8] = b"text/plain\0";
const HTTPD_401: &[u8] = b"401 Unauthorized\0";

/// Starts the HTTP route shell and intentionally leaks the server so ESP-IDF's
/// server task keeps running for the lifetime of the firmware process.
pub fn start_http_api() -> anyhow::Result<()> {
    let config = Configuration {
        stack_size: 8192,
        max_open_sockets: 8,
        max_uri_handlers: 32,
        max_resp_headers: 8,
        uri_match_wildcard: true,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    register_http_handlers(&mut server)?;
    start_live_telemetry_cadence_task(server.handle())?;
    log::info!(
        "axeos_api_route_shell=started registered_routes={}",
        phase05_routes().len()
    );

    core::mem::forget(server);
    Ok(())
}

fn start_live_telemetry_cadence_task(server: sys::httpd_handle_t) -> anyhow::Result<()> {
    let server_addr = server as usize;
    std::thread::Builder::new()
        .name("axeos-live-ws".to_owned())
        .spawn(move || live_telemetry_cadence_loop(server_addr))?;
    Ok(())
}

fn live_telemetry_cadence_loop(server_addr: usize) {
    let server = server_addr as sys::httpd_handle_t;
    loop {
        std::thread::sleep(Duration::from_millis(LIVE_TELEMETRY_CADENCE_MS));
        broadcast_live_telemetry_cadence(server);
    }
}

fn broadcast_live_telemetry_cadence(server: sys::httpd_handle_t) {
    let snapshot = collect_api_snapshot();
    let Ok(current) = serde_json::to_value(system_info_from_snapshot(&snapshot)) else {
        log::warn!("axeos_websocket_live_cadence=skipped reason=serialize_current");
        return;
    };
    let Some(frame) = websocket_api::live_cadence_frame(current) else {
        return;
    };
    let Ok(body) = serde_json::to_string(&frame) else {
        log::warn!("axeos_websocket_live_cadence=skipped reason=serialize_frame");
        return;
    };

    broadcast_websocket_text_frame(server, WebSocketRouteKind::LiveTelemetry, &body);
}

fn broadcast_websocket_text_frame(
    server: sys::httpd_handle_t,
    route: WebSocketRouteKind,
    body: &str,
) {
    for session in websocket_api::client_sessions(route) {
        let result = send_websocket_text_frame_async(server, session, body);
        if result == sys::ESP_OK {
            continue;
        }

        log::warn!(
            "axeos_websocket_broadcast=unregistering_stale route={route:?} session={session} error={result}"
        );
        websocket_api::unregister_client(session);
    }
}

fn register_http_handlers(server: &mut EspHttpServer<'static>) -> anyhow::Result<()> {
    server.fn_handler("/api/system/info", Method::Get, handle_system_info)?;
    server.fn_handler("/api/system", Method::Patch, handle_settings_patch)?;
    server.fn_handler("/api/system/logs", Method::Get, handle_logs_download)?;
    server.fn_handler("/api/system/asic", Method::Get, handle_asic_settings)?;
    server.fn_handler("/api/system/statistics", Method::Get, handle_statistics)?;
    server.fn_handler("/api/system/scoreboard", Method::Get, handle_scoreboard)?;
    server.fn_handler("/api/system/pause", Method::Post, handle_pause)?;
    server.fn_handler("/api/system/resume", Method::Post, handle_resume)?;
    server.fn_handler("/api/system/restart", Method::Post, handle_restart)?;
    server.fn_handler("/api/system/identify", Method::Post, handle_identify)?;
    server.fn_handler(
        "/api/system/blockFound/dismiss",
        Method::Post,
        handle_block_found_dismiss,
    )?;
    server.fn_handler("/api/system/OTA", Method::Post, handle_unsupported_update)?;
    server.fn_handler(
        "/api/system/OTAWWW",
        Method::Post,
        handle_unsupported_update,
    )?;
    register_websocket_handlers(server)?;
    server.fn_handler("/api/*", Method::Get, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Post, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Patch, handle_unknown_api_route)?;

    Ok(())
}

fn register_websocket_handlers(server: &mut EspHttpServer<'static>) -> anyhow::Result<()> {
    register_websocket_handler(server, API_WS_PATH, websocket_logs_handler)?;
    register_websocket_handler(server, API_WS_LIVE_PATH, websocket_live_handler)?;
    Ok(())
}

fn register_websocket_handler(
    server: &mut EspHttpServer<'static>,
    path: &'static [u8],
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
        "failed to register websocket route {}: esp_err={result}",
        String::from_utf8_lossy(&path[..path.len().saturating_sub(1)])
    ))
}

fn handle_system_info<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let snapshot = collect_api_snapshot();
        send_json(request, &system_info_from_snapshot(&snapshot))
    })
}

fn handle_settings_patch<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let mut request = request;
        let body_len = request_body_len(&mut request);
        if let SettingsPatchBodyDecision::Reject(response) = plan_settings_patch_body_size(body_len)
        {
            return send_public_response(request, response);
        }

        let body = match read_body_string(&mut request, body_len) {
            Ok(body) => body,
            Err(public_error) => {
                return send_text_error(request, 400, public_error.body());
            }
        };
        let accepted = match plan_settings_patch_body(&body) {
            Ok(accepted) => accepted,
            Err(error) => {
                log::warn!("axeos_settings_patch=rejected reason={:?}", error.reason());
                return send_text_error(request, 400, error.public_error().body());
            }
        };

        let snapshot = settings_adapter::current_settings_snapshot();
        let plan = SettingsPersistencePlan::from_accepted_patch(&snapshot, accepted);
        let mut adapter = match settings_adapter::FirmwareSettingsAdapter::open() {
            Ok(adapter) => adapter,
            Err(error) => {
                log::warn!("axeos_settings_patch=adapter_open_failed error={error}");
                return send_text_error(
                    request,
                    400,
                    SettingsPatchPublicError::WrongApiInput.body(),
                );
            }
        };
        let success = match execute_settings_persistence_plan(&plan, &mut adapter) {
            Ok(success) => success,
            Err(error) => {
                log::warn!(
                    "axeos_settings_patch=persistence_failed reason={:?} steps={:?}",
                    error.reason(),
                    error.completed_steps()
                );
                return send_text_error(request, 400, error.public_error().body());
            }
        };
        let effects = success.effects().to_vec();
        send_settings_response(request, success.public_response())?;
        apply_settings_effects(&effects);
        Ok(())
    })
}

fn handle_logs_download<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let headers = log_download_headers();
        let response_headers = [
            ("Content-Type", headers.content_type),
            ("Content-Disposition", headers.content_disposition),
        ];
        let mut response = request.into_response(200, Some("OK"), &response_headers)?;
        for chunk in log_buffer::download_chunks() {
            response.write_all(chunk.as_bytes())?;
        }
        Ok(())
    })
}

fn handle_asic_settings<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let snapshot = collect_api_snapshot();
        send_json(request, &asic_settings_from_snapshot(&snapshot))
    })
}

fn handle_statistics<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let timestamp_ms = uptime_millis();
        send_json(request, &empty_statistics_response(timestamp_ms, None))
    })
}

fn handle_scoreboard<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_json(request, &scoreboard_response(&[]))
    })
}

fn handle_pause<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, pause_mining_plan())
}

fn handle_resume<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    let snapshot = collect_api_snapshot();
    handle_command(request, resume_mining_plan(&snapshot.mining))
}

fn handle_restart<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, restart_plan())
}

fn handle_identify<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, identify_plan(IdentifyMode::Inactive))
}

fn handle_block_found_dismiss<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    let state = BlockFoundNotificationState {
        block_found: 0,
        show_new_block: false,
    };
    handle_command(request, block_found_dismiss_plan(state))
}

fn handle_command<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
    plan: CommandPlan,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let effect = plan.effect;
        send_json(request, &plan.response)?;
        apply_command_effect(effect);
        Ok(())
    })
}

fn handle_unsupported_update<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_public_response(request, unsupported_update_response())
    })
}

fn handle_unknown_api_route<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_public_response(request, unknown_api_route_response())
    })
}

fn handle_with_access_gate<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
    handler: impl FnOnce(ApiRequest<'request, 'connection>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    match plan_http_access(access_input(&mut request)) {
        HttpAccessDecision::Allow => handler(request),
        HttpAccessDecision::Deny(response) => send_public_response(request, response),
    }
}

fn access_input<'request, 'connection>(
    request: &mut ApiRequest<'request, 'connection>,
) -> RouteAccessInput {
    let raw_request = (*request.connection()).handle();
    access_input_from_raw(raw_request)
}

fn access_input_from_raw(request: *mut sys::httpd_req_t) -> RouteAccessInput {
    let request_ip = peer_ipv4(request).unwrap_or(Ipv4Addr::UNSPECIFIED);

    RouteAccessInput {
        ap_mode_enabled: ap_mode_enabled(),
        request_ip,
        origin: origin_gate_from_raw(request),
    }
}

fn ap_mode_enabled() -> bool {
    let mut mode = 0;
    let result = unsafe { sys::esp_wifi_get_mode(&mut mode) };
    result == sys::ESP_OK
        && matches!(
            mode,
            sys::wifi_mode_t_WIFI_MODE_AP | sys::wifi_mode_t_WIFI_MODE_APSTA
        )
}

fn peer_ipv4(request: *mut sys::httpd_req_t) -> Option<Ipv4Addr> {
    unsafe {
        let sockfd = sys::httpd_req_to_sockfd(request);
        if sockfd == -1 {
            return None;
        }

        let mut addr = sys::sockaddr_in {
            sin_len: core::mem::size_of::<sys::sockaddr_in>() as _,
            sin_family: sys::AF_INET as _,
            ..Default::default()
        };
        let mut addr_len = core::mem::size_of::<sys::sockaddr_in>() as sys::socklen_t;

        if sys::lwip_getpeername(
            sockfd,
            &mut addr as *mut _ as *mut sys::sockaddr,
            &mut addr_len,
        ) != sys::ESP_OK
        {
            return None;
        }

        Some(Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)))
    }
}

unsafe extern "C" fn websocket_logs_handler(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    handle_websocket_upgrade(request, WebSocketRouteKind::Logs)
}

unsafe extern "C" fn websocket_live_handler(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    handle_websocket_upgrade(request, WebSocketRouteKind::LiveTelemetry)
}

fn handle_websocket_upgrade(
    request: *mut sys::httpd_req_t,
    route: WebSocketRouteKind,
) -> sys::esp_err_t {
    if unsafe { (*request).method } != sys::http_method_HTTP_GET as i32 {
        return handle_websocket_frame(request);
    }

    match plan_websocket_upgrade(access_input_from_raw(request), route) {
        WebSocketUpgradeDecision::Accept(plan) => {
            let session = unsafe { sys::httpd_req_to_sockfd(request) };
            match websocket_api::register_client(session, plan.route) {
                websocket_api::WebSocketRegisterOutcome::Accepted { active_clients } => {
                    log::info!(
                        "axeos_websocket_upgrade=accepted route={:?} active_clients={active_clients}",
                        plan.route
                    );
                    send_websocket_connect_frames(request, plan.route)
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
            log::warn!("axeos_websocket_upgrade=rejected route={route:?}");
            send_raw_public_response(request, response)
        }
    }
}

fn handle_websocket_frame(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t::default();
    let result = unsafe { sys::httpd_ws_recv_frame(request, &mut frame, 0) };
    if result != sys::ESP_OK {
        return result;
    }

    if frame.type_ == sys::httpd_ws_type_t_HTTPD_WS_TYPE_CLOSE {
        let session = unsafe { sys::httpd_req_to_sockfd(request) };
        websocket_api::unregister_client(session);
    }

    sys::ESP_OK
}

fn send_websocket_connect_frames(
    request: *mut sys::httpd_req_t,
    route: WebSocketRouteKind,
) -> sys::esp_err_t {
    match route {
        WebSocketRouteKind::Logs => {
            let buffer = log_buffer::retained_log_buffer();
            let _ = websocket_api::raw_log_chunks(&buffer);
            sys::ESP_OK
        }
        WebSocketRouteKind::LiveTelemetry => {
            let snapshot = collect_api_snapshot();
            let Ok(current) = serde_json::to_value(system_info_from_snapshot(&snapshot)) else {
                return sys::ESP_FAIL;
            };
            let Some(frame) = websocket_api::live_connect_frame(current.clone()) else {
                return sys::ESP_FAIL;
            };
            let Ok(body) = serde_json::to_string(&frame) else {
                return sys::ESP_FAIL;
            };
            let result = send_websocket_text_frame(request, &body);
            if result == sys::ESP_OK {
                let _ = websocket_api::live_cadence_frame(current);
            }
            result
        }
    }
}

fn send_websocket_text_frame(request: *mut sys::httpd_req_t, body: &str) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t {
        final_: true,
        fragmented: false,
        type_: sys::httpd_ws_type_t_HTTPD_WS_TYPE_TEXT,
        payload: body.as_ptr() as *mut u8,
        len: body.len(),
    };
    unsafe { sys::httpd_ws_send_frame(request, &mut frame) }
}

fn send_websocket_text_frame_async(
    server: sys::httpd_handle_t,
    session: i32,
    body: &str,
) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t {
        final_: true,
        fragmented: false,
        type_: sys::httpd_ws_type_t_HTTPD_WS_TYPE_TEXT,
        payload: body.as_ptr() as *mut u8,
        len: body.len(),
    };
    unsafe { sys::httpd_ws_send_frame_async(server, session, &mut frame) }
}

fn origin_gate_from_raw(request: *mut sys::httpd_req_t) -> OriginGate {
    let mut buffer = [0; ORIGIN_HEADER_BUFFER_BYTES];
    let result = unsafe {
        sys::httpd_req_get_hdr_value_str(
            request,
            ORIGIN_HEADER.as_ptr().cast(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };
    match result {
        sys::ESP_OK => {}
        sys::ESP_ERR_NOT_FOUND => return OriginGate::Missing,
        sys::ESP_ERR_HTTPD_RESULT_TRUNC => return OriginGate::Invalid,
        _ => return OriginGate::Invalid,
    }

    let Ok(origin) = (unsafe { CStr::from_ptr(buffer.as_ptr()) }).to_str() else {
        return OriginGate::Invalid;
    };

    origin_gate_from_header(origin)
}

fn send_raw_public_response(
    request: *mut sys::httpd_req_t,
    response: PublicHttpResponse,
) -> sys::esp_err_t {
    if set_raw_status(request, response.status) != sys::ESP_OK {
        return sys::ESP_FAIL;
    }
    if set_raw_content_type(request, response.content_type) != sys::ESP_OK {
        return sys::ESP_FAIL;
    }

    unsafe {
        sys::httpd_resp_send(
            request,
            response.body.as_ptr().cast(),
            response.body.len() as isize,
        )
    }
}

fn set_raw_status(request: *mut sys::httpd_req_t, status: u16) -> sys::esp_err_t {
    let status_ptr = match status {
        400 => sys::HTTPD_400.as_ptr(),
        401 => HTTPD_401.as_ptr(),
        404 => sys::HTTPD_404.as_ptr(),
        500 => sys::HTTPD_500.as_ptr(),
        _ => sys::HTTPD_500.as_ptr(),
    };
    unsafe { sys::httpd_resp_set_status(request, status_ptr.cast()) }
}

fn set_raw_content_type(
    request: *mut sys::httpd_req_t,
    maybe_content_type: Option<&'static str>,
) -> sys::esp_err_t {
    let Some(content_type) = maybe_content_type else {
        return sys::ESP_OK;
    };
    let content_type_ptr = match content_type {
        "application/json" => APPLICATION_JSON_CSTR.as_ptr(),
        "text/plain" => TEXT_PLAIN_CSTR.as_ptr(),
        _ => TEXT_PLAIN_CSTR.as_ptr(),
    };
    unsafe { sys::httpd_resp_set_type(request, content_type_ptr.cast()) }
}

fn send_json<'request, 'connection, T: Serialize>(
    request: ApiRequest<'request, 'connection>,
    value: &T,
) -> anyhow::Result<()> {
    let body = serde_json::to_vec(value)?;
    request
        .into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
        .write_all(&body)?;
    Ok(())
}

fn send_settings_response<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
    response: SettingsPublicResponse,
) -> anyhow::Result<()> {
    match response {
        SettingsPublicResponse::EmptySuccess => {
            request
                .into_response(200, Some("OK"), &[])?
                .write_all(b"")?;
            Ok(())
        }
        SettingsPublicResponse::Error(error) => send_text_error(request, 400, error.body()),
    }
}

fn send_text_error<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
    status: u16,
    body: &'static str,
) -> anyhow::Result<()> {
    request
        .into_response(status, None, &[("Content-Type", "text/plain")])?
        .write_all(body.as_bytes())?;
    Ok(())
}

fn send_public_response(
    request: ApiRequest<'_, '_>,
    response: PublicHttpResponse,
) -> anyhow::Result<()> {
    let maybe_content_type = response.content_type;
    if let Some(content_type) = maybe_content_type {
        request
            .into_response(response.status, None, &[("Content-Type", content_type)])?
            .write_all(response.body.as_bytes())?;
        return Ok(());
    }

    request
        .into_status_response(response.status)?
        .write_all(response.body.as_bytes())?;
    Ok(())
}

fn request_body_len(request: &mut ApiRequest<'_, '_>) -> usize {
    let raw_request = (*request.connection()).handle();
    unsafe { (*raw_request).content_len }
}

fn read_body_string(
    request: &mut ApiRequest<'_, '_>,
    body_len: usize,
) -> Result<String, SettingsPatchPublicError> {
    let mut body = vec![0; body_len];
    let mut offset = 0;
    while offset < body_len {
        let read = request
            .read(&mut body[offset..])
            .map_err(|_| SettingsPatchPublicError::WrongApiInput)?;
        if read == 0 {
            return Err(SettingsPatchPublicError::WrongApiInput);
        }
        offset += read;
    }

    String::from_utf8(body).map_err(|_| SettingsPatchPublicError::InvalidJson)
}

fn apply_settings_effects(effects: &[SettingsPersistenceEffect]) {
    for effect in effects {
        match effect {
            SettingsPersistenceEffect::BestEffortApplyHostname { hostname } => {
                apply_hostname_effect(hostname);
            }
        }
    }
}

fn apply_hostname_effect(hostname: &str) {
    const NETIF_KEYS: [&[u8]; 2] = [b"WIFI_STA_DEF\0", b"WIFI_AP_DEF\0"];

    let Ok(hostname_cstr) = CString::new(hostname) else {
        log::warn!("axeos_settings_effect=hostname_failed reason=interior_nul");
        return;
    };

    let mut applied = false;
    for key in NETIF_KEYS {
        let netif = unsafe { sys::esp_netif_get_handle_from_ifkey(key.as_ptr().cast()) };
        if netif.is_null() {
            continue;
        }

        let result = unsafe { sys::esp_netif_set_hostname(netif, hostname_cstr.as_ptr()) };
        if result == sys::ESP_OK {
            applied = true;
            continue;
        }

        log::warn!("axeos_settings_effect=hostname_failed esp_err={result}");
    }

    if applied {
        log::info!("axeos_settings_effect=hostname_applied");
        return;
    }

    log::warn!("axeos_settings_effect=hostname_skipped reason=netif_unavailable");
}

fn apply_command_effect(effect: CommandEffect) {
    match effect {
        CommandEffect::MiningActivity(effect) => {
            log::info!(
                "axeos_command_effect=mining_activity next_activity={:?}",
                effect.next_activity
            );
        }
        CommandEffect::RestartAfterResponse => {
            log::info!("axeos_command_effect=restart_after_response");
            unsafe { sys::esp_restart() };
        }
        CommandEffect::Identify(effect) => match effect {
            IdentifyModeEffect::Enable { duration_ms } => {
                log::info!("axeos_command_effect=identify_enable duration_ms={duration_ms}");
            }
            IdentifyModeEffect::Disable => {
                log::info!("axeos_command_effect=identify_disable");
            }
        },
        CommandEffect::BlockFoundDismiss(effect) => {
            log::info!(
                "axeos_command_effect=block_found_dismiss block_found={} show_new_block={}",
                effect.next_state.block_found,
                effect.next_state.show_new_block
            );
        }
    }
}

fn uptime_millis() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000
}
