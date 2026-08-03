use crate::*;

#[derive(Serialize)]
pub(crate) struct Phase33ClassificationOutput {
    status: &'static str,
    category: &'static str,
    session: Option<String>,
    boot_ordinal: Option<u64>,
    device_url: Option<String>,
}

pub(crate) fn run_phase33_classify_command(
    args: VerifySettingsDurabilityArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let path = environment.workspace_path(&args.trace);
    let bytes = std::fs::read(path.as_std_path())
        .with_context(|| format!("failed to read Phase 33 trace {path}"))?;
    let start = usize::try_from(args.start_byte).context("start-byte does not fit usize")?;
    if start > bytes.len() {
        bail!("start-byte exceeds Phase 33 trace length");
    }
    let text = String::from_utf8_lossy(&bytes[start..]);
    let expected = || -> Result<(&str, u64)> {
        Ok((
            args.expected_session
                .as_deref()
                .context("expected-session is required for this mode")?,
            args.expected_ordinal
                .context("expected-ordinal is required for this mode")?,
        ))
    };

    let classified = match args.mode {
        Phase33ClassifyMode::Baseline => classify_phase33_baseline(&text),
        Phase33ClassifyMode::Delivery => {
            let (session, ordinal) = expected()?;
            classify_phase33_delivery(&text, session, ordinal).map(|()| {
                bitaxe_api::phase33_evidence::Phase33BootEvidence {
                    session: session.to_owned(),
                    boot_ordinal: ordinal,
                    device_url: String::new(),
                }
            })
        }
        Phase33ClassifyMode::PostRestart => {
            let (session, ordinal) = expected()?;
            classify_phase33_post_restart(&text, session, ordinal)
        }
    };

    let output = match classified {
        Ok(evidence) => Phase33ClassificationOutput {
            status: "passed",
            category: "none",
            session: Some(evidence.session),
            boot_ordinal: Some(evidence.boot_ordinal),
            device_url: (!evidence.device_url.is_empty()).then_some(evidence.device_url),
        },
        Err(error) => Phase33ClassificationOutput {
            status: "failed",
            category: error.category,
            session: None,
            boot_ordinal: None,
            device_url: None,
        },
    };
    serde_json::to_string(&output).context("failed to encode Phase 33 classification")
}
