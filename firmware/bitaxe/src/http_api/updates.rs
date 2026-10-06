use super::*;

pub(super) fn handle_firmware_ota_update<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    let decision = plan_update_request(UpdateRequestInput {
        route: UpdateRouteKind::FirmwareOta,
        access: access_input(&mut request),
    });

    let plan = match decision {
        UpdateRequestDecision::AcceptFirmwareOta(plan) => plan,
        UpdateRequestDecision::Reject(response) => return send_public_response(request, response),
        UpdateRequestDecision::AcceptOtaWww(_) => {
            log::warn!("firmware_ota_update=rejected reason=unexpected_otawww_decision");
            return send_public_response(request, unsupported_update_response());
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

    let Some(mutation_guard) = crate::noise_serial_runtime::MutationGuard::acquire() else {
        return send_text_error(request, 409, "Diagnostic owns configuration");
    };
    let raw_request = (*request.connection()).handle();
    let result = crate::ota_update::stream_firmware_ota(raw_request, record_firmware_ota_status);
    match result {
        FirmwareOtaApplyResult::Complete { bytes_written } => {
            log::info!("firmware_ota_update=complete bytes_written={bytes_written}");
            send_public_response(request, plan.success_response)?;
            schedule_firmware_ota_restart(mutation_guard);
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

pub(super) fn handle_otawww_update<'request, 'connection>(
    mut request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    let plan = match plan_update_request(UpdateRequestInput {
        route: UpdateRouteKind::AxeOsStaticOtaWww,
        access: access_input(&mut request),
    }) {
        UpdateRequestDecision::AcceptOtaWww(plan) => plan,
        UpdateRequestDecision::Reject(response) => {
            if response.body == UPDATE_AP_MODE_REJECTION_BODY {
                log::warn!("otawww_update=rejected reason=ap_mode");
            }
            return send_public_response(request, response);
        }
        UpdateRequestDecision::AcceptFirmwareOta(_) => {
            log::warn!("otawww_update=rejected reason=unexpected_firmware_ota_decision");
            return send_public_response(request, unsupported_update_response());
        }
    };

    let Some(_mutation_guard) = crate::noise_serial_runtime::MutationGuard::acquire() else {
        return send_text_error(request, 409, "Diagnostic owns configuration");
    };
    log::info!("otawww_update=start filename={}", plan.filename);
    let raw_request = (*request.connection()).handle();
    match crate::www_update::stream_www_update(raw_request, record_www_update_status) {
        WwwUpdateResult::Rejected(response) => {
            log::warn!("otawww_update=rejected status={}", response.status);
            send_public_response(request, response)
        }
        WwwUpdateResult::Complete { bytes_written } => {
            log::info!("otawww_update=complete bytes_written={bytes_written}");
            send_public_response(request, www_success_response())?;
            record_www_update_status(WwwUpdateStatus::Finished);
            Ok(())
        }
        WwwUpdateResult::ProtocolError { code } => {
            log::warn!("otawww_update=protocol_error code={code}");
            send_public_response(request, www_protocol_error_response())
        }
        WwwUpdateResult::WriteError { esp_err } => {
            log::warn!("otawww_update=write_error esp_err={esp_err}");
            send_public_response(request, www_write_error_response())
        }
    }
}
