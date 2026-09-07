use super::BwgWorkerNvs;
use bitaxe_worker_control::QualificationLedger;
const KEY: &str = "qual_attempts";
impl BwgWorkerNvs {
    pub(crate) fn qualification_ledger(&self) -> anyhow::Result<QualificationLedger> {
        let Some(length) = self.nvs.blob_len(KEY)? else {
            return Ok(QualificationLedger::default());
        };
        if length == 0 || length > 512 {
            anyhow::bail!("qualification_budget=invalid_storage");
        }
        let mut bytes = [0_u8; 512];
        let value = self
            .nvs
            .get_blob(KEY, &mut bytes)?
            .ok_or_else(|| anyhow::anyhow!("qualification_budget=missing_storage"))?;
        let ledger: QualificationLedger = serde_json::from_slice(value)?;
        ledger.validate()?;
        Ok(ledger)
    }
    pub(crate) fn store_qualification_ledger(
        &mut self,
        ledger: &QualificationLedger,
    ) -> anyhow::Result<()> {
        ledger.validate()?;
        let bytes = serde_json::to_vec(ledger)?;
        if bytes.len() > 512 {
            anyhow::bail!("qualification_budget=storage_bound");
        }
        self.nvs.set_blob(KEY, &bytes)?;
        if self.qualification_ledger()? != *ledger {
            anyhow::bail!("qualification_budget=readback");
        }
        Ok(())
    }
}
