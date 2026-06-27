//! ESP-IDF HTTP shell for the Phase 05 AxeOS API route table.

use std::ffi::CStr;
use std::net::Ipv4Addr;
use std::ptr;

use bitaxe_api::{
    asic_settings_from_snapshot, block_found_dismiss_plan, empty_statistics_response,
    identify_plan, log_download_headers, maybe_origin_ip_from_header, pause_mining_plan,
    phase05_routes, plan_http_access, plan_websocket_upgrade, restart_plan, resume_mining_plan,
    scoreboard_response, system_info_from_snapshot, unknown_api_route_response,
    unsupported_update_response, BlockFoundNotificationState, CommandPlan, HttpAccessDecision,
    IdentifyMode, PublicHttpResponse, RouteAccessInput, SettingsPatchPublicError,
    WebSocketRouteKind, WebSocketUpgradeDecision,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::sys;
use serde::Serialize;

use crate::runtime_snapshot::collect_api_snapshot;

type ApiRequest<'request, 'connection> = Request<&'request mut EspHttpConnection<'connection>>;

const API_WS_PATH: &[u8] = b"/api/ws\0";
const API_WS_LIVE_PATH: &[u8] = b"/api/ws/live\0";
const APPLICATION_JSON_CSTR: &[u8] = b"application/json\0";
const ORIGIN_HEADER: &[u8] = b"Origin\0";
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
    log::info!(
        "axeos_api_route_shell=started registered_routes={}",
        phase05_routes().len()
    );

    core::mem::forget(server);
    Ok(())
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
        log::warn!("axeos_settings_patch=deferred reason=task2_persistence_adapter_pending");
        send_text_error(request, 400, SettingsPatchPublicError::WrongApiInput.body())
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
        request
            .into_response(200, Some("OK"), &response_headers)?
            .write_all(b"")?;
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
        log::info!("axeos_command_effect=deferred effect={:?}", plan.effect);
        send_json(request, &plan.response)
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
        maybe_origin_ip: origin_ip_from_raw(request),
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
        return sys::ESP_OK;
    }

    match plan_websocket_upgrade(access_input_from_raw(request), route) {
        WebSocketUpgradeDecision::Accept(plan) => {
            log::info!("axeos_websocket_upgrade=accepted route={:?}", plan.route);
            sys::ESP_OK
        }
        WebSocketUpgradeDecision::Reject(response) => {
            log::warn!("axeos_websocket_upgrade=rejected route={route:?}");
            send_raw_public_response(request, response)
        }
    }
}

fn origin_ip_from_raw(request: *mut sys::httpd_req_t) -> Option<Ipv4Addr> {
    let mut buffer = [0; 128];
    let result = unsafe {
        sys::httpd_req_get_hdr_value_str(
            request,
            ORIGIN_HEADER.as_ptr().cast(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };
    if result != sys::ESP_OK {
        return None;
    }

    let origin = unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str().ok()?;
    maybe_origin_ip_from_header(origin)
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

fn uptime_millis() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000
}
