use bitaxe_stratum::v1::production_session::ProductionSessionEvent;
use bitaxe_stratum::v1::production_work::ScoreboardCandidate;

pub(super) fn record(candidate: ScoreboardCandidate) -> Option<ProductionSessionEvent> {
    if let Err(error) = crate::scoreboard_adapter::record_candidate(candidate) {
        log::warn!("scoreboard=record_failed category={}", error.category());
    }
    None
}
