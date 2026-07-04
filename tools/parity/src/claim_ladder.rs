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
