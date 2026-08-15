use super::*;

pub(super) fn handle_system_info<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        publish_projected_system_info(crate::runtime_uptime::millis(), |system_info| {
            send_json(request, &system_info)
        })
        .map_err(|error| anyhow::anyhow!("operator snapshot publication failed: {error}"))
    })
}

pub(super) fn handle_command_status<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_json(
            request,
            &command_status_wire(crate::runtime_uptime::millis()),
        )
    })
}

pub(super) fn handle_wifi_scan<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let response = match wifi_adapter::scan_visible_networks() {
            Ok(response) => response,
            Err(failure) => {
                log::warn!("wifi_scan_status=failed category={}", failure.category());
                return send_public_response(
                    request,
                    PublicHttpResponse {
                        status: 500,
                        body: "WiFi scan failed",
                        content_type: Some("text/plain"),
                    },
                );
            }
        };

        send_json(request, &response)
    })
}

pub(super) fn handle_logs_download<'request, 'connection>(
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

pub(super) fn handle_asic_settings<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let snapshot = collect_api_snapshot();
        send_json(request, &asic_settings_from_snapshot(&snapshot))
    })
}

pub(super) fn handle_statistics<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let timestamp_ms = crate::runtime_uptime::millis();
        send_json(request, &projected_statistics(timestamp_ms))
    })
}

pub(super) fn handle_scoreboard<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_json(
            request,
            &projected_scoreboard(crate::runtime_uptime::millis()),
        )
    })
}

pub(super) fn handle_pause<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, pause_mining_plan())
}

pub(super) fn handle_resume<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, resume_mining_plan())
}

pub(super) fn handle_restart<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, restart_plan())
}

pub(super) fn handle_identify<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(request, identify_plan(identify_mode()))
}

pub(super) fn handle_block_found_dismiss<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_command(
        request,
        block_found_dismiss_plan(block_found_notification_state()),
    )
}

pub(super) fn handle_command<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
    plan: CommandPlan,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let effect = plan.effect;
        let maybe_deferred_effect = maybe_prepare_deferred_command_effect(&effect)?;
        send_json(request, &plan.response)?;
        apply_command_effect(effect, maybe_deferred_effect)?;
        Ok(())
    })
}

pub(super) fn handle_unknown_api_route<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        send_public_response(request, unknown_api_route_response())
    })
}
