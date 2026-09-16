import { dirname, resolve } from "node:path";
import { exactObject, missing } from "../fixed-usb-qualification/contract.mjs";
import { verifyEffectInputs } from "./context.mjs";
import { baseline, readJournal } from "./journal.mjs";
import { requireNoHolders, processSnapshot, sameProcess } from "./host-resources.mjs";
import { canonical, check, digest, privateRoot, proof } from "./files.mjs";

export function flashArguments(root, context, index, port) {
  return ["flash-monitor", "--board", "205", "--port", port, "--manifest", context.manifest,
    "--evidence-dir", resolve(root, `install-${index}`), "--capture-timeout-seconds", "30", "--redact-evidence"];
}
export function quoteJustArgument(value) { return `'${value.replaceAll("'", "'\\''")}'`; }
/** Validate late execution against parent-pinned hashes, never self-rehashed disk claims. */
export async function admitExecution(root, mode, index, permit, operations = {}) {
  exactObject(permit, ["kind", "contextSha256", "claimSha256"]);
  check(permit.kind === "execute" && Number.isInteger(index) && index >= 0 && index <= 4 && ["detect", "flash"].includes(mode), "noise_execute_shape");
  root = await privateRoot(root);
  const record = (await proof(root, "context.json")).value;
  exactObject(record, ["context", "sha256"]);
  check(record.sha256 === permit.contextSha256 && digest(JSON.stringify(record.context)) === permit.contextSha256, "noise_execute_context_changed");
  const context = record.context;
  const assigned = (await proof(dirname(root), `ordinal-${context.ordinal}.json`)).value;
  check(assigned.root === root && assigned.context_sha256 === permit.contextSha256, "noise_execute_assignment_changed");
  await (operations.verifySources ?? verifyEffectInputs)(context, operations);
  await missing(resolve(root, "failure.json"));
  const rows = await readJournal(root, context), last = rows.at(-1); baseline(last?.state, true);
  if (mode === "detect") { check(permit.claimSha256 === null, "noise_detector_claim"); return { context, argv: ["detect-ultra205"] }; }
  const proofClaim = await proof(root, `install-${index}.claim.json`), claim = proofClaim.value;
  check(proofClaim.sha256 === permit.claimSha256 && claim.contextSha256 === permit.contextSha256 && claim.index === index &&
    claim.beforeSequence === last.sequence && claim.beforeStateSha256 === digest(canonical(last)), "noise_execute_claim_changed");
  const expected = flashArguments(root, context, index, claim.detector.port);
  check(JSON.stringify(expected) === JSON.stringify(claim.argv), "noise_execute_argv_changed");
  const owner = await proof(root, `install-${index}.host-root.json`), armed = await proof(root, `install-${index}.observer-armed.json`);
  const processes = await (operations.processSnapshot ?? processSnapshot)();
  check(owner.sha256 === claim.ownerSha256 && armed.sha256 === claim.armedSha256 && sameProcess(owner.value, armed.value) &&
    owner.value.pid === (operations.pid ?? process.pid) && processes.some((row) => sameProcess(row, owner.value)), "noise_execute_owner_changed");
  const observation = await proof(root, `install-${index}.detect.observation.json`);
  check(observation.sha256 === claim.detector.observationSha256, "noise_execute_detector_changed");
  const { readFile } = await import("node:fs/promises");
  const log = await readFile(resolve(root, `install-${index}.detect.stdout.log`));
  check(digest(log) === claim.detector.logSha256, "noise_execute_detector_changed");
  const fields = [...log.toString("utf8").matchAll(/^([a-z][a-z0-9_]*): (.+)$/gmu)];
  for (const [key, expectedValue] of [["port", claim.detector.port], ["usb_profile", claim.detector.profile], ["physical_identity_sha256", claim.detector.physical]]) {
    const values = fields.filter((row) => row[1] === key);
    check(values.length === 1 && values[0][2] === expectedValue, "noise_execute_detector_fields");
  }
  requireNoHolders(claim.detector.port, operations);
  await missing(resolve(root, `install-${index}`));
  await missing(resolve(root, "failure.json"));
  return { context, argv: expected };
}
