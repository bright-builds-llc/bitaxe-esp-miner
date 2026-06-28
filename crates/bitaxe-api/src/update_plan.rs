//! Pure OTA and OTAWWW route decisions for firmware adapters.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/http_server.c`
//! - `.planning/phases/07-ota-filesystem-and-release-packaging/07-CONTEXT.md`

use crate::route_shell::{
    plan_http_access, unsupported_update_response, HttpAccessDecision, PublicHttpResponse,
    RouteAccessInput,
};

const TEXT_PLAIN: &str = "text/plain";

/// Phase 7 update route owned by the firmware adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRouteKind {
    /// Firmware app OTA route: `/api/system/OTA`.
    FirmwareOta,
    /// AxeOS static partition route: `/api/system/OTAWWW`.
    AxeOsStaticOtaWww,
}

/// Pure update request input before firmware upload effects run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateRequestInput {
    /// Route that received the upload request.
    pub route: UpdateRouteKind,
    /// Shared private-network/origin gate input.
    pub access: RouteAccessInput,
}

/// Public status labels used by the firmware update state surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatusLabel {
    /// Upload accepted and initialization is starting.
    Starting,
    /// Upload/write work is in progress.
    WorkingProgress,
    /// Firmware validation or activation failed.
    ValidationError,
    /// Firmware upload succeeded and reboot is scheduled.
    Rebooting,
}

impl UpdateStatusLabel {
    /// Returns the upstream-compatible status text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "Starting...",
            Self::WorkingProgress => "Working ({percent}%)",
            Self::ValidationError => "Validation Error",
            Self::Rebooting => "Rebooting...",
        }
    }
}

/// Firmware OTA accept plan for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirmwareOtaDecision {
    /// Expected upload filename.
    pub filename: &'static str,
    /// Initial public status label.
    pub start_status: UpdateStatusLabel,
    /// Progress status template.
    pub progress_template: &'static str,
    /// Public success response.
    pub success_response: PublicHttpResponse,
    /// Public status label for validation or activation errors.
    pub validation_error_status: UpdateStatusLabel,
    /// Public validation or activation error response.
    pub validation_error_response: PublicHttpResponse,
}

/// Explicit REL-03 gap for OTAWWW/static partition updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtaWwwGapDecision {
    /// Public fail-closed response.
    pub public_response: PublicHttpResponse,
    /// Gap owner.
    pub owner: &'static str,
    /// Release impact statement.
    pub release_impact: &'static str,
    /// Follow-up implementation path.
    pub follow_up: &'static str,
}

/// Pure update route decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRequestDecision {
    /// Request is rejected before upload effects run.
    Reject(PublicHttpResponse),
    /// Firmware OTA upload may proceed in the firmware adapter.
    AcceptFirmwareOta(FirmwareOtaDecision),
    /// OTAWWW remains fail-closed with an explicit REL-03 gap.
    OtaWwwGap(OtaWwwGapDecision),
}

/// Plans an update request before firmware upload effects run.
#[must_use]
pub fn plan_update_request(input: UpdateRequestInput) -> UpdateRequestDecision {
    let access_without_ap_bypass = RouteAccessInput {
        ap_mode_enabled: false,
        ..input.access
    };

    if let HttpAccessDecision::Deny(response) = plan_http_access(access_without_ap_bypass) {
        return UpdateRequestDecision::Reject(response);
    }

    if input.access.ap_mode_enabled {
        return UpdateRequestDecision::Reject(ap_mode_update_response());
    }

    match input.route {
        UpdateRouteKind::FirmwareOta => {
            UpdateRequestDecision::AcceptFirmwareOta(firmware_ota_decision())
        }
        UpdateRouteKind::AxeOsStaticOtaWww => UpdateRequestDecision::OtaWwwGap(otawww_gap()),
    }
}

const fn firmware_ota_decision() -> FirmwareOtaDecision {
    FirmwareOtaDecision {
        filename: "esp-miner.bin",
        start_status: UpdateStatusLabel::Starting,
        progress_template: UpdateStatusLabel::WorkingProgress.as_str(),
        success_response: PublicHttpResponse {
            status: 200,
            body: "Firmware update complete, rebooting now!",
            content_type: Some(TEXT_PLAIN),
        },
        validation_error_status: UpdateStatusLabel::ValidationError,
        validation_error_response: PublicHttpResponse {
            status: 500,
            body: "Validation / Activation Error",
            content_type: Some(TEXT_PLAIN),
        },
    }
}

const fn otawww_gap() -> OtaWwwGapDecision {
    OtaWwwGapDecision {
        public_response: unsupported_update_response(),
        owner: "phase-07-release",
        release_impact: "AxeOS static update unavailable until interruption evidence exists",
        follow_up: "implement whole-www partition update with interruption/recovery evidence",
    }
}

const fn ap_mode_update_response() -> PublicHttpResponse {
    PublicHttpResponse {
        status: 500,
        body: "Not allowed in AP mode",
        content_type: Some(TEXT_PLAIN),
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::route_shell::OriginGate;

    use super::{
        plan_update_request, UpdateRequestDecision, UpdateRequestInput, UpdateRouteKind,
        UpdateStatusLabel,
    };

    fn private_update_input(route: UpdateRouteKind) -> UpdateRequestInput {
        UpdateRequestInput {
            route,
            access: crate::RouteAccessInput {
                ap_mode_enabled: false,
                request_ip: Ipv4Addr::new(192, 168, 1, 25),
                origin: OriginGate::Parsed(Ipv4Addr::new(192, 168, 1, 2)),
            },
        }
    }

    fn ap_mode_update_input(route: UpdateRouteKind) -> UpdateRequestInput {
        UpdateRequestInput {
            route,
            access: crate::RouteAccessInput {
                ap_mode_enabled: true,
                request_ip: Ipv4Addr::new(192, 168, 4, 2),
                origin: OriginGate::Missing,
            },
        }
    }

    #[test]
    fn public_network_update_request_is_denied_before_route_work() {
        // Arrange
        let input = UpdateRequestInput {
            route: UpdateRouteKind::FirmwareOta,
            access: crate::RouteAccessInput {
                ap_mode_enabled: false,
                request_ip: Ipv4Addr::new(8, 8, 8, 8),
                origin: OriginGate::Parsed(Ipv4Addr::new(203, 0, 113, 10)),
            },
        };

        // Act
        let decision = plan_update_request(input);

        // Assert
        let UpdateRequestDecision::Reject(response) = decision else {
            panic!("public update request must be rejected");
        };
        assert_eq!(response.status, 401);
        assert_eq!(response.body, "Unauthorized");
    }

    #[test]
    fn ap_mode_firmware_ota_is_rejected_before_upload_work() {
        // Arrange
        let input = ap_mode_update_input(UpdateRouteKind::FirmwareOta);

        // Act
        let decision = plan_update_request(input);

        // Assert
        let UpdateRequestDecision::Reject(response) = decision else {
            panic!("AP-mode firmware OTA must be rejected");
        };
        assert_eq!(response.status, 500);
        assert_eq!(response.body, "Not allowed in AP mode");
    }

    #[test]
    fn ap_mode_otawww_is_rejected_before_gap_or_upload_work() {
        // Arrange
        let input = ap_mode_update_input(UpdateRouteKind::AxeOsStaticOtaWww);

        // Act
        let decision = plan_update_request(input);

        // Assert
        let UpdateRequestDecision::Reject(response) = decision else {
            panic!("AP-mode OTAWWW must be rejected");
        };
        assert_eq!(response.status, 500);
        assert_eq!(response.body, "Not allowed in AP mode");
    }

    #[test]
    fn firmware_ota_accepts_with_upstream_visible_status_and_response_copy() {
        // Arrange
        let input = private_update_input(UpdateRouteKind::FirmwareOta);

        // Act
        let decision = plan_update_request(input);

        // Assert
        let UpdateRequestDecision::AcceptFirmwareOta(plan) = decision else {
            panic!("private firmware OTA should be accepted");
        };
        assert_eq!(plan.filename, "esp-miner.bin");
        assert_eq!(plan.start_status, UpdateStatusLabel::Starting);
        assert_eq!(plan.start_status.as_str(), "Starting...");
        assert_eq!(plan.progress_template, "Working ({percent}%)");
        assert_eq!(plan.success_response.status, 200);
        assert_eq!(
            plan.success_response.body,
            "Firmware update complete, rebooting now!"
        );
        assert_eq!(
            plan.validation_error_status,
            UpdateStatusLabel::ValidationError
        );
        assert_eq!(plan.validation_error_status.as_str(), "Validation Error");
        assert_eq!(plan.validation_error_response.status, 500);
        assert_eq!(
            plan.validation_error_response.body,
            "Validation / Activation Error"
        );
    }

    #[test]
    fn otawww_defaults_to_explicit_rel03_gap() {
        // Arrange
        let input = private_update_input(UpdateRouteKind::AxeOsStaticOtaWww);

        // Act
        let decision = plan_update_request(input);

        // Assert
        let UpdateRequestDecision::OtaWwwGap(gap) = decision else {
            panic!("OTAWWW must remain a typed REL-03 gap");
        };
        assert_eq!(gap.public_response.status, 400);
        assert_eq!(gap.public_response.body, "Wrong API input");
        assert_eq!(gap.owner, "phase-07-release");
        assert_eq!(
            gap.release_impact,
            "AxeOS static update unavailable until interruption evidence exists"
        );
        assert_eq!(
            gap.follow_up,
            "implement whole-www partition update with interruption/recovery evidence"
        );
    }
}
