//! ESP-IDF HTTP static file and recovery handlers.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/http_server/recovery_page.html`

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use bitaxe_api::{
    resolve_static_request, FilesystemAvailability, RecoveryFallback, RedirectToRoot,
    RejectPathTraversal, ServeRecovery, ServeStatic, StaticFileCatalog, StaticRequest,
    StaticRouteDecision, CONTENT_ENCODING_HEADER, STATIC_REDIRECT_BODY,
};
use esp_idf_svc::http::server::{EspHttpConnection, EspHttpServer, Request};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;

use crate::filesystem::FilesystemStatus;

type StaticHttpRequest<'request, 'connection> =
    Request<&'request mut EspHttpConnection<'connection>>;

const RECOVERY_PAGE_HTML: &str = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>AxeOS Recovery</title></head>
<body><h1>AxeOS Recovery</h1><p>Please upload www.bin to recover AxeOS.</p></body>
</html>
"#;
const TEXT_PLAIN: &str = "text/plain";
const WWW_BASE_PATH: &str = "/www";

/// Registers the explicit embedded recovery route.
pub fn register_recovery(
    server: &mut EspHttpServer<'static>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    server.fn_handler("/recovery", Method::Get, move |request| {
        handle_recovery(request, filesystem_status)
    })?;
    Ok(())
}

/// Registers the static wildcard route after all API routes.
pub fn register_static(
    server: &mut EspHttpServer<'static>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    if matches!(filesystem_status, FilesystemStatus::Unavailable { .. }) {
        log::warn!("static_files=spiffs_unavailable recovery=available");
    }

    server.fn_handler("/*", Method::Get, move |request| {
        handle_static(request, filesystem_status)
    })?;
    Ok(())
}

fn handle_recovery<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    let decision = resolve_static_request(
        StaticRequest {
            path: "/recovery",
            filesystem: filesystem_availability(filesystem_status),
        },
        StaticFileCatalog::new(&[]),
    );

    send_static_decision(request, decision)
}

fn handle_static<'request, 'connection>(
    mut request: StaticHttpRequest<'request, 'connection>,
    filesystem_status: FilesystemStatus,
) -> anyhow::Result<()> {
    let path = request_path(&mut request);
    let catalog_entries = match filesystem_status {
        FilesystemStatus::Available { .. } => collect_static_catalog(),
        FilesystemStatus::Unavailable { .. } => Vec::new(),
    };
    let catalog_refs: Vec<&str> = catalog_entries.iter().map(String::as_str).collect();
    let decision = resolve_static_request(
        StaticRequest {
            path: &path,
            filesystem: filesystem_availability(filesystem_status),
        },
        StaticFileCatalog::new(&catalog_refs),
    );

    send_static_decision(request, decision)
}

fn send_static_decision<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    decision: StaticRouteDecision,
) -> anyhow::Result<()> {
    match decision {
        StaticRouteDecision::ServeStatic(file) => send_file(request, file),
        StaticRouteDecision::ServeRecovery(recovery) => send_recovery(request, recovery),
        StaticRouteDecision::RedirectToRoot(redirect) => send_redirect(request, redirect),
        StaticRouteDecision::RejectPathTraversal(rejection) => send_rejection(request, rejection),
        StaticRouteDecision::RecoveryFallback(fallback) => {
            send_recovery_fallback(request, fallback)
        }
    }
}

fn send_file<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    file: ServeStatic,
) -> anyhow::Result<()> {
    let content_type = content_type_for(&file.path);
    let mut headers = vec![("Content-Type", content_type)];
    if let Some(cache_control) = file.cache_control {
        debug_assert_eq!(cache_control, "max-age=2592000");
        headers.push(("Cache-Control", cache_control));
    }
    if let Some(content_encoding) = file.content_encoding {
        debug_assert_eq!(CONTENT_ENCODING_HEADER, "Content-Encoding");
        debug_assert_eq!(content_encoding, "gzip");
        headers.push((CONTENT_ENCODING_HEADER, content_encoding));
    }

    let disk_path = format!("{WWW_BASE_PATH}{}", file.path);
    let mut static_file = File::open(&disk_path)?;
    let mut response = request.into_response(200, Some("OK"), &headers)?;
    let mut buffer = [0_u8; 1024];
    loop {
        let read = static_file.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }

        response.write_all(&buffer[..read])?;
    }
}

fn send_recovery<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    recovery: ServeRecovery,
) -> anyhow::Result<()> {
    request
        .into_response(200, Some("OK"), &[("Content-Type", recovery.content_type)])?
        .write_all(RECOVERY_PAGE_HTML.as_bytes())?;
    Ok(())
}

fn send_recovery_fallback<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    fallback: RecoveryFallback,
) -> anyhow::Result<()> {
    log::warn!(
        "static_files=spiffs_unavailable recovery=available reason={}",
        fallback.reason
    );
    send_recovery(request, fallback.recovery)
}

fn send_redirect<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    redirect: RedirectToRoot,
) -> anyhow::Result<()> {
    debug_assert_eq!(redirect.body, "Redirect to the captive portal");
    request
        .into_response(
            redirect.status,
            Some("Temporary Redirect"),
            &[
                ("Location", redirect.location),
                ("Content-Type", TEXT_PLAIN),
            ],
        )?
        .write_all(STATIC_REDIRECT_BODY.as_bytes())?;
    Ok(())
}

fn send_rejection<'request, 'connection>(
    request: StaticHttpRequest<'request, 'connection>,
    rejection: RejectPathTraversal,
) -> anyhow::Result<()> {
    debug_assert_eq!(rejection.body, "Wrong API input");
    request
        .into_response(rejection.status, None, &[("Content-Type", TEXT_PLAIN)])?
        .write_all(rejection.body.as_bytes())?;
    Ok(())
}

fn request_path(request: &mut StaticHttpRequest<'_, '_>) -> String {
    let uri = request.connection().uri();
    let Some((path, _query)) = uri.split_once('?') else {
        return uri.to_owned();
    };

    path.to_owned()
}

fn filesystem_availability(status: FilesystemStatus) -> FilesystemAvailability {
    match status {
        FilesystemStatus::Available { .. } => FilesystemAvailability::Available,
        FilesystemStatus::Unavailable { .. } => FilesystemAvailability::Unavailable,
    }
}

fn collect_static_catalog() -> Vec<String> {
    let mut files = Vec::new();
    collect_static_catalog_from(Path::new(WWW_BASE_PATH), "", &mut files);
    files
}

fn collect_static_catalog_from(directory: &Path, prefix: &str, files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        log::warn!(
            "static_files=catalog_unavailable path={}",
            directory.display()
        );
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let catalog_path = if prefix.is_empty() {
            format!("/{name}")
        } else {
            format!("{prefix}/{name}")
        };

        if path.is_dir() {
            collect_static_catalog_from(&path, &catalog_path, files);
            continue;
        }

        files.push(catalog_path);
    }
}

fn content_type_for(path: &str) -> &'static str {
    let path = path.strip_suffix(".gz").unwrap_or(path);
    if path.ends_with(".html") {
        return "text/html";
    }
    if path.ends_with(".css") {
        return "text/css";
    }
    if path.ends_with(".js") {
        return "application/javascript";
    }
    if path.ends_with(".json") {
        return "application/json";
    }
    if path.ends_with(".svg") {
        return "image/svg+xml";
    }
    if path.ends_with(".png") {
        return "image/png";
    }
    if path.ends_with(".ico") {
        return "image/x-icon";
    }
    if path.ends_with(".woff2") {
        return "font/woff2";
    }

    "application/octet-stream"
}
