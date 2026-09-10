import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { digest, exactObject, protectedPath, readJson, requireCondition } from "./contract.mjs";
import { validateState } from "./judge.mjs";

export async function readNoMiningStates(root, context) {
  const files = (await readdir(root)).filter((file) => /^no-mining-state-[0-9]{4}\.json$/u.test(file)).sort();
  requireCondition(files.length > 0 && files.length <= 512, "no_mining_recovery_records_missing");
  const records = [];
  for (const [index, file] of files.entries()) {
    const path = resolve(root, file);
    await protectedPath(path);
    const record = await readJson(path);
    exactObject(record, ["schema", "context_sha256", "sequence", "state"]);
    requireCondition(record.schema === "fixed-usb-no-mining-state-v1" && record.context_sha256 === digest(JSON.stringify(context)) &&
      record.sequence === index + 1 && file === `no-mining-state-${String(index + 1).padStart(4, "0")}.json`, "no_mining_record_integrity");
    validateState(record.state, context);
    requireCondition(!record.state.running && record.state.renewalsConfirmed === 0 && record.state.status !== "window_loaded", "no_mining_state_required");
    records.push(record);
  }
  return records;
}
