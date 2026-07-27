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

pub(super) fn handle_otawww_update_gap<'request, 'connection>(
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
