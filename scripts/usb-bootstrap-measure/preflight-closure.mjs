import { link, unlink, lstat, realpath } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { missing } from "../fixed-usb-qualification/contract.mjs";
import { canonical, privateRoot, proof, writeNew } from "../str005-noise-serial/files.mjs";
import { createChecker, validateChecker } from "./checker-identity.mjs";
import { PREFLIGHT_AMENDMENT, check, object } from "./values.mjs";
async function checkedReceipt(path) {
  const stat = await lstat(path);
  check(stat.isFile() && !stat.isSymbolicLink() && stat.uid === process.getuid() && stat.nlink === 1 && (stat.mode & 0o777) === 0o600 && await realpath(path) === path,
    "bootstrap_closure_private_path");
}
const NONCLAIMS = ["no_hardware_execution_proof", "no_fresh_device_accounting", "no_qualification_credit", "no_continuation_authority"];
export const preflightClosurePath = root => `${root}.preflight-closure.json`;
async function corpus(root, operations) {
  return (operations.verifyPreflightCorpus ?? (await import("./preflight-corpus.mjs")).verifyPreflightCorpus)(root, operations);
}
async function ancestry(facts, operations) {
  if (operations.verifyClosureAncestry) return operations.verifyClosureAncestry(facts);
  const inspect = operations.predecessor ?? (await import("./context.mjs")).predecessor;
  const previous = await inspect(facts.context.predecessor.root, operations, false);
  check(canonical(previous.originalCampaign) === canonical(facts.context.originalCampaign) && canonical(previous.expectedAccounting) === canonical(facts.context.expectedAccounting), "bootstrap_closure_ancestry");
}
function projection(facts, checkerIdentity) {
  return { schema: "usb-bootstrap-measure-preflight-closure-v1", amendmentSha256: PREFLIGHT_AMENDMENT.sha256,
    failedRoot: facts.root, failedContextSha256: facts.contextSha256, assignment: facts.assignment, failedSourceCommit: facts.failedSourceCommit,
    status: "unverified", classification: "interrupted_before_effects", earliestCause: { source: "parent-observed", code: "bootstrap_operation_failed", observationSha256: facts.commandLog.sha256 },
    diagnosedCause: { source: "read-only-reproduction", code: "noise_forbidden_inventory_name", repositoryPath: "scripts/phase28.1.1.1-synthetic-pool-credentials.mjs" },
    inspectedInputs: facts.inspectedInputs, checkerIdentity, effectsObserved: false, nonClaims: NONCLAIMS };
}
function sameFacts(a, b) {
  check(canonical({ context: a.context, assignment: a.assignment, log: a.commandLog, files: a.inspectedInputs }) ===
    canonical({ context: b.context, assignment: b.assignment, log: b.commandLog, files: b.inspectedInputs }), "bootstrap_closure_input_changed");
}
const result = (facts, path, digest, checkerIdentity) => ({ ...facts, closurePath: path, closureSha256: digest, checkerIdentity, status: "unverified", classification: "interrupted_before_effects", effectsObserved: false });
/** The only mutation is exclusive publication of a sibling receipt after independent inspection. */
export async function closePreflight(root, operations = {}) {
  check(root === resolve(root), "bootstrap_closure_path"); await privateRoot(dirname(root));
  const path = preflightClosurePath(root), pending = `${path}.pending`; await missing(path); await missing(pending);
  const facts = await corpus(root, operations); await ancestry(facts, operations);
  const checker = await createChecker(facts.context, operations), value = projection(facts, checker);
  sameFacts(facts, await corpus(root, operations));
  await writeNew(pending, value); await operations.beforeClosurePublish?.();
  sameFacts(facts, await corpus(root, operations));
  check(canonical(await createChecker(facts.context, operations)) === canonical(checker), "bootstrap_closure_checker_changed");
  await link(pending, path); await unlink(pending);
  await checkedReceipt(path);
  const stored = await proof(dirname(root), path); check(canonical(stored.value) === canonical(value), "bootstrap_closure_changed");
  return result(facts, path, stored.sha256, checker);
}
/** Full historical verification is read-only; it never reruns the failed command or repairs its root. */
export async function inspectPreflightClosure(path, operations = {}, { deep = true } = {}) {
  check(typeof path === "string" && path === resolve(path) && path.endsWith(".preflight-closure.json"), "bootstrap_closure_path");
  const root = path.slice(0, -".preflight-closure.json".length); await privateRoot(dirname(root)); await missing(`${path}.pending`);
  await checkedReceipt(path);
  const stored = await proof(dirname(root), path), value = stored.value;
  object(value, ["schema", "amendmentSha256", "failedRoot", "failedContextSha256", "assignment", "failedSourceCommit", "status", "classification", "earliestCause", "diagnosedCause", "inspectedInputs", "checkerIdentity", "effectsObserved", "nonClaims"]);
  const facts = await corpus(root, operations);
  check(canonical(value) === canonical(projection(facts, value.checkerIdentity)), "bootstrap_closure_changed");
  await validateChecker(value.checkerIdentity, facts.context, operations);
  if (deep) await ancestry(facts, operations);
  check((await proof(dirname(root), path)).sha256 === stored.sha256, "bootstrap_closure_changed");
  return result(facts, path, stored.sha256, value.checkerIdentity);
}
export async function reviewPreflight(root, operations = {}) { return inspectPreflightClosure(preflightClosurePath(root), operations); }
