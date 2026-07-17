//! Exhaustive, narrow Phase 35 parity promotion.

mod checklist;
mod evaluator;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use checklist::ChecklistSnapshot;
pub(crate) use evaluator::evaluate_phase35_promotion;
pub(crate) use types::{Phase35EvidenceSource, Phase35LiveRechecks};

#[cfg(test)]
pub(crate) use types::{
    Phase35ClaimScope, Phase35NonPromotionReason, Phase35PromotionDecision, Phase35PromotionError,
    PHASE35_PROMOTABLE_ROWS,
};
