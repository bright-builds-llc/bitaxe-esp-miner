//! Claim-specific Phase 36 correction and checklist projection.

mod checklist;
mod evaluator;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use tests::prerequisites as synthetic_phase36_prerequisites;

pub(crate) use checklist::Phase36ChecklistSnapshot;
pub(crate) use evaluator::evaluate_phase36_promotion;
#[cfg(test)]
pub(crate) use types::{
    Phase36ClaimDecision, Phase36ClaimScope, Phase36DecisionReason, Phase36PromotionError,
    PHASE36_AFFECTED_ROWS,
};
pub(crate) use types::{
    Phase36ClaimPrerequisites, Phase36PromotionMatrix, ValidatedHostnameDurabilityFacts,
};

pub(crate) fn current_phase36_evaluator_digest() -> String {
    crate::phase35_evidence::sha256_hex(
        concat!(
            include_str!("phase36_promotion/checklist.rs"),
            include_str!("phase36_promotion/evaluator.rs"),
            include_str!("phase36_promotion/types.rs"),
        )
        .as_bytes(),
    )
}
