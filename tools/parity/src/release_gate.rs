use camino::Utf8PathBuf;

pub(crate) const DEFAULT_LICENSE_INVENTORY_PATH: &str = "docs/release/license-inventory.md";
pub(crate) const DEFAULT_PROVENANCE_PATH: &str = "docs/release/provenance-manifest.md";
pub(crate) const DEFAULT_CARGO_ABOUT_PATH: &str = "docs/release/cargo-about.html";

#[derive(Debug)]
pub(crate) struct ReleaseGateDocuments {
    pub(crate) license_inventory_path: Utf8PathBuf,
    pub(crate) license_inventory_markdown: String,
    pub(crate) provenance_path: Utf8PathBuf,
    pub(crate) provenance_markdown: String,
    pub(crate) cargo_about_path: Utf8PathBuf,
    pub(crate) maybe_cargo_about_html: Option<String>,
    pub(crate) maybe_manifest_path: Option<Utf8PathBuf>,
    pub(crate) maybe_manifest_json: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ReleaseGateReport {
    pub(crate) errors: Vec<String>,
}

impl ReleaseGateReport {
    pub(crate) fn passed(&self) -> bool {
        self.errors.is_empty()
    }
}

pub(crate) fn validate_release_gate(documents: &ReleaseGateDocuments) -> ReleaseGateReport {
    let license_sections = parse_h2_sections(&documents.license_inventory_markdown);
    let provenance_sections = parse_h2_sections(&documents.provenance_markdown);
    let mut errors = Vec::new();

    validate_required_sections(
        &mut errors,
        "license inventory",
        &documents.license_inventory_path,
        &license_sections,
        &[
            "Cargo crates",
            "Bazel and rules",
            "ESP-IDF and esp-rs",
            "Flashing tools",
            "Static assets",
            "Release artifacts",
        ],
    );
    validate_required_sections(
        &mut errors,
        "provenance manifest",
        &documents.provenance_path,
        &provenance_sections,
        &[
            "Source commit",
            "Reference commit",
            "Static asset source",
            "Recovery page source",
            "GPL review status",
            "Release artifact review",
        ],
    );
    validate_cargo_about_report(
        &mut errors,
        &documents.cargo_about_path,
        &documents.maybe_cargo_about_html,
        &license_sections,
    );
    validate_unknown_follow_up(
        &mut errors,
        "license inventory",
        &documents.license_inventory_path,
        &license_sections,
    );
    validate_unknown_follow_up(
        &mut errors,
        "provenance manifest",
        &documents.provenance_path,
        &provenance_sections,
    );
    validate_manifest_if_provided(
        &mut errors,
        documents.maybe_manifest_path.as_ref(),
        documents.maybe_manifest_json.as_deref(),
    );

    ReleaseGateReport { errors }
}

pub(crate) fn render_release_gate_report(report: &ReleaseGateReport) -> String {
    if report.passed() {
        return "release_gate: passed\n".to_owned();
    }

    let mut output = String::from("release_gate: failed\nerrors:\n");
    for error in &report.errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

#[derive(Debug)]
struct MarkdownSection {
    name: String,
    normalized_name: String,
    body: String,
    start_line: usize,
}

fn validate_required_sections(
    errors: &mut Vec<String>,
    label: &str,
    path: &Utf8PathBuf,
    sections: &[MarkdownSection],
    required_sections: &[&str],
) {
    for required_section in required_sections {
        if find_section(sections, required_section).is_some() {
            continue;
        }

        errors.push(format!(
            "{label} `{path}` missing required section `{required_section}`"
        ));
    }
}

fn validate_cargo_about_report(
    errors: &mut Vec<String>,
    cargo_about_path: &Utf8PathBuf,
    maybe_cargo_about_html: &Option<String>,
    license_sections: &[MarkdownSection],
) {
    match maybe_cargo_about_html {
        Some(contents) if !contents.trim().is_empty() => {}
        Some(_) => {
            errors.push(format!(
                "generated Cargo dependency license report `{cargo_about_path}` is empty"
            ));
        }
        None => {
            errors.push(format!(
                "generated Cargo dependency license report `{cargo_about_path}` is missing"
            ));
        }
    }

    let Some(cargo_section) = find_section(license_sections, "Cargo crates") else {
        return;
    };

    if cargo_section_references_report(cargo_section, cargo_about_path) {
        return;
    }

    errors.push(format!(
        "license inventory section `Cargo crates` does not reference `{cargo_about_path}`"
    ));
}

fn validate_unknown_follow_up(
    errors: &mut Vec<String>,
    label: &str,
    path: &Utf8PathBuf,
    sections: &[MarkdownSection],
) {
    for section in sections {
        let body = section.body.to_ascii_lowercase();
        if !body.contains("unknown") {
            continue;
        }

        let has_owner = body.contains("owner");
        let has_follow_up = body.contains("follow-up") || body.contains("follow up");
        if has_owner && has_follow_up {
            continue;
        }

        errors.push(format!(
            "{label} `{path}` section `{}` line {} says `unknown` without explicit owner and follow-up",
            section.name, section.start_line
        ));
    }
}

fn validate_manifest_if_provided(
    errors: &mut Vec<String>,
    maybe_manifest_path: Option<&Utf8PathBuf>,
    maybe_manifest_json: Option<&str>,
) {
    let Some(manifest_path) = maybe_manifest_path else {
        return;
    };

    match maybe_manifest_json {
        Some(contents) if !contents.trim().is_empty() => {}
        Some(_) => {
            errors.push(format!("package manifest `{manifest_path}` is empty"));
        }
        None => {
            errors.push(format!("package manifest `{manifest_path}` is missing"));
        }
    }
}

fn parse_h2_sections(markdown: &str) -> Vec<MarkdownSection> {
    let mut sections = Vec::new();
    let mut maybe_name: Option<String> = None;
    let mut maybe_start_line: Option<usize> = None;
    let mut body = String::new();

    for (line_index, line) in markdown.lines().enumerate() {
        if let Some(next_name) = h2_heading(line) {
            push_section(
                &mut sections,
                maybe_name.take(),
                maybe_start_line.take(),
                &mut body,
            );
            maybe_name = Some(next_name);
            maybe_start_line = Some(line_index + 1);
            continue;
        }

        if maybe_name.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }

    push_section(
        &mut sections,
        maybe_name.take(),
        maybe_start_line.take(),
        &mut body,
    );
    sections
}

fn push_section(
    sections: &mut Vec<MarkdownSection>,
    maybe_name: Option<String>,
    maybe_start_line: Option<usize>,
    body: &mut String,
) {
    let Some(name) = maybe_name else {
        return;
    };

    sections.push(MarkdownSection {
        normalized_name: normalize_section_name(&name),
        name,
        body: std::mem::take(body),
        start_line: maybe_start_line.unwrap_or(0),
    });
}

fn h2_heading(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let heading = trimmed.strip_prefix("## ")?;
    if heading.starts_with('#') {
        return None;
    }

    let name = heading.trim();
    if name.is_empty() {
        return None;
    }

    Some(name.to_owned())
}

fn find_section<'a>(
    sections: &'a [MarkdownSection],
    required_section: &str,
) -> Option<&'a MarkdownSection> {
    let normalized_required = normalize_section_name(required_section);
    sections
        .iter()
        .find(|section| section.normalized_name == normalized_required)
}

fn normalize_section_name(section: &str) -> String {
    let mut normalized = String::new();

    for character in section.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cargo_section_references_report(
    section: &MarkdownSection,
    cargo_about_path: &Utf8PathBuf,
) -> bool {
    let normalized_body = section.body.replace('\\', "/");
    let normalized_path = cargo_about_path.as_str().replace('\\', "/");
    if normalized_body.contains(&normalized_path) {
        return true;
    }

    let Some(file_name) = cargo_about_path.file_name() else {
        return false;
    };

    normalized_body.contains(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LICENSE_INVENTORY: &str = r#"
# Phase 7 Release License Inventory

## Cargo crates

- Report: `docs/release/cargo-about.html`
- Owner: release tooling
- Follow-up: regenerate before release

## Bazel and rules

- Owner: release tooling
- Follow-up: review Bzlmod inputs before release

## ESP-IDF and esp-rs

- Owner: firmware release
- Follow-up: review linked components before release

## Flashing tools

- Owner: firmware release
- Follow-up: review espflash and image generation tools before release

## Static assets

- Owner: firmware release
- Follow-up: review static asset source paths before release

## Release artifacts

- Owner: release tooling
- Follow-up: review checksums before publication
"#;

    const PROVENANCE_MANIFEST: &str = r#"
# Phase 7 Release Provenance Manifest

## Source commit

- Owner: release tooling
- Follow-up: record the source commit before release

## Reference commit

- Owner: release tooling
- Follow-up: record the pinned reference commit before release

## Static asset source

- Owner: firmware release
- Follow-up: record static source paths before release

## Recovery page source

- Owner: firmware release
- Follow-up: record recovery source path before release

## GPL review status

- Owner: release reviewer
- Follow-up: complete GPL review before release

## Release artifact review

- Owner: release reviewer
- Follow-up: review artifact checksums before release
"#;

    #[test]
    fn release_gate_passes_complete_inventory_and_provenance() {
        // Arrange
        let documents = complete_documents();

        // Act
        let report = validate_release_gate(&documents);
        let output = render_release_gate_report(&report);

        // Assert
        assert!(report.passed(), "{output}");
        assert_eq!(output, "release_gate: passed\n");
    }

    #[test]
    fn release_gate_fails_when_license_section_is_missing() {
        // Arrange
        let mut documents = complete_documents();
        documents.license_inventory_markdown =
            LICENSE_INVENTORY.replace("## Bazel and rules", "## Bazel inputs");

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("Bazel and rules")));
    }

    #[test]
    fn release_gate_fails_when_provenance_section_is_missing() {
        // Arrange
        let mut documents = complete_documents();
        documents.provenance_markdown =
            PROVENANCE_MANIFEST.replace("## GPL review status", "## GPL notes");

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("GPL review status")));
    }

    #[test]
    fn release_gate_fails_when_cargo_about_report_is_missing() {
        // Arrange
        let mut documents = complete_documents();
        documents.maybe_cargo_about_html = None;

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("cargo-about.html")));
    }

    #[test]
    fn release_gate_fails_when_cargo_about_report_is_empty() {
        // Arrange
        let mut documents = complete_documents();
        documents.maybe_cargo_about_html = Some("   \n".to_owned());

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report.errors.iter().any(|error| error.contains("empty")));
    }

    #[test]
    fn release_gate_fails_when_cargo_about_report_is_not_referenced() {
        // Arrange
        let mut documents = complete_documents();
        documents.license_inventory_markdown = LICENSE_INVENTORY.replace(
            "docs/release/cargo-about.html",
            "docs/release/other-report.html",
        );

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("Cargo crates")));
    }

    #[test]
    fn release_gate_fails_when_unknown_lacks_owner_and_follow_up() {
        // Arrange
        let mut documents = complete_documents();
        documents.license_inventory_markdown =
            LICENSE_INVENTORY.replacen("- Owner: release tooling", "- Status: unknown", 1);

        // Act
        let report = validate_release_gate(&documents);

        // Assert
        assert!(!report.passed());
        assert!(report.errors.iter().any(|error| {
            error.contains("unknown") && error.contains("owner") && error.contains("follow-up")
        }));
    }

    fn complete_documents() -> ReleaseGateDocuments {
        ReleaseGateDocuments {
            license_inventory_path: Utf8PathBuf::from(DEFAULT_LICENSE_INVENTORY_PATH),
            license_inventory_markdown: LICENSE_INVENTORY.to_owned(),
            provenance_path: Utf8PathBuf::from(DEFAULT_PROVENANCE_PATH),
            provenance_markdown: PROVENANCE_MANIFEST.to_owned(),
            cargo_about_path: Utf8PathBuf::from(DEFAULT_CARGO_ABOUT_PATH),
            maybe_cargo_about_html: Some("<html>licenses</html>".to_owned()),
            maybe_manifest_path: None,
            maybe_manifest_json: None,
        }
    }
}
