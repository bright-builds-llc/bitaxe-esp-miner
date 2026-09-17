import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { flashArguments, quoteJustArgument } from "../str005-noise-serial/operator-execution.mjs";
import { requireNoHolders, processSnapshot, sameProcess } from "../str005-noise-serial/host-resources.mjs";
import { canonical, proof } from "../str005-noise-serial/files.mjs";
import { loadEffectContext, recheckEffectAdmission } from "./context.mjs";
import { readFreshDetector } from "../str005-noise-serial/install.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { check, object, sha256 } from "./values.mjs";
export { flashArguments, quoteJustArgument };

/** Parent-pinned permits, exclusive scope assignments and fresh baseline precede every child effect. */
export async function admitExecution(root, mode, index, permit, operations = {}) {
  object(permit, ["kind", "contextSha256", "claimSha256"]);
  check(permit.kind === "execute" && ["detect", "flash"].includes(mode), "v2_execute_shape");
  const context = await loadEffectContext(root, operations);
  check(context.install_indices.includes(index) && sha256(JSON.stringify(context)) === permit.contextSha256, "v2_execute_context");
  await missing(resolve(root, "failure.json"));
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state, true);
  if (mode === "detect") {
    check(permit.claimSha256 === null, "v2_detect_claim");
    await recheckEffectAdmission(root, context, operations); return { context, argv: ["detect-ultra205"] };
  }
  const p = await proof(root, `install-${index}.claim.json`), claim = p.value;
  check(p.sha256 === permit.claimSha256 && claim.contextSha256 === permit.contextSha256 && claim.index === index &&
    claim.beforeSequence === last.sequence && claim.beforeStateSha256 === sha256(canonical(last)), "v2_execute_claim");
  const expected = flashArguments(root, context, index, claim.detector.port);
  check(JSON.stringify(expected) === JSON.stringify(claim.argv), "v2_execute_arguments");
  const owner = await proof(root, `install-${index}.host-root.json`), armed = await proof(root, `install-${index}.observer-armed.json`);
  const current = await (operations.processSnapshot ?? processSnapshot)();
  check(owner.sha256 === claim.ownerSha256 && armed.sha256 === claim.armedSha256 && sameProcess(owner.value, armed.value) &&
    owner.value.pid === (operations.pid ?? process.pid) && current.some((row) => sameProcess(row, owner.value)), "v2_execute_owner");
  const observed = await proof(root, `install-${index}.detect.observation.json`);
  const log = await readFile(resolve(root, `install-${index}.detect.stdout.log`));
  check(observed.sha256 === claim.detector.observationSha256 && sha256(log) === claim.detector.logSha256, "v2_execute_detector");
  const fields = [...log.toString("utf8").matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  for (const [name, value] of [["port", claim.detector.port], ["usb_profile", claim.detector.profile], ["physical_identity_sha256", claim.detector.physical]]) {
    const found = fields.filter((row) => row[1] === name);
    check(found.length === 1 && found[0][2] === value, "v2_execute_detector_fields");
  }
  // Hashes bind the parent's claim; freshness is rechecked at the actual child
  // dispatch boundary, after potentially slow source/process admission checks.
  const fresh = await readFreshDetector(root, index, operations);
  check(canonical(fresh) === canonical(claim.detector), "v2_execute_detector_changed");
  requireNoHolders(claim.detector.port, operations);
  await missing(resolve(root, `install-${index}`));
  await recheckEffectAdmission(root, context, operations);
  const now = (operations.unixNow ?? Date.now)(), finished = observed.value.finished_at_unix_ms;
  check(now >= finished && now - finished <= 60000, "noise_detector_stale");
  return { context, argv: expected };
}
