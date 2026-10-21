use super::*;

const BEFORE: &str = "Claim. Non-claims: profiles.";
const AFTER: &str = "Claim. Non-claims: profiles. Later soak evidence.";

fn revision() -> Revision<'static> {
    Revision {
        row_id: "PWR-002",
        before_status: "verified",
        after_status: "verified",
        before_target: "crate/src/lib.rs",
        after_target: "crate/src/lib.rs",
        before_evidence: "unit,hardware-smoke",
        after_evidence: "unit,hardware-smoke,soak",
        maybe_before_notes: Some(BEFORE),
        maybe_after_notes: Some(AFTER),
    }
}

fn refusal(revision: &Revision) -> String {
    require_authorized(&authority_for("PWR-002", "soak", AFTER), revision)
        .expect_err("revision must be refused")
}

fn passage_only_authority() -> Authority {
    let mut authority = authority_for("PWR-002", "soak", AFTER);
    authority.rows[0].added_evidence.clear();
    authority
}

fn passage_only_revision() -> Revision<'static> {
    Revision {
        after_evidence: "unit,hardware-smoke",
        ..revision()
    }
}

#[test]
fn the_granted_revision_is_accepted() {
    // Arrange
    let authority = authority_for("PWR-002", "soak", AFTER);

    // Act
    let result = require_authorized(&authority, &revision());

    // Assert
    assert!(result.is_ok());
}

#[test]
fn a_passage_only_revision_is_accepted() {
    // Arrange
    let authority = passage_only_authority();

    // Act
    let result = require_authorized(&authority, &passage_only_revision());

    // Assert
    assert!(result.is_ok());
}

#[test]
fn a_passage_only_revision_that_changes_the_evidence_is_refused() {
    // Arrange
    let revision = Revision {
        after_evidence: "unit,hardware-smoke,soak",
        ..passage_only_revision()
    };

    // Act
    let error =
        require_authorized(&passage_only_authority(), &revision).expect_err("changed evidence");

    // Assert
    assert!(error.contains("exactly the granted evidence"));
}

#[test]
fn a_passage_only_revision_with_identical_notes_is_refused() {
    // Arrange: pinning the unchanged notes must not authorize a no-op revision.
    let mut authority = passage_only_authority();
    authority.rows[0].after_notes_sha256 = sha256_hex(BEFORE.as_bytes());
    let revision = Revision {
        maybe_after_notes: Some(BEFORE),
        ..passage_only_revision()
    };

    // Act
    let error = require_authorized(&authority, &revision).expect_err("identical notes");

    // Assert
    assert!(error.contains("separated, printable"));
}

#[test]
fn a_passage_only_grant_is_well_formed() {
    // Arrange
    let grant = &passage_only_authority().rows[0];

    // Act
    let valid = is_valid_grant(grant);

    // Assert
    assert!(valid);
}

#[test]
fn a_grant_with_a_malformed_token_is_refused() {
    // Arrange
    let mut authority = authority_for("PWR-002", "soak", AFTER);
    authority.rows[0]
        .added_evidence
        .push("Hardware Smoke".to_owned());

    // Act
    let valid = is_valid_grant(&authority.rows[0]);

    // Assert
    assert!(!valid);
}

#[test]
fn a_row_the_authority_does_not_name_is_refused() {
    // Arrange
    let revision = Revision {
        row_id: "PWR-003",
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("does not name PWR-003"));
}

#[test]
fn a_status_change_is_refused() {
    // Arrange
    let revision = Revision {
        after_status: "implemented",
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("keeps a verified row verified"));
}

#[test]
fn a_target_change_is_refused() {
    // Arrange
    let revision = Revision {
        after_target: "crate/src/other.rs",
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("owned target"));
}

#[test]
fn evidence_other_than_the_granted_tokens_is_refused() {
    // Arrange
    let revision = Revision {
        after_evidence: "unit,hardware-smoke,release-gate",
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("exactly the granted evidence"));
}

#[test]
fn notes_that_drop_part_of_the_old_notes_are_refused() {
    // Arrange: dropping the non-claims would widen the claim.
    let revision = Revision {
        maybe_after_notes: Some("Claim. Later soak evidence."),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("only append"));
}

#[test]
fn a_suffix_glued_to_the_old_notes_is_refused() {
    // Arrange
    let revision = Revision {
        maybe_after_notes: Some("Claim. Non-claims: profiles.X"),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("separated, printable"));
}

#[test]
fn a_blank_suffix_is_refused() {
    // Arrange
    let revision = Revision {
        maybe_after_notes: Some("Claim. Non-claims: profiles.   "),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("separated, printable"));
}

#[test]
fn a_suffix_with_invisible_formatting_is_refused() {
    // Arrange
    let revision = Revision {
        maybe_after_notes: Some("Claim. Non-claims: profiles. Later\u{202e} soak."),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("separated, printable"));
}

#[test]
fn notes_other_than_the_granted_text_are_refused() {
    // Arrange: a valid append, but not the one the owner pinned.
    let revision = Revision {
        maybe_after_notes: Some("Claim. Non-claims: profiles. Other text."),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("granted digest"));
}

#[test]
fn a_second_revision_of_an_already_revised_row_is_refused() {
    // Arrange: the row already carries the granted notes.
    let revision = Revision {
        before_evidence: "unit,hardware-smoke,soak",
        after_evidence: "unit,hardware-smoke,soak,soak",
        maybe_before_notes: Some(AFTER),
        maybe_after_notes: Some("Claim. Non-claims: profiles. Later soak evidence. More."),
        ..revision()
    };

    // Act
    let error = refusal(&revision);

    // Assert
    assert!(error.contains("granted digest"));
}

#[test]
fn an_authority_outside_its_directory_is_refused() {
    // Arrange
    let workspace = Utf8PathBuf::from("/nonexistent");

    // Act
    let error = read_binding(
        &workspace,
        Some(Utf8Path::new("docs/parity/authority.json")),
    )
    .err()
    .expect("outside the authority directory");

    // Assert
    assert!(error.contains(AUTHORITY_ROOT));
}

#[test]
fn evidence_outside_the_evidence_tree_is_refused() {
    // Arrange
    let escaping = EvidenceFile {
        path: "docs/parity/evidence/../checklist.md".to_owned(),
        sha256: "0".repeat(64),
    };

    // Act
    let error =
        require_evidence_file(Utf8Path::new("/nonexistent"), &escaping).expect_err("escaping path");

    // Assert
    assert!(error.contains(EVIDENCE_ROOT));
}

#[test]
fn evidence_that_changed_after_the_decision_is_refused() {
    // Arrange
    let workspace = Utf8PathBuf::from_path_buf(
        std::env::temp_dir().join(format!("bitaxe-verified-revision-{}", std::process::id())),
    )
    .expect("temporary path must be UTF-8");
    std::fs::create_dir_all(workspace.join(EVIDENCE_ROOT)).expect("evidence root");
    std::fs::write(workspace.join("docs/parity/evidence/soak.md"), "edited").expect("evidence");
    let evidence = EvidenceFile {
        path: "docs/parity/evidence/soak.md".to_owned(),
        sha256: sha256_hex(b"original"),
    };

    // Act
    let result = require_evidence_file(&workspace, &evidence);

    // Assert
    std::fs::remove_dir_all(&workspace).expect("cleanup");
    assert!(result.expect_err("changed evidence").contains("changed"));
}

#[test]
fn an_authority_edited_after_its_receipt_fails_on_replay() {
    // Arrange
    let workspace = Utf8PathBuf::from_path_buf(
        std::env::temp_dir().join(format!("bitaxe-verified-authority-{}", std::process::id())),
    )
    .expect("temporary path must be UTF-8");
    std::fs::create_dir_all(workspace.join(AUTHORITY_ROOT)).expect("authority root");
    let path = format!("{AUTHORITY_ROOT}/decision.json");
    std::fs::write(workspace.join(&path), "{}").expect("authority");

    // Act
    let result = validate_receipt_binding(&workspace, Some(&path), Some(&sha256_hex(b"original")));

    // Assert
    std::fs::remove_dir_all(&workspace).expect("cleanup");
    assert!(result
        .expect_err("edited authority")
        .contains("digest mismatch"));
}
