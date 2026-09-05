use super::BwgWorkerNvs;
use bitaxe_api::acceptance_budget::AcceptanceBudget;

const KEY: &str = "accept_budget";

impl BwgWorkerNvs {
    pub(crate) fn maybe_acceptance_budget(&self) -> anyhow::Result<Option<AcceptanceBudget>> {
        let Some(length) = self.nvs.blob_len(KEY)? else {
            return Ok(None);
        };
        if length == 0 || length > 512 {
            anyhow::bail!("acceptance_budget=invalid_storage");
        }
        let mut bytes = [0_u8; 512];
        let value = self
            .nvs
            .get_blob(KEY, &mut bytes)?
            .ok_or_else(|| anyhow::anyhow!("acceptance_budget=missing_storage"))?;
        let budget: AcceptanceBudget = serde_json::from_slice(value)?;
        budget.validate()?;
        Ok(Some(budget))
    }
    pub(crate) fn store_acceptance_budget(
        &mut self,
        budget: &AcceptanceBudget,
    ) -> anyhow::Result<()> {
        budget.validate()?;
        let bytes = serde_json::to_vec(budget)?;
        if bytes.len() > 512 {
            anyhow::bail!("acceptance_budget=storage_bound");
        }
        self.nvs.set_blob(KEY, &bytes)?;
        if self.maybe_acceptance_budget()?.as_ref() != Some(budget) {
            anyhow::bail!("acceptance_budget=readback");
        }
        Ok(())
    }
}
