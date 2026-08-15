//! ESP-IDF HTTP shell for the Phase 05 AxeOS API route table.

use std::ffi::{c_void, CStr, CString};
use std::net::Ipv4Addr;
use std::ptr;
use std::sync::OnceLock;
use std::time::Duration;

use bitaxe_api::{
    asic_settings_from_snapshot, block_found_dismiss_plan, decide_v12_settings_body,
    execute_settings_persistence_plan, identify_plan, log_download_headers, normalize_peer_ipv4,
    origin_gate_from_header, pause_mining_plan, phase07_route_report, plan_http_access,
    plan_settings_patch_body, plan_settings_patch_body_size, plan_theme_post, plan_update_request,
    plan_websocket_upgrade, restart_plan, resume_mining_plan, theme_settings_from_snapshot,
    unknown_api_route_response, unsupported_update_response, CommandEffect, CommandPlan,
    HttpAccessDecision, IdentifyModeEffect, OperatorSnapshotPublishError, OriginGate,
    PeerIpv4Normalization, PublicHttpResponse, RouteAccessInput, SettingsPatchBodyDecision,
    SettingsPatchFailureReason, SettingsPatchPublicError, SettingsPersistenceEffect,
    SettingsPersistenceFailure, SettingsPersistencePlan, SettingsPublicResponse, ThemePostResponse,
    UpdateRequestDecision, UpdateRequestInput, UpdateRouteKind, V12SettingsChange,
    V12SettingsDecision, V12SettingsExclusionReason, WebSocketRouteKind, WebSocketUpgradeDecision,
    LIVE_TELEMETRY_CADENCE_MS,
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
    apply_block_found_dismiss_command, apply_identify_mode_command,
    apply_mining_operator_intent_command, block_found_notification_state, collect_api_snapshot,
    command_status_wire, identify_mode, projected_scoreboard, projected_statistics,
    publish_projected_live_telemetry_payload, publish_projected_system_info,
    record_restart_command,
};
use crate::{
    log_buffer, network_stack, settings_adapter, static_files, websocket_api, wifi_adapter,
};

mod access;
mod deferred_effect_queue;
mod deferred_effects;
mod handlers;
mod response;
mod settings;
mod theme;
mod updates;
mod websocket;

use access::*;
use deferred_effects::*;
use handlers::*;
use response::*;
use settings::*;
use theme::*;
use updates::*;
use websocket::*;

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
const HTTP_SERVER_TASK_STACK_BYTES: usize = 16 * 1024;
const LIVE_TELEMETRY_THREAD_STACK_BYTES: usize = 16 * 1024;
const DEFERRED_EFFECT_QUEUE_CAPACITY: usize = 8;
const DEFERRED_EFFECT_THREAD_STACK_BYTES: usize = 8 * 1024;
const RESTART_POST_RESPONSE_DELAY_MS: u64 = 1_000;
const SETTINGS_EFFECTS_POST_RESPONSE_DELAY_MS: u64 = 100;

pub fn start_http_api(filesystem_status: FilesystemStatus) -> anyhow::Result<()> {
    network_stack::initialize()?;
    initialize_deferred_effect_worker()?;

    let config = Configuration {
        stack_size: HTTP_SERVER_TASK_STACK_BYTES,
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

fn register_http_handlers(
    server: &mut EspHttpServer<'static>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    static_files::register_recovery(server, filesystem_status)?;
    server.fn_handler("/api/system/info", Method::Get, handle_system_info)?;
    server.fn_handler(
        "/api/system/command-status",
        Method::Get,
        handle_command_status,
    )?;
    server.fn_handler("/api/system/wifi/scan", Method::Get, handle_wifi_scan)?;
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
    server.fn_handler("/api/theme", Method::Get, handle_theme_get)?;
    server.fn_handler("/api/theme", Method::Post, handle_theme_post)?;
    register_websocket_handlers(server)?;
    server.fn_handler("/api/*", Method::Get, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Post, handle_unknown_api_route)?;
    server.fn_handler("/api/*", Method::Patch, handle_unknown_api_route)?;
    static_files::register_static(server, filesystem_status)?;

    Ok(())
}
