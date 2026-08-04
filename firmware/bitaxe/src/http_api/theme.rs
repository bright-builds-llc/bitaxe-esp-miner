use super::*;

const THEME_PERSISTENCE_ERROR_BODY: &str = "Failed to save theme";

pub(super) fn handle_theme_get<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let theme = theme_settings_from_snapshot(&settings_adapter::current_settings_snapshot());
        send_json(request, &theme)
    })
}

pub(super) fn handle_theme_post<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let mut request = request;
        let body_len = request_body_len(&mut request);
        if body_len > bitaxe_api::MAX_THEME_POST_BODY_BYTES {
            return send_text_error(request, 400, "Invalid JSON");
        }
        let body = match read_body_string(&mut request, body_len) {
            Ok(body) => body,
            Err(_) => return send_text_error(request, 400, "Invalid JSON"),
        };
        let plan = match plan_theme_post(&body) {
            Ok(plan) => plan,
            Err(error) => return send_text_error(request, error.status(), error.body()),
        };
        if plan.has_writes() && settings_adapter::persist_theme_update(&plan).is_err() {
            return send_text_error(request, 500, THEME_PERSISTENCE_ERROR_BODY);
        }
        send_json(request, &ThemePostResponse::ok())
    })
}
