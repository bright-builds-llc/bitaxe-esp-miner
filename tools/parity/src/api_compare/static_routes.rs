use super::*;

pub(super) fn validate_static_route_evidence(
    static_usage: &StaticRouteUsageFixture,
    validation_errors: &mut Vec<String>,
) -> usize {
    let mut checked = 0;
    let static_routes = static_route_set(&static_usage.service_calls);

    for (method, path) in REQUIRED_STATIC_USAGE_ROUTES {
        checked += 1;
        if !static_routes.contains(&route_key(method, path)) {
            validation_errors.push(format!(
                "AxeOS static route usage fixture missing {method} {path}"
            ));
        }
    }

    for call in &static_usage.service_calls {
        checked += 1;
        if call.evidence_type != "static-route" {
            validation_errors.push(format!(
                "AxeOS route usage {} ({} {}) has wrong evidence label {}",
                call.surface, call.method, call.path, call.evidence_type
            ));
        }

        if matches!(call.path.as_str(), "/api/system/OTA" | "/api/system/OTAWWW") {
            validate_phase7_update_route(call, validation_errors);
        }

        if call.path == "/recovery" {
            validate_phase7_static_boundary(call, "recovery-route", validation_errors);
        }

        if call.path == "/*" {
            validate_phase7_static_boundary(call, "static-fallback", validation_errors);
        }

        checked += validate_verified_claim_policy(call, validation_errors);
    }

    checked += 1;
    if static_usage.static_packaging.phase05_success_claim {
        validation_errors
            .push("static/recovery packaging fixture must not claim Phase 05 success".to_owned());
    }

    checked += 1;
    if !static_usage.static_packaging.phase07_owner {
        validation_errors
            .push("static/recovery packaging fixture must mark Phase 7 ownership".to_owned());
    }

    checked
}

fn validate_phase7_update_route(call: &StaticRouteUsage, validation_errors: &mut Vec<String>) {
    if call.phase_owner != "phase07" {
        validation_errors.push(format!(
            "{} ({}) must remain Phase 7-owned, got {}",
            call.surface, call.path, call.phase_owner
        ));
    }

    if call.phase05_behavior != "unsafe-success-blocked" {
        validation_errors.push(format!(
            "{} ({}) must be unsafe-success-blocked in Phase 05, got {}",
            call.surface, call.path, call.phase05_behavior
        ));
    }

    if call.counts_as_phase05_success {
        validation_errors.push(format!(
            "{} ({}) must not count as Phase 05 update success",
            call.surface, call.path
        ));
    }
}

fn validate_phase7_static_boundary(
    call: &StaticRouteUsage,
    expected_category: &str,
    validation_errors: &mut Vec<String>,
) {
    if call.category != expected_category {
        validation_errors.push(format!(
            "{} ({}) must be category {expected_category}, got {}",
            call.surface, call.path, call.category
        ));
    }

    if call.phase_owner != "phase07" {
        validation_errors.push(format!(
            "{} ({}) must remain Phase 7-owned, got {}",
            call.surface, call.path, call.phase_owner
        ));
    }

    if call.counts_as_phase05_success {
        validation_errors.push(format!(
            "{} ({}) must not count as Phase 05 static/recovery packaging success",
            call.surface, call.path
        ));
    }
}

fn validate_verified_claim_policy(
    call: &StaticRouteUsage,
    validation_errors: &mut Vec<String>,
) -> usize {
    let Some(claim) = &call.verified_claim else {
        return 0;
    };

    if claim.status != "verified" || !is_release_sensitive_route(&call.method, &call.path) {
        return 1;
    }

    let has_unknown_evidence = claim
        .evidence
        .iter()
        .any(|evidence| !is_known_verified_evidence_label(evidence.as_str()));
    let has_strong_evidence = claim
        .evidence
        .iter()
        .any(|evidence| STRONG_VERIFIED_EVIDENCE_LABELS.contains(&evidence.as_str()));

    if has_unknown_evidence || !has_strong_evidence {
        validation_errors.push(format!(
            "release-sensitive route {} has insufficient verified evidence: evidence={}",
            route_key(&call.method, &call.path),
            claim.evidence.join(", ")
        ));
    }

    1
}

fn is_known_verified_evidence_label(evidence: &str) -> bool {
    WEAK_VERIFIED_EVIDENCE_LABELS.contains(&evidence)
        || STRONG_VERIFIED_EVIDENCE_LABELS.contains(&evidence)
}

fn is_release_sensitive_route(method: &str, path: &str) -> bool {
    REQUIRED_PHASE07_ROUTE_POLICY
        .iter()
        .any(|policy| route_method_label(policy.method) == method && policy.path == path)
}
