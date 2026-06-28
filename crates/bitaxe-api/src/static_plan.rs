//! Pure static file and recovery route decisions for firmware adapters.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `reference/esp-miner/main/filesystem.c`
//! - `reference/esp-miner/main/http_server/recovery_page.html`

/// Cache header value for non-directory static assets.
pub const STATIC_CACHE_CONTROL: &str = "max-age=2592000";
/// Header name used when serving a gzipped static asset variant.
pub const CONTENT_ENCODING_HEADER: &str = "Content-Encoding";
/// Header value used when serving a gzipped static asset variant.
pub const GZIP_CONTENT_ENCODING: &str = "gzip";
/// Upstream-compatible missing-file redirect body.
pub const STATIC_REDIRECT_BODY: &str = "Redirect to the captive portal";

/// Static route request data from the firmware adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticRequest<'a> {
    /// Raw request URI path.
    pub path: &'a str,
    /// SPIFFS/static filesystem availability state.
    pub filesystem: FilesystemAvailability,
}

/// Catalog of static file paths discovered by the firmware adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticFileCatalog<'a> {
    /// Absolute catalog paths rooted at the static base.
    pub files: &'a [&'a str],
}

impl<'a> StaticFileCatalog<'a> {
    /// Creates a static file catalog from adapter-provided paths.
    #[must_use]
    pub const fn new(files: &'a [&'a str]) -> Self {
        Self { files }
    }

    fn contains(self, path: &str) -> bool {
        self.files.contains(&path)
    }
}

/// Static filesystem state visible before route resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemAvailability {
    /// SPIFFS/static filesystem is mounted and available.
    Available,
    /// SPIFFS/static filesystem is unavailable.
    Unavailable,
}

/// Static file response plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeStatic {
    /// Catalog path to open in the firmware adapter.
    pub path: String,
    /// Optional cache header for non-directory assets.
    pub cache_control: Option<&'static str>,
    /// Optional content encoding header value.
    pub content_encoding: Option<&'static str>,
}

/// Embedded recovery response plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServeRecovery {
    /// Source asset for the recovery response.
    pub source: RecoverySource,
    /// Public content type.
    pub content_type: &'static str,
}

/// Recovery asset source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    /// Recovery HTML embedded in the firmware image.
    EmbeddedHtml,
}

/// Missing static asset redirect plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedirectToRoot {
    /// HTTP status code.
    pub status: u16,
    /// Redirect target.
    pub location: &'static str,
    /// Public response body.
    pub body: &'static str,
}

/// Rejected unsafe path plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RejectPathTraversal {
    /// HTTP status code.
    pub status: u16,
    /// Public response body.
    pub body: &'static str,
}

/// Explicit recovery fallback when static files cannot be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryFallback {
    /// Recovery response to serve instead.
    pub recovery: ServeRecovery,
    /// Visible reason for the firmware adapter/status layer.
    pub reason: &'static str,
}

/// Pure static route resolution decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticRouteDecision {
    /// Serve a mounted static file.
    ServeStatic(ServeStatic),
    /// Serve embedded recovery HTML.
    ServeRecovery(ServeRecovery),
    /// Redirect missing static assets to `/`.
    RedirectToRoot(RedirectToRoot),
    /// Reject a path before filesystem access.
    RejectPathTraversal(RejectPathTraversal),
    /// Serve recovery because SPIFFS/static files are unavailable.
    RecoveryFallback(RecoveryFallback),
}

/// Resolves a static or recovery request without reading real files.
#[must_use]
pub fn resolve_static_request(
    request: StaticRequest<'_>,
    catalog: StaticFileCatalog<'_>,
) -> StaticRouteDecision {
    if is_unsafe_path(request.path) {
        return StaticRouteDecision::RejectPathTraversal(reject_path_traversal());
    }

    if request.path == "/recovery" {
        if request.filesystem == FilesystemAvailability::Unavailable {
            return StaticRouteDecision::RecoveryFallback(recovery_fallback());
        }

        return StaticRouteDecision::ServeRecovery(serve_recovery());
    }

    if request.filesystem == FilesystemAvailability::Unavailable {
        return StaticRouteDecision::RecoveryFallback(recovery_fallback());
    }

    let Some(static_path) = maybe_static_path(request.path) else {
        return StaticRouteDecision::RejectPathTraversal(reject_path_traversal());
    };

    let maybe_gzip_path = format!("{}.gz", static_path.path);
    if catalog.contains(&maybe_gzip_path) {
        return StaticRouteDecision::ServeStatic(ServeStatic {
            path: maybe_gzip_path,
            cache_control: cache_control_for(static_path.directory_request),
            content_encoding: Some(GZIP_CONTENT_ENCODING),
        });
    }

    if catalog.contains(&static_path.path) {
        return StaticRouteDecision::ServeStatic(ServeStatic {
            path: static_path.path,
            cache_control: cache_control_for(static_path.directory_request),
            content_encoding: None,
        });
    }

    StaticRouteDecision::RedirectToRoot(redirect_to_root())
}

struct NormalizedStaticPath {
    path: String,
    directory_request: bool,
}

fn maybe_static_path(path: &str) -> Option<NormalizedStaticPath> {
    if path == "/" {
        return Some(NormalizedStaticPath {
            path: "/index.html".to_owned(),
            directory_request: true,
        });
    }

    if path.ends_with('/') {
        return Some(NormalizedStaticPath {
            path: format!("{path}index.html"),
            directory_request: true,
        });
    }

    Some(NormalizedStaticPath {
        path: path.to_owned(),
        directory_request: false,
    })
}

fn is_unsafe_path(path: &str) -> bool {
    if path.is_empty() || !path.starts_with('/') || path.starts_with("//") {
        return true;
    }

    path.contains("..") || path.contains('\\') || path.contains('\0') || path.contains("://")
}

const fn cache_control_for(directory_request: bool) -> Option<&'static str> {
    if directory_request {
        return None;
    }

    Some(STATIC_CACHE_CONTROL)
}

const fn serve_recovery() -> ServeRecovery {
    ServeRecovery {
        source: RecoverySource::EmbeddedHtml,
        content_type: "text/html",
    }
}

const fn recovery_fallback() -> RecoveryFallback {
    RecoveryFallback {
        recovery: serve_recovery(),
        reason: "spiffs-unavailable",
    }
}

const fn redirect_to_root() -> RedirectToRoot {
    RedirectToRoot {
        status: 302,
        location: "/",
        body: STATIC_REDIRECT_BODY,
    }
}

const fn reject_path_traversal() -> RejectPathTraversal {
    RejectPathTraversal {
        status: 400,
        body: "Wrong API input",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_static_request, FilesystemAvailability, RecoverySource, StaticFileCatalog,
        StaticRequest, StaticRouteDecision, CONTENT_ENCODING_HEADER, GZIP_CONTENT_ENCODING,
        STATIC_CACHE_CONTROL, STATIC_REDIRECT_BODY,
    };

    fn available_request(path: &'static str) -> StaticRequest<'static> {
        StaticRequest {
            path,
            filesystem: FilesystemAvailability::Available,
        }
    }

    fn unavailable_request(path: &'static str) -> StaticRequest<'static> {
        StaticRequest {
            path,
            filesystem: FilesystemAvailability::Unavailable,
        }
    }

    fn catalog(paths: &'static [&'static str]) -> StaticFileCatalog<'static> {
        StaticFileCatalog::new(paths)
    }

    #[test]
    fn root_resolves_to_index_html_without_cache_header() {
        // Arrange
        let request = available_request("/");
        let catalog = catalog(&["/index.html"]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::ServeStatic(file) = decision else {
            panic!("root should serve index.html");
        };
        assert_eq!(file.path, "/index.html");
        assert_eq!(file.cache_control, None);
        assert_eq!(file.content_encoding, None);
    }

    #[test]
    fn recovery_serves_embedded_html_when_filesystem_is_available() {
        // Arrange
        let request = available_request("/recovery");
        let catalog = catalog(&[]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::ServeRecovery(recovery) = decision else {
            panic!("recovery route should serve embedded HTML");
        };
        assert_eq!(recovery.source, RecoverySource::EmbeddedHtml);
        assert_eq!(recovery.content_type, "text/html");
    }

    #[test]
    fn recovery_uses_embedded_fallback_when_filesystem_is_unavailable() {
        // Arrange
        let request = unavailable_request("/recovery");
        let catalog = catalog(&[]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::RecoveryFallback(fallback) = decision else {
            panic!("unavailable filesystem should expose recovery fallback");
        };
        assert_eq!(fallback.recovery.source, RecoverySource::EmbeddedHtml);
        assert_eq!(fallback.reason, "spiffs-unavailable");
    }

    #[test]
    fn gz_variant_is_preferred_and_marks_content_encoding() {
        // Arrange
        let request = available_request("/assets/app.js");
        let catalog = catalog(&["/assets/app.js", "/assets/app.js.gz"]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::ServeStatic(file) = decision else {
            panic!("static asset should be served");
        };
        assert_eq!(file.path, "/assets/app.js.gz");
        assert_eq!(file.cache_control, Some(STATIC_CACHE_CONTROL));
        assert_eq!(CONTENT_ENCODING_HEADER, "Content-Encoding");
        assert_eq!(file.content_encoding, Some(GZIP_CONTENT_ENCODING));
    }

    #[test]
    fn non_directory_static_asset_sets_cache_control() {
        // Arrange
        let request = available_request("/assets/app.css");
        let catalog = catalog(&["/assets/app.css"]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::ServeStatic(file) = decision else {
            panic!("static asset should be served");
        };
        assert_eq!(file.path, "/assets/app.css");
        assert_eq!(file.cache_control, Some("max-age=2592000"));
    }

    #[test]
    fn missing_file_redirects_to_root_with_captive_portal_body() {
        // Arrange
        let request = available_request("/missing.js");
        let catalog = catalog(&["/index.html"]);

        // Act
        let decision = resolve_static_request(request, catalog);

        // Assert
        let StaticRouteDecision::RedirectToRoot(redirect) = decision else {
            panic!("missing file should redirect to root");
        };
        assert_eq!(redirect.status, 302);
        assert_eq!(redirect.location, "/");
        assert_eq!(redirect.body, STATIC_REDIRECT_BODY);
    }

    #[test]
    fn unsafe_paths_are_rejected_before_catalog_lookup() {
        // Arrange
        let unsafe_paths = [
            "/../secret",
            "/assets/../secret",
            "/assets\\secret",
            "/assets/\0secret",
            "//etc/passwd",
            "file:///etc/passwd",
        ];
        let catalog = catalog(&["/index.html", "/secret"]);

        for path in unsafe_paths {
            // Act
            let decision = resolve_static_request(available_request(path), catalog);

            // Assert
            let StaticRouteDecision::RejectPathTraversal(rejection) = decision else {
                panic!("{path} should be rejected before filesystem access");
            };
            assert_eq!(rejection.status, 400);
            assert_eq!(rejection.body, "Wrong API input");
        }
    }
}
