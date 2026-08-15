use std::time::Instant;

use bitaxe_api::{CommandStatusWire, SystemInfoWire};
use bitaxe_http_transport::StrictHttpClient;

use super::{CampaignTerminalCategory, HTTP_DEADLINE};

pub(super) fn fetch_system_info(
    http: &StrictHttpClient,
) -> Result<Option<SystemInfoWire>, CampaignTerminalCategory> {
    // A lost read cannot prove or disprove a command effect. Keep waiting
    // within the phase deadline; a successful malformed response remains an
    // explicit correlation failure below.
    let Ok(observation) = http.get_system_info(Instant::now() + HTTP_DEADLINE) else {
        return Ok(None);
    };
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| response.status() == 200)
    else {
        return Ok(None);
    };
    serde_json::from_slice(response.body())
        .map(Some)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)
}

pub(super) fn fetch_command_status(
    http: &StrictHttpClient,
) -> Result<Option<CommandStatusWire>, CampaignTerminalCategory> {
    let Ok(observation) = http.get_command_status(Instant::now() + HTTP_DEADLINE) else {
        return Ok(None);
    };
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| response.status() == 200)
    else {
        return Ok(None);
    };
    serde_json::from_slice(response.body())
        .map(Some)
        .map_err(|_| CampaignTerminalCategory::NetworkCorrelationFailed)
}
