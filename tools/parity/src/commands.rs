use crate::*;

mod phase33;
mod phase35;
mod release;

pub(crate) use phase33::*;
pub(crate) use phase35::*;
pub(crate) use release::*;

pub(crate) fn run_revise_checklist_documentation_command(
    args: &ReviseChecklistDocumentationArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let outcome =
        checklist_revision::publish_current_revision(&environment.workspace_dir, &args.change_spec)
            .map_err(anyhow::Error::msg)?;
    Ok(format!(
        "checklist_revision={} affected_rows={} checklist_sha256={}",
        outcome.revision_id, outcome.affected_rows, outcome.checklist_sha256
    ))
}

pub(crate) fn run_transition_item_command(
    args: &TransitionItemArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    checklist_revision::transition_current_item(environment, args).map_err(anyhow::Error::msg)
}
