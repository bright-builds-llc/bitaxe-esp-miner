use super::*;

pub(super) fn handle_settings_patch<'request, 'connection>(
    request: ApiRequest<'request, 'connection>,
) -> anyhow::Result<()> {
    handle_with_access_gate(request, |request| {
        let mut request = request;
        let body_len = request_body_len(&mut request);
        if let SettingsPatchBodyDecision::Reject(response) = plan_settings_patch_body_size(body_len)
        {
            settings_patch_warn_retained("axeos_settings_patch=rejected reason=body_too_large");
            return send_public_response(request, response);
        }

        let body = match read_body_string(&mut request, body_len) {
            Ok(body) => body,
            Err(public_error) => {
                settings_patch_warn_retained("axeos_settings_patch=rejected reason=body_read");
                return send_text_error(request, 400, public_error.body());
            }
        };
        let decision = match decide_v12_settings_body(&body) {
            Ok(decision) => decision,
            Err(error) => {
                settings_patch_warn_retained(&format!(
                    "axeos_settings_patch=rejected reason={}",
                    settings_patch_failure_label(error.reason())
                ));
                return send_text_error(request, 400, error.public_error().body());
            }
        };
        let hostname = match decision {
            V12SettingsDecision::Authorized(V12SettingsChange::Hostname(hostname)) => hostname,
            V12SettingsDecision::Authorized(V12SettingsChange::StartMiningOnBoot(preference)) => {
                settings_patch_retained(
                    "axeos_settings_patch=authorized category=start_mining_on_boot",
                );
                if settings_adapter::persist_start_mining_on_boot(preference).is_err() {
                    settings_patch_warn_retained(
                        "axeos_settings_patch=persistence_failed category=start_mining_on_boot",
                    );
                    return send_text_error(
                        request,
                        400,
                        SettingsPatchPublicError::WrongApiInput.body(),
                    );
                }
                let _ = crate::production_mining_session::notify(
                    bitaxe_stratum::v1::production_session::ProductionSessionWakeup::SettingsChanged,
                );
                send_settings_response(request, SettingsPublicResponse::EmptySuccess)?;
                settings_patch_retained(
                    "axeos_settings_patch=persistence_confirmed category=start_mining_on_boot",
                );
                return Ok(());
            }
            V12SettingsDecision::CompatibilityOnly {
                reason,
                field_count,
            } => {
                settings_patch_retained(&format!(
                    "axeos_settings_patch=compatibility_only reason={} fields={field_count}",
                    settings_exclusion_label(reason)
                ));
                send_settings_response(request, SettingsPublicResponse::EmptySuccess)?;
                settings_patch_retained(
                    "axeos_settings_patch=response_scheduled status=200 empty_body=true",
                );
                return Ok(());
            }
        };

        let mut adapter = match settings_adapter::FirmwareSettingsAdapter::open() {
            Ok(adapter) => adapter,
            Err(_) => {
                settings_patch_warn_retained(
                    "axeos_settings_patch=persistence_failed reason=adapter_open",
                );
                return send_text_error(
                    request,
                    400,
                    SettingsPatchPublicError::WrongApiInput.body(),
                );
            }
        };
        let plan = SettingsPersistencePlan::for_hostname(hostname);
        settings_patch_retained("axeos_settings_patch=authorized category=hostname");
        let success = match execute_settings_persistence_plan(&plan, &mut adapter) {
            Ok(success) => success,
            Err(error) => {
                settings_patch_warn_retained(&format!(
                    "axeos_settings_patch=persistence_failed reason={} disposition={:?}",
                    settings_persistence_failure_label(error.reason()),
                    error.disposition()
                ));
                return send_text_error(request, 400, error.public_error().body());
            }
        };
        settings_patch_retained("axeos_settings_patch=persistence_confirmed category=hostname");
        let _ = crate::production_mining_session::notify(
            bitaxe_stratum::v1::production_session::ProductionSessionWakeup::SettingsChanged,
        );
        let maybe_effect_lease =
            success.maybe_acquire_best_effort_effect_lease(prepare_settings_effects);
        if maybe_effect_lease.is_none() {
            settings_patch_warn_retained(
                "axeos_settings_patch=effects_degraded category=worker_unavailable",
            );
        }
        send_settings_response(request, success.public_response())?;
        settings_patch_retained(
            "axeos_settings_patch=response_scheduled status=200 empty_body=true",
        );
        if maybe_effect_lease
            .is_some_and(|effect_lease| effect_lease.release_after_response().is_err())
        {
            settings_patch_warn_retained(
                "axeos_settings_patch=effects_degraded category=worker_disconnected",
            );
        }
        Ok(())
    })
}
