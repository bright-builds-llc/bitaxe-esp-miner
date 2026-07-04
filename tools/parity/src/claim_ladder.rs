#![allow(dead_code)]

const REQUIRED_TIER_IDS: &[&str] = &[
    "version_1_0_controlled_no_share",
    "version_1_1_prerequisite_readiness",
    "version_1_1_live_socket_runtime",
    "version_1_1_live_asic_share_outcome",
    "explicit_deferred_non_claim",
];
const REQUIRED_PHRASES: &[&str] = &["allowed claim", "blocked claim", "explicit non-claim"];
const CONTROLLED_NO_SHARE_SOAK: &str = "approved_controlled_no_share_soak";
const LIVE_SHARE_TERMS: &[&str] = &[
    "accepted share",
    "accepted shares",
    "rejected share",
    "rejected shares",
];

pub(crate) struct ClaimTier {
    pub(crate) id: &'static str,
    pub(crate) allowed_claim: &'static str,
    pub(crate) blocked_claims: &'static [&'static str],
    pub(crate) evidence_required: &'static str,
}

const CLAIM_LADDER_TIERS: &[ClaimTier] = &[
    ClaimTier {
        id: "version_1_0_controlled_no_share",
        allowed_claim: "Phase 21 controlled no-share closure",
        blocked_claims: &[
            "accepted shares",
            "rejected shares",
            "unbounded production mining",
            "full production mining",
        ],
        evidence_required: "approved_controlled_no_share_soak evidence",
    },
    ClaimTier {
        id: "version_1_1_prerequisite_readiness",
        allowed_claim: "Prerequisite readiness before BM1366 work dispatch",
        blocked_claims: &[
            "accepted shares",
            "rejected shares",
            "full active safety closure",
            "unbounded production mining",
        ],
        evidence_required: "fresh or explicitly bounded prerequisite observations",
    },
    ClaimTier {
        id: "version_1_1_live_socket_runtime",
        allowed_claim: "Live Stratum v1 socket/runtime evidence",
        blocked_claims: &[
            "live ASIC-derived share outcomes",
            "non-205 board support",
            "Stratum v2",
            "OTA/recovery trust",
        ],
        evidence_required: "redacted live socket/runtime artifacts",
    },
    ClaimTier {
        id: "version_1_1_live_asic_share_outcome",
        allowed_claim: "Accepted or rejected live ASIC-derived share outcome",
        blocked_claims: &[
            "non-205 board support",
            "Stratum v2",
            "runtime display/input parity",
            "BAP behavior",
        ],
        evidence_required: "parsed pool response to live ASIC-derived work",
    },
    ClaimTier {
        id: "explicit_deferred_non_claim",
        allowed_claim: "Deferred surface is named without promotion",
        blocked_claims: &[
            "full active safety closure",
            "OTA/recovery trust",
            "runtime display/input parity",
            "BAP behavior",
        ],
        evidence_required: "future exact-surface evidence and parity validation",
    },
];

pub(crate) fn claim_ladder_tiers() -> &'static [ClaimTier] {
    CLAIM_LADDER_TIERS
}

pub(crate) fn validate_claim_ladder_document(markdown: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lower_markdown = markdown.to_ascii_lowercase();

    for tier_id in REQUIRED_TIER_IDS {
        if !markdown.contains(tier_id) {
            errors.push(format!("claim ladder is missing required tier id {tier_id}"));
        }
    }

    for phrase in REQUIRED_PHRASES {
        if !lower_markdown.contains(phrase) {
            errors.push(format!("claim ladder is missing required phrase {phrase}"));
        }
    }

    validate_controlled_no_share_overclaims(markdown, &mut errors);

    errors
}

fn validate_controlled_no_share_overclaims(markdown: &str, errors: &mut Vec<String>) {
    for paragraph in markdown_paragraphs(markdown) {
        let normalized = paragraph.to_ascii_lowercase();
        if !normalized.contains(CONTROLLED_NO_SHARE_SOAK) {
            continue;
        }

        if LIVE_SHARE_TERMS
            .iter()
            .any(|live_share_term| normalized.contains(live_share_term))
        {
            errors.push(
                "controlled no-share overclaim: approved_controlled_no_share_soak cannot appear in the same paragraph as accepted/rejected share terms"
                    .to_owned(),
            );
        }
    }
}

fn markdown_paragraphs(markdown: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            push_paragraph(&mut paragraphs, &mut current);
            continue;
        }

        if starts_new_markdown_block(trimmed) {
            push_paragraph(&mut paragraphs, &mut current);
        }

        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(trimmed);
    }

    push_paragraph(&mut paragraphs, &mut current);
    paragraphs
}

fn starts_new_markdown_block(trimmed: &str) -> bool {
    trimmed.starts_with('#')
        || trimmed.starts_with("- ")
        || trimmed
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
            && trimmed.contains(". ")
}

fn push_paragraph(paragraphs: &mut Vec<String>, current: &mut String) {
    if current.trim().is_empty() {
        current.clear();
        return;
    }

    paragraphs.push(current.trim().to_owned());
    current.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAIM_LADDER_DOCUMENT: &str = include_str!(
        "../../../docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md"
    );

    #[test]
    fn claim_ladder_tiers_returns_required_tier_ids() {
        // Arrange
        let required_tiers = [
            "version_1_0_controlled_no_share",
            "version_1_1_prerequisite_readiness",
            "version_1_1_live_socket_runtime",
            "version_1_1_live_asic_share_outcome",
            "explicit_deferred_non_claim",
        ];

        // Act
        let tiers = claim_ladder_tiers();

        // Assert
        for required_tier in required_tiers {
            assert!(
                tiers.iter().any(|tier| tier.id == required_tier),
                "missing claim ladder tier {required_tier}"
            );
        }
    }

    #[test]
    fn phase22_claim_ladder_document_is_valid() {
        // Arrange
        let markdown = CLAIM_LADDER_DOCUMENT;

        // Act
        let errors = validate_claim_ladder_document(markdown);

        // Assert
        assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
    }

    #[test]
    fn controlled_no_share_soak_cannot_overclaim_live_share_outcomes() {
        // Arrange
        let markdown = r#"
## Allowed Claims

approved_controlled_no_share_soak proves accepted share and rejected shares.

## Blocked Claims

## Explicit Non-Claims

version_1_0_controlled_no_share
version_1_1_prerequisite_readiness
version_1_1_live_socket_runtime
version_1_1_live_asic_share_outcome
explicit_deferred_non_claim
"#;

        // Act
        let errors = validate_claim_ladder_document(markdown);

        // Assert
        assert!(
            errors
                .iter()
                .any(|error| error.contains("controlled no-share overclaim")),
            "expected controlled no-share overclaim error, got {errors:#?}"
        );
    }

    #[test]
    fn missing_explicit_deferred_non_claim_tier_is_rejected() {
        // Arrange
        let markdown = CLAIM_LADDER_DOCUMENT.replace("explicit_deferred_non_claim", "");

        // Act
        let errors = validate_claim_ladder_document(&markdown);

        // Assert
        assert!(
            errors
                .iter()
                .any(|error| error.contains("explicit_deferred_non_claim")),
            "expected missing tier id error, got {errors:#?}"
        );
    }
}
