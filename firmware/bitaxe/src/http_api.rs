//! ESP-IDF HTTP shell for the Phase 05 AxeOS API route table.

use std::ffi::{CStr, CString};
use std::net::Ipv4Addr;
use std::ptr;
use std::time::Duration;

use bitaxe_api::{
    asic_settings_from_snapshot, block_found_dismiss_plan, empty_statistics_response,
    execute_settings_persistence_plan, identify_plan, log_download_headers,
    origin_gate_from_header, pause_mining_plan, phase07_route_report, plan_http_access,
    plan_settings_patch_body, plan_settings_patch_body_size, plan_update_request,
    plan_websocket_upgrade, restart_plan, resume_mining_plan, scoreboard_response,
    system_info_from_snapshot, unknown_api_route_response, unsupported_update_response,
    CommandEffect, CommandPlan, HttpAccessDecision, IdentifyModeEffect, OriginGate,
    PublicHttpResponse, RouteAccessInput, SettingsPatchBodyDecision, SettingsPatchPublicError,
    SettingsPersistenceEffect, SettingsPersistencePlan, SettingsPublicResponse,
    UpdateRequestDecision, UpdateRequestInput, UpdateRouteKind, WebSocketRouteKind,
    WebSocketUpgradeDecision, LIVE_TELEMETRY_CADENCE_MS,
};
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{Configuration, EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::sys;
use serde::Serialize;

use crate::filesystem::FilesystemStatus;
use crate::ota_update::{FirmwareOtaApplyResult, FirmwareOtaStatus};
use crate::runtime_snapshot::{
    apply_block_found_dismiss_command, apply_identify_mode_command, apply_mining_activity_command,
    block_found_notification_state, collect_api_snapshot, identify_mode, mining_runtime_state,
};
use crate::{log_buffer, network_stack, settings_adapter, static_files, websocket_api};

type ApiRequest<'request, 'connection> = Request<&'request mut EspHttpConnection<'connection>>;

const API_WS_ROUTE: &str = "/api/ws";
const API_WS_LIVE_ROUTE: &str = "/api/ws/live";
const API_WS_PATH: &[u8] = b"/api/ws\0";
const API_WS_LIVE_PATH: &[u8] = b"/api/ws/live\0";
const CONNECTION_HEADER: &[u8] = b"Connection\0";
const APPLICATION_JSON_CSTR: &[u8] = b"application/json\0";
const ORIGIN_HEADER: &[u8] = b"Origin\0";
const ORIGIN_HEADER_BUFFER_BYTES: usize = 128;
const TEXT_PLAIN_CSTR: &[u8] = b"text/plain\0";
const HTTPD_401: &[u8] = b"401 Unauthorized\0";
const UPGRADE_HEADER: &[u8] = b"Upgrade\0";
const UPDATE_AP_MODE_REJECTION_BODY: &str = "Not allowed in AP mode";
const WEBSOCKET_UPGRADE_REQUIRED_BODY: &str = "WebSocket upgrade required";
const LIVE_TELEMETRY_THREAD_STACK_BYTES: usize = 16 * 1024;

/// Starts the HTTP route shell and intentionally leaks the server so ESP-IDF's
/// server task keeps running for the lifetime of the firmware process.
pub fn start_http_api(filesystem_status: FilesystemStatus) -> anyhow::Result<()> {
    network_stack::initialize()?;

    let config = Configuration {
        stack_size: 8192,
        max_open_sockets: 7,
        max_uri_handlers: 32,
        max_resp_headers: 8,
        uri_match_wildcard: true,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    if let Err(error) = settings_adapter::initialize_current_settings_snapshot() {
        log::warn!("axeos_settings_snapshot=startup_refresh_failed error={error}");
    }

    register_http_handlers(&mut server, filesystem_status)?;
    start_live_telemetry_cadence_task(server.handle())?;
    let route_report = phase07_route_report();
    log::info!(
        "axeos_api_route_shell=started manifest_routes={} firmware_update_routes={} otawww_gap_routes={} recovery_routes={} static_file_routes={}",
        route_report.total_routes,
        route_report.firmware_update_routes,
        route_report.otawww_gap_routes,
        route_report.recovery_routes,
        route_report.static_file_routes
    );

    core::mem::forget(server);
    Ok(())
}

fn start_live_telemetry_cadence_task(server: sys::httpd_handle_t) -> anyhow::Result<()> {
    let server_addr = server as usize;
    std::thread::Builder::new()
        .name("axeos-live-ws".to_owned())
        .stack_size(LIVE_TELEMETRY_THREAD_STACK_BYTES)
        .spawn(move || live_telemetry_cadence_loop(server_addr))?;
    Ok(())
}

fn live_telemetry_cadence_loop(server_addr: usize) {
    let server = server_addr as sys::httpd_handle_t;
    loop {
        std::thread::sleep(Duration::from_millis(LIVE_TELEMETRY_CADENCE_MS));
        broadcast_live_telemetry_cadence(server);
        broadcast_raw_log_chunks(server);
        prune_stale_websocket_sessions(server);
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

fn broadcast_raw_log_chunks(server: sys::httpd_handle_t) {
    let buffer = log_buffer::retained_log_buffer();
    for chunk in websocket_api::raw_log_chunks(&buffer) {
        broadcast_websocket_text_frame(server, WebSocketRouteKind::Logs, &chunk);
    }
}

fn prune_stale_websocket_sessions(server: sys::httpd_handle_t) {
    ping_websocket_route(server, WebSocketRouteKind::Logs);
    ping_websocket_route(server, WebSocketRouteKind::LiveTelemetry);
}

fn ping_websocket_route(server: sys::httpd_handle_t, route: WebSocketRouteKind) {
    for session in websocket_api::client_sessions(route) {
        let result = send_websocket_ping_frame_async(server, session);
        if result == sys::ESP_OK {
            continue;
        }

        log::warn!(
            "axeos_websocket_ping=unregistering_stale route={route:?} session={session} error={result}"
        );
        websocket_api::unregister_client(session);
    }
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

fn register_http_handlers(
    server: &mut EspHttpServer<'static>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    static_files::register_recovery(server, filesystem_status)?;
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
    server.fn_handler("/api/system/OTA", Method::Post, handle_firmware_ota_update)?;
    server.fn_handler("/api/system/OTAWWW", Method::Post, handle_otawww_update_gap)?;
    register_websocket_handlers(server)?;
    server.fn_handler("/api/*", Method::Get, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Post, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Patch, handle_unknown_api_route)?;
    static_files::register_static(server, filesystem_status)?;

    Ok(())
}

fn register_websocket_handlers(server: &mut EspHttpServer<'static>) -> anyhow::Result<()> {
    register_websocket_handler(server, API_WS_PATH, API_WS_ROUTE, websocket_logs_handler)?;
    register_websocket_handler(
        server,
        API_WS_LIVE_PATH,
        API_WS_LIVE_ROUTE,
        websocket_live_handler,
    )?;
    Ok(())
}

fn register_websocket_handler(
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
        let snapshot = settings_adapter::current_settings_snapshot();
        let plan = SettingsPersistencePlan::from_accepted_patch(&snapshot, accepted);
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
        settings_adapter::apply_persisted_settings_writes(plan.writes());
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
    let mining = mining_runtime_state();
    handle_command(request, resume_mining_plan(&mining))
}

fn handle_restart<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, restart_plan())
}

fn handle_identify<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, identify_plan(identify_mode()))
}

fn handle_block_found_dismiss<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(
        request,
        block_found_dismiss_plan(block_found_notification_state()),
    )
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

fn handle_firmware_ota_update<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    let decision = plan_update_request(UpdateRequestInput {
        route: UpdateRouteKind::FirmwareOta,
        access: access_input(&mut request),
    });

    let plan = match decision {
        UpdateRequestDecision::AcceptFirmwareOta(plan) => plan,
        UpdateRequestDecision::Reject(response) => return send_public_response(request, response),
        UpdateRequestDecision::OtaWwwGap(gap) => {
            return send_public_response(request, gap.public_response);
        }
    };

    debug_assert_eq!(
        plan.success_response.body,
        "Firmware update complete, rebooting now!"
    );
    debug_assert_eq!(
        plan.validation_error_response.body,
        "Validation / Activation Error"
    );

    let raw_request = (*request.connection()).handle();
    let result = crate::ota_update::stream_firmware_ota(raw_request, record_firmware_ota_status);
    match result {
        FirmwareOtaApplyResult::Complete { bytes_written } => {
            log::info!("firmware_ota_update=complete bytes_written={bytes_written}");
            send_public_response(request, plan.success_response)?;
            schedule_firmware_ota_restart();
            Ok(())
        }
        FirmwareOtaApplyResult::ProtocolError { code } => {
            log::warn!("firmware_ota_update=protocol_error code={code}");
            send_text_error(request, 500, "Protocol Error")
        }
        FirmwareOtaApplyResult::WriteError { esp_err } => {
            log::warn!("firmware_ota_update=write_error esp_err={esp_err}");
            send_text_error(request, 500, "Write Error")
        }
        FirmwareOtaApplyResult::ValidationError { esp_err } => {
            log::warn!("firmware_ota_update=validation_error esp_err={esp_err}");
            send_public_response(request, plan.validation_error_response)
        }
    }
}

fn handle_otawww_update_gap<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    match plan_update_request(UpdateRequestInput {
        route: UpdateRouteKind::AxeOsStaticOtaWww,
        access: access_input(&mut request),
    }) {
        UpdateRequestDecision::Reject(response) => {
            if response.body == UPDATE_AP_MODE_REJECTION_BODY {
                log::warn!("otawww_update=rejected reason=ap_mode");
            }
            send_public_response(request, response)
        }
        UpdateRequestDecision::OtaWwwGap(gap) => {
            debug_assert_eq!(gap.public_response.body, "Wrong API input");
            log::warn!(
                "otawww_update=gap reason=interruption_evidence_missing owner={}",
                gap.owner
            );
            send_public_response(request, gap.public_response)
        }
        UpdateRequestDecision::AcceptFirmwareOta(_) => {
            log::warn!("otawww_update=gap reason=unexpected_firmware_ota_decision");
            send_public_response(request, unsupported_update_response())
        }
    }
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
    let path = request_path_without_query(request.connection().uri()).to_owned();
    let input = access_input(&mut request);
    match plan_http_access(input) {
        HttpAccessDecision::Allow => handler(request),
        HttpAccessDecision::Deny(response) => {
            log_access_denied("http", &path, input);
            send_public_response(request, response)
        }
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
            log::warn!("axeos_access_gate_peer_ip=unavailable reason=no_socket");
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
            log::warn!("axeos_access_gate_peer_ip=unavailable reason=getpeername_failed");
            return None;
        }

        Some(peer_ipv4_from_s_addr(addr.sin_addr.s_addr))
    }
}

fn peer_ipv4_from_s_addr(raw_addr: u32) -> Ipv4Addr {
    let network_order_ip = Ipv4Addr::from(u32::from_be(raw_addr));
    if is_rfc1918_ipv4(network_order_ip) {
        return network_order_ip;
    }

    let host_order_ip = Ipv4Addr::from(raw_addr);
    if is_rfc1918_ipv4(host_order_ip) {
        log::warn!(
            "axeos_access_gate_peer_ip_byte_order=host_order raw=0x{raw_addr:08x} network_order_ip={network_order_ip} host_order_ip={host_order_ip}"
        );
        return host_order_ip;
    }

    network_order_ip
}

fn is_rfc1918_ipv4(ip: Ipv4Addr) -> bool {
    let [first, second, _, _] = ip.octets();
    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
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
                websocket_api::WebSocketRegisterOutcome::Accepted { active_clients } => {
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
                        websocket_api::unregister_client(session);
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

fn is_websocket_upgrade_request(request: *mut sys::httpd_req_t) -> bool {
    raw_header_matches(request, UPGRADE_HEADER, |value| {
        value.eq_ignore_ascii_case("websocket")
    }) && raw_header_matches(request, CONNECTION_HEADER, |value| {
        value
            .split(',')
            .any(|part| part.trim().eq_ignore_ascii_case("upgrade"))
    })
}

fn raw_header_matches(
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

fn request_path_without_query(uri: &str) -> &str {
    uri.split_once('?')
        .map(|(path, _query)| path)
        .unwrap_or(uri)
}

fn websocket_route_path(route: WebSocketRouteKind) -> &'static str {
    match route {
        WebSocketRouteKind::Logs => "/api/ws",
        WebSocketRouteKind::LiveTelemetry => "/api/ws/live",
    }
}

fn log_access_denied(kind: &str, path: &str, input: RouteAccessInput) {
    log::warn!(
        "axeos_access_gate=denied kind={kind} path={path} ap_mode_enabled={} request_ip={} origin={:?}",
        input.ap_mode_enabled,
        input.request_ip,
        input.origin
    );
}

fn handle_websocket_frame(request: *mut sys::httpd_req_t) -> sys::esp_err_t {
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

fn unregister_request_websocket_session(request: *mut sys::httpd_req_t, reason: &str) {
    let session = unsafe { sys::httpd_req_to_sockfd(request) };
    if session < 0 {
        log::warn!("axeos_websocket_session=unregister_skipped reason={reason} session=missing");
        return;
    }

    websocket_api::unregister_client(session);
    log::info!("axeos_websocket_session=unregistered reason={reason} session={session}");
}

fn send_websocket_connect_frames(
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
            let snapshot = collect_api_snapshot();
            let Ok(current) = serde_json::to_value(system_info_from_snapshot(&snapshot)) else {
                return sys::ESP_FAIL;
            };
            let Some(frame) = websocket_api::live_connect_frame(current) else {
                return sys::ESP_FAIL;
            };
            let Ok(body) = serde_json::to_string(&frame) else {
                return sys::ESP_FAIL;
            };
            send_websocket_text_frame(request, &body)
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

fn send_websocket_ping_frame_async(server: sys::httpd_handle_t, session: i32) -> sys::esp_err_t {
    let mut frame = sys::httpd_ws_frame_t {
        final_: true,
        fragmented: false,
        type_: sys::httpd_ws_type_t_HTTPD_WS_TYPE_PING,
        payload: ptr::null_mut(),
        len: 0,
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
            apply_mining_activity_command(effect);
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
                apply_identify_mode_command(effect);
                log::info!("axeos_command_effect=identify_enable duration_ms={duration_ms}");
            }
            IdentifyModeEffect::Disable => {
                apply_identify_mode_command(effect);
                log::info!("axeos_command_effect=identify_disable");
            }
        },
        CommandEffect::BlockFoundDismiss(effect) => {
            apply_block_found_dismiss_command(effect);
            log::info!(
                "axeos_command_effect=block_found_dismiss block_found={} show_new_block={}",
                effect.next_state.block_found,
                effect.next_state.show_new_block
            );
        }
    }
}

fn record_firmware_ota_status(status: FirmwareOtaStatus) {
    let text = status.status_text();
    log::info!("firmware_ota_status={text}");
    log_buffer::append_runtime_log_line(&format!("firmware_ota_status={text}"));
}

fn schedule_firmware_ota_restart() {
    let result = std::thread::Builder::new()
        .name("firmware-ota-restart".to_owned())
        .spawn(|| {
            std::thread::sleep(Duration::from_millis(1000));
            log::info!("firmware_ota_update=restart_now");
            unsafe { sys::esp_restart() };
        });

    if let Err(error) = result {
        log::warn!("firmware_ota_update=restart_thread_failed error={error}");
        unsafe { sys::esp_restart() };
    }
}

fn uptime_millis() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000
}
