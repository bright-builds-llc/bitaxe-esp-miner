import { processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { proof } from "../str005-noise-serial/files.mjs";
import { check, object, sha256 } from "./values.mjs";
const admissions = new Map();
/** Only the fixed private entry calls this after receiving its one-shot IPC startup. */
export async function admitOperatorParent(root, binding) {
  object(binding, ["schema", "contextSha256", "nonce"]);
  check(process.connected && binding.schema === "usb-bootstrap-measure-operator-child-start-v1" && /^[a-f0-9]{64}$/u.test(binding.nonce), "bootstrap_operator_owner");
  const locator = (await proof(`${root}.operator`, "locator.json")).value;
  const rows = await processSnapshot();
  check(locator.contextSha256 === binding.contextSha256 && locator.owner.pid === process.ppid &&
    rows.some(row => sameProcess(row, locator.owner)) && locator.owner.pid === locator.owner.pgid, "bootstrap_operator_owner");
  const claim = (await proof(`${root}.operator`, "supervisor-start.json")).value;
  check(claim.contextSha256 === binding.contextSha256 && claim.nonceSha256 === sha256(binding.nonce) && !admissions.has(root), "bootstrap_operator_owner");
  let lost = false;
  const listeners = new Set();
  const lose = () => { if (lost) return; lost = true; for (const listener of listeners) listener(); };
  process.once("disconnect", lose);
  const lifetime = {
    check() { if (!process.connected || process.ppid !== locator.owner.pid) lose(); check(!lost, "bootstrap_operator_owner"); },
    onLoss(listener) { listeners.add(listener); if (lost) listener(); else if (!process.connected || process.ppid !== locator.owner.pid) lose(); return () => listeners.delete(listener); },
    dispose() { listeners.clear(); process.removeListener("disconnect", lose); },
  };
  lifetime.check();
  admissions.set(root, { owner: locator.owner, contextSha256: binding.contextSha256, lifetime });
}
export async function requireOperatorParent(root, context) {
  if (context.schema !== "usb-bootstrap-measure-context-v1") return;
  const binding = admissions.get(root); admissions.delete(root);
  check(binding && binding.contextSha256 === sha256(JSON.stringify(context)) && process.ppid === binding.owner.pid &&
    (await processSnapshot()).some(row => sameProcess(row, binding.owner)), "bootstrap_operator_owner");
  binding.lifetime.check(); return binding.lifetime;
}
