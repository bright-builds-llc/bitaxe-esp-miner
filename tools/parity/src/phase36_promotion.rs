//! Claim-specific Phase 36 correction and checklist projection.

mod checklist;
mod evaluator;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use checklist::Phase36ChecklistSnapshot;
pub(crate) use evaluator::evaluate_phase36_promotion;
pub(crate) use types::{
    Phase36ClaimPrerequisites, Phase36PromotionError, Phase36PromotionMatrix,
    ValidatedHostnameDurabilityFacts,
};

#[cfg(test)]
pub(crate) use types::{
    Phase36ClaimDecision, Phase36ClaimScope, Phase36DecisionReason, PHASE36_AFFECTED_ROWS,
};
