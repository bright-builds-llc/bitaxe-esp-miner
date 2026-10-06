use super::BwgWorkerNvs;
use bitaxe_worker_control::SoakLedger;
/// Separate key so older firmware, which never reads it, keeps its own ledgers unchanged (ADR-0033).
const KEY: &str = "soak_ledger";
impl BwgWorkerNvs {
    pub(crate) fn soak_ledger(&self) -> anyhow::Result<SoakLedger> {
        let Some(length) = self.nvs.blob_len(KEY)? else {
            return Ok(SoakLedger::default());
        };
        if length == 0 || length > 512 {
            anyhow::bail!("soak_budget=invalid_storage");
        }
        let mut bytes = [0_u8; 512];
        let value = self
            .nvs
            .get_blob(KEY, &mut bytes)?
            .ok_or_else(|| anyhow::anyhow!("soak_budget=missing_storage"))?;
        let ledger: SoakLedger = serde_json::from_slice(value)?;
        ledger.validate()?;
        Ok(ledger)
    }
    pub(crate) fn store_soak_ledger(&mut self, ledger: &SoakLedger) -> anyhow::Result<()> {
        ledger.validate()?;
        let bytes = serde_json::to_vec(ledger)?;
        if bytes.len() > 512 {
            anyhow::bail!("soak_budget=storage_bound");
        }
        self.nvs.set_blob(KEY, &bytes)?;
        if self.soak_ledger()? != *ledger {
            anyhow::bail!("soak_budget=readback");
        }
        Ok(())
    }
}
