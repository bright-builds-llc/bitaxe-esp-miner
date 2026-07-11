//! Pure accepted-state classification for BM1366 diagnostic snapshots.
//!
//! The types in this module deliberately retain only counts, booleans, and
//! closed-set categories. Raw register values and wire data stay at the I/O
//! boundary.

/// Ordered observation boundaries in the BM1366 initialization and work path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AcceptedStateStage {
    PostEnumerate,
    PostMiningReady,
    PostMaxBaud,
    PostMaskReload,
    PostFirstWork,
}

impl AcceptedStateStage {
    const ALL: [Self; 5] = [
        Self::PostEnumerate,
        Self::PostMiningReady,
        Self::PostMaxBaud,
        Self::PostMaskReload,
        Self::PostFirstWork,
    ];

    /// Returns the stable report vocabulary for this stage.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PostEnumerate => "post_enumerate",
            Self::PostMiningReady => "post_mining_ready",
            Self::PostMaxBaud => "post_max_baud",
            Self::PostMaskReload => "post_mask_reload",
            Self::PostFirstWork => "post_first_work",
        }
    }
}

/// Closed comparison state used for chip count and overall classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptedStateStatus {
    Match,
    Mismatch,
    Unavailable,
}

impl AcceptedStateStatus {
    /// Returns the stable report vocabulary for this comparison state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Redaction-safe electrical activity category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerDeltaClass {
    Falling,
    Flat,
    RisingHashing,
    Unavailable,
}

impl PowerDeltaClass {
    /// Returns the stable report vocabulary for this power category.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Falling => "falling",
            Self::Flat => "flat",
            Self::RisingHashing => "rising_hashing",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Closed next-investigation recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptedStateRecommendation {
    AcceptedStateTransitionDivergence,
    ColdBootRecoveryLifecycleParity,
    UpstreamInitTranscriptPrefixBisection,
    None,
}

impl AcceptedStateRecommendation {
    /// Returns the stable report vocabulary for this recommendation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AcceptedStateTransitionDivergence => "accepted_state_transition_divergence",
            Self::ColdBootRecoveryLifecycleParity => "cold_boot_recovery_lifecycle_parity",
            Self::UpstreamInitTranscriptPrefixBisection => {
                "upstream_init_transcript_prefix_bisection"
            }
            Self::None => "none",
        }
    }
}

/// Category-only snapshot captured at one accepted-state boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedStateSnapshot {
    pub stage: AcceptedStateStage,
    pub chip_count_class: AcceptedStateStatus,
    pub readable_response_count: u32,
    pub error_counter_active: bool,
    pub domain_counter_active: bool,
    pub total_counter_active: bool,
    pub power_delta_class: PowerDeltaClass,
    pub result_correlated: bool,
    pub submit_observed: bool,
}

impl AcceptedStateSnapshot {
    fn has_safe_readable_observation(self) -> bool {
        self.readable_response_count > 0
            && self.chip_count_class != AcceptedStateStatus::Unavailable
    }

    fn has_result_progress(self) -> bool {
        self.result_correlated || self.submit_observed
    }

    /// Renders the category-only firmware/comparator interchange marker.
    pub fn marker(self) -> String {
        let observation = if self.has_safe_readable_observation() {
            "available"
        } else {
            "unavailable"
        };
        format!(
            "accepted_state_snapshot stage={} observation={observation} chip_count_class={} readable_responses={} error_counter_active={} domain_counter_active={} total_counter_active={} power_delta_class={} result_correlated={} submit_observed={} redacted=true",
            self.stage.as_str(),
            self.chip_count_class.as_str(),
            self.readable_response_count,
            self.error_counter_active,
            self.domain_counter_active,
            self.total_counter_active,
            self.power_delta_class.as_str(),
            self.result_correlated,
            self.submit_observed,
        )
    }
}

/// Deterministic comparison of upstream and Rust accepted-state snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedStateClassification {
    pub accepted_state_status: AcceptedStateStatus,
    pub first_divergent_stage: Option<AcceptedStateStage>,
    pub recommended_investigation: AcceptedStateRecommendation,
}

/// Classifies ordered accepted-state observations without performing I/O.
pub fn classify_accepted_state(
    upstream: &[AcceptedStateSnapshot],
    rust: &[AcceptedStateSnapshot],
) -> AcceptedStateClassification {
    if let Some(first_missing_stage) = AcceptedStateStage::ALL.into_iter().find(|stage| {
        snapshot_for_stage(upstream, *stage).is_none() || snapshot_for_stage(rust, *stage).is_none()
    }) {
        return AcceptedStateClassification {
            accepted_state_status: AcceptedStateStatus::Unavailable,
            first_divergent_stage: Some(first_missing_stage),
            recommended_investigation: AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity,
        };
    }

    let result_progress = rust
        .iter()
        .copied()
        .any(AcceptedStateSnapshot::has_result_progress);
    let mut maybe_first_divergent_stage = None;
    let mut saw_missing_observation = false;
    let mut saw_counter_divergence = false;
    let mut saw_other_mismatch = false;

    for stage in AcceptedStateStage::ALL {
        let maybe_upstream = snapshot_for_stage(upstream, stage);
        let maybe_rust = snapshot_for_stage(rust, stage);

        let (Some(upstream_snapshot), Some(rust_snapshot)) = (maybe_upstream, maybe_rust) else {
            saw_missing_observation = true;
            maybe_first_divergent_stage.get_or_insert(stage);
            continue;
        };

        if !upstream_snapshot.has_safe_readable_observation()
            || !rust_snapshot.has_safe_readable_observation()
        {
            saw_missing_observation = true;
            maybe_first_divergent_stage.get_or_insert(stage);
            continue;
        }

        if upstream_counter_active_rust_inactive(upstream_snapshot, rust_snapshot) {
            saw_counter_divergence = true;
            maybe_first_divergent_stage.get_or_insert(stage);
        } else if state_values_differ(upstream_snapshot, rust_snapshot) {
            saw_other_mismatch = true;
            maybe_first_divergent_stage.get_or_insert(stage);
        }
    }

    let accepted_state_status = if saw_missing_observation {
        AcceptedStateStatus::Unavailable
    } else if saw_counter_divergence || saw_other_mismatch {
        AcceptedStateStatus::Mismatch
    } else {
        AcceptedStateStatus::Match
    };

    let recommended_investigation = if saw_missing_observation {
        AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
    } else if result_progress {
        AcceptedStateRecommendation::None
    } else if saw_counter_divergence {
        AcceptedStateRecommendation::AcceptedStateTransitionDivergence
    } else if saw_other_mismatch {
        AcceptedStateRecommendation::UpstreamInitTranscriptPrefixBisection
    } else {
        AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
    };

    AcceptedStateClassification {
        accepted_state_status,
        first_divergent_stage: maybe_first_divergent_stage,
        recommended_investigation,
    }
}

fn snapshot_for_stage(
    snapshots: &[AcceptedStateSnapshot],
    stage: AcceptedStateStage,
) -> Option<AcceptedStateSnapshot> {
    snapshots
        .iter()
        .copied()
        .find(|snapshot| snapshot.stage == stage)
}

fn upstream_counter_active_rust_inactive(
    upstream: AcceptedStateSnapshot,
    rust: AcceptedStateSnapshot,
) -> bool {
    (upstream.error_counter_active && !rust.error_counter_active)
        || (upstream.domain_counter_active && !rust.domain_counter_active)
        || (upstream.total_counter_active && !rust.total_counter_active)
}

fn state_values_differ(upstream: AcceptedStateSnapshot, rust: AcceptedStateSnapshot) -> bool {
    upstream.chip_count_class != rust.chip_count_class
        || upstream.readable_response_count != rust.readable_response_count
        || upstream.error_counter_active != rust.error_counter_active
        || upstream.domain_counter_active != rust.domain_counter_active
        || upstream.total_counter_active != rust.total_counter_active
        || upstream.power_delta_class != rust.power_delta_class
}

#[cfg(test)]
mod tests {
    use super::{
        classify_accepted_state, AcceptedStateRecommendation, AcceptedStateSnapshot,
        AcceptedStateStage, AcceptedStateStatus, PowerDeltaClass,
    };

    fn snapshot(stage: AcceptedStateStage) -> AcceptedStateSnapshot {
        AcceptedStateSnapshot {
            stage,
            chip_count_class: AcceptedStateStatus::Match,
            readable_response_count: 1,
            error_counter_active: false,
            domain_counter_active: false,
            total_counter_active: false,
            power_delta_class: PowerDeltaClass::Flat,
            result_correlated: false,
            submit_observed: false,
        }
    }

    fn complete_snapshots() -> Vec<AcceptedStateSnapshot> {
        AcceptedStateStage::ALL.into_iter().map(snapshot).collect()
    }

    #[test]
    fn accepted_state_marker_is_category_only() {
        // Arrange
        let snapshot = snapshot(AcceptedStateStage::PostFirstWork);

        // Act
        let marker = snapshot.marker();

        // Assert
        assert_eq!(marker, "accepted_state_snapshot stage=post_first_work observation=available chip_count_class=match readable_responses=1 error_counter_active=false domain_counter_active=false total_counter_active=false power_delta_class=flat result_correlated=false submit_observed=false redacted=true");
        assert!(!marker.contains("asic_address"));
        assert!(!marker.contains("register_value"));
    }

    #[test]
    fn accepted_state_routes_upstream_active_rust_inactive_counter() {
        // Arrange
        let mut upstream = complete_snapshots();
        let upstream_first_work = upstream
            .iter_mut()
            .find(|snapshot| snapshot.stage == AcceptedStateStage::PostFirstWork)
            .expect("complete fixture contains post-first-work stage");
        upstream_first_work.total_counter_active = true;
        let rust = complete_snapshots();

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::AcceptedStateTransitionDivergence
        );
        assert_eq!(
            classification.first_divergent_stage,
            Some(AcceptedStateStage::PostFirstWork)
        );
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Mismatch
        );
    }

    #[test]
    fn accepted_state_routes_missing_observation_to_lifecycle_parity() {
        // Arrange
        let upstream = complete_snapshots();
        let mut rust = complete_snapshots();
        let rust_enumerate = rust
            .iter_mut()
            .find(|snapshot| snapshot.stage == AcceptedStateStage::PostEnumerate)
            .expect("complete fixture contains post-enumerate stage");
        rust_enumerate.readable_response_count = 0;
        rust_enumerate.chip_count_class = AcceptedStateStatus::Unavailable;

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
        );
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
    }

    #[test]
    fn accepted_state_routes_matching_idle_state_to_lifecycle_parity() {
        // Arrange
        let upstream = complete_snapshots();
        let rust = complete_snapshots();

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
        );
        assert_eq!(classification.first_divergent_stage, None);
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Match
        );
    }

    #[test]
    fn accepted_state_routes_other_mismatch_to_transcript_bisection() {
        // Arrange
        let upstream = complete_snapshots();
        let mut rust = complete_snapshots();
        let rust_max_baud = rust
            .iter_mut()
            .find(|snapshot| snapshot.stage == AcceptedStateStage::PostMaxBaud)
            .expect("complete fixture contains post-max-baud stage");
        rust_max_baud.chip_count_class = AcceptedStateStatus::Mismatch;

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::UpstreamInitTranscriptPrefixBisection
        );
        assert_eq!(
            classification.first_divergent_stage,
            Some(AcceptedStateStage::PostMaxBaud)
        );
    }

    #[test]
    fn accepted_state_routes_correlate_and_submit_to_none() {
        // Arrange
        let upstream = complete_snapshots();
        let mut rust = complete_snapshots();
        let rust_first_work = rust
            .iter_mut()
            .find(|snapshot| snapshot.stage == AcceptedStateStage::PostFirstWork)
            .expect("complete fixture contains post-first-work stage");
        rust_first_work.result_correlated = true;
        rust_first_work.submit_observed = true;

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::None
        );
    }

    #[test]
    fn accepted_state_unavailable_observation_precedes_mismatch() {
        // Arrange
        let upstream = complete_snapshots();
        let mut rust = complete_snapshots();
        rust[2].chip_count_class = AcceptedStateStatus::Mismatch;
        rust[3].readable_response_count = 0;
        rust[3].chip_count_class = AcceptedStateStatus::Unavailable;

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
        );
    }

    #[test]
    fn accepted_state_unavailable_observation_precedes_result_progress() {
        // Arrange
        let upstream = complete_snapshots();
        let mut rust = complete_snapshots();
        rust[3].readable_response_count = 0;
        rust[3].chip_count_class = AcceptedStateStatus::Unavailable;
        rust[4].result_correlated = true;
        rust[4].submit_observed = true;

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
        );
    }

    #[test]
    fn accepted_state_empty_inputs_are_unavailable() {
        // Arrange
        let upstream = Vec::new();
        let rust = Vec::new();

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
        assert_eq!(
            classification.first_divergent_stage,
            Some(AcceptedStateStage::PostEnumerate)
        );
        assert_eq!(
            classification.recommended_investigation,
            AcceptedStateRecommendation::ColdBootRecoveryLifecycleParity
        );
    }

    #[test]
    fn accepted_state_matching_three_of_five_inputs_are_unavailable() {
        // Arrange
        let upstream = complete_snapshots().into_iter().take(3).collect::<Vec<_>>();
        let rust = upstream.clone();

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
        assert_eq!(
            classification.first_divergent_stage,
            Some(AcceptedStateStage::PostMaskReload)
        );
    }

    #[test]
    fn accepted_state_one_sided_missing_input_is_unavailable() {
        // Arrange
        let upstream = complete_snapshots();
        let rust = upstream.iter().copied().take(4).collect::<Vec<_>>();

        // Act
        let classification = classify_accepted_state(&upstream, &rust);

        // Assert
        assert_eq!(
            classification.accepted_state_status,
            AcceptedStateStatus::Unavailable
        );
        assert_eq!(
            classification.first_divergent_stage,
            Some(AcceptedStateStage::PostFirstWork)
        );
    }
}
