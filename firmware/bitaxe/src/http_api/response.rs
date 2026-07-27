use super::*;

pub(super) fn send_raw_public_response(
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

pub(super) fn set_raw_status(request: *mut sys::httpd_req_t, status: u16) -> sys::esp_err_t {
    let status_ptr = match status {
        400 => sys::HTTPD_400.as_ptr(),
        401 => HTTPD_401.as_ptr(),
        404 => sys::HTTPD_404.as_ptr(),
        500 => sys::HTTPD_500.as_ptr(),
        _ => sys::HTTPD_500.as_ptr(),
    };
    unsafe { sys::httpd_resp_set_status(request, status_ptr.cast()) }
}

pub(super) fn set_raw_content_type(
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

pub(super) fn send_json<'request, 'connection, T: Serialize>(
    request: ApiRequest<'request, 'connection>,
    value: &T,
) -> anyhow::Result<()> {
    let body = serde_json::to_vec(value)?;
    request
        .into_response(200, Some("OK"), &[("Content-Type", "application/json")])?
        .write_all(&body)?;
    Ok(())
}

pub(super) fn send_settings_response<'request, 'connection>(
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

pub(super) fn send_text_error<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
    status: u16,
    body: &'static str,
) -> anyhow::Result<()> {
    request
        .into_response(status, None, &[("Content-Type", "text/plain")])?
        .write_all(body.as_bytes())?;
    Ok(())
}

pub(super) fn send_public_response(
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

pub(super) fn request_body_len(request: &mut ApiRequest<'_, '_>) -> usize {
    let raw_request = (*request.connection()).handle();
    unsafe { (*raw_request).content_len }
}

pub(super) fn read_body_string(
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

pub(super) fn settings_patch_failure_label(reason: &SettingsPatchFailureReason) -> &'static str {
    match reason {
        SettingsPatchFailureReason::MalformedJson { .. } => "malformed_json",
        SettingsPatchFailureReason::NonObjectJson => "non_object_json",
        SettingsPatchFailureReason::InvalidKnownFields(_) => "invalid_known_fields",
    }
}

pub(super) fn settings_exclusion_label(reason: V12SettingsExclusionReason) -> &'static str {
    match reason {
        V12SettingsExclusionReason::EmptyPatch => "empty_patch",
        V12SettingsExclusionReason::UnknownField => "unknown_field",
        V12SettingsExclusionReason::BroaderKnownField => "broader_known_field",
        V12SettingsExclusionReason::CredentialField => "credential_field",
        V12SettingsExclusionReason::HardwareControlField => "hardware_control_field",
        V12SettingsExclusionReason::MiningOrSelfTestField => "mining_or_self_test_field",
        V12SettingsExclusionReason::MixedFieldSet => "mixed_field_set",
    }
}

pub(super) fn settings_persistence_failure_label(
    reason: &SettingsPersistenceFailure,
) -> &'static str {
    match reason {
        SettingsPersistenceFailure::Validation => "validation",
        SettingsPersistenceFailure::Transaction => "transaction",
        SettingsPersistenceFailure::Write { .. } => "write",
        SettingsPersistenceFailure::Commit => "commit",
        SettingsPersistenceFailure::Reload => "reload",
        SettingsPersistenceFailure::Reconcile => "reconcile",
        SettingsPersistenceFailure::Publication => "publication",
    }
}

pub(super) fn settings_patch_retained(line: &str) {
    log::info!("{line}");
    log_buffer::append_runtime_log_line(line);
}

pub(super) fn settings_patch_warn_retained(line: &str) {
    log::warn!("{line}");
    log_buffer::append_runtime_log_line(line);
}
