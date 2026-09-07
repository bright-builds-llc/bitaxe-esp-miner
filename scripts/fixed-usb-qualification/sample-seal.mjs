import { createHash } from "node:crypto";
import { lstat, open, readFile } from "node:fs/promises";
import { isDeepStrictEqual } from "node:util";
import { resolve } from "node:path";
import { digest, exactObject, fileDigest, missing, protectedPath, readJson, requireCondition, writeNew } from "./contract.mjs";
import { validateState } from "./judge.mjs";
const SEALED = "sealed.samples.jsonl";
async function optional(path) {
  try { await protectedPath(path); return await readJson(path); }
  catch (error) { if (error.code === "ENOENT") return undefined; throw error; }
}
async function readBounded(path) {
  await protectedPath(path);
  requireCondition((await lstat(path)).size <= 33554432, "sample_journal_bound");
  return readFile(path);
}
export function parseSamples(bytes, context) {
  requireCondition(bytes.length > 0 && bytes.length <= 33554432 && bytes.at(-1) === 10, "sample_journal_shape");
  const lines = bytes.toString("utf8").slice(0, -1).split("\n");
  requireCondition(lines.length <= 512, "sample_sequence");
  const records = lines.map((line) => JSON.parse(line));
  requireCondition(records.length <= 512 && records.every((record, index) => record.sequence === index + 1), "sample_sequence");
  for (const record of records) { exactObject(record, ["sequence", "state"]); validateState(record.state, context); }
  return records;
}
export function benignClose(state, finalState) {
  return Boolean(state) && ["closing", "closed"].includes(state.status) && isDeepStrictEqual({ ...state, status: "closed" }, { ...finalState, status: "closed" });
}
async function conflict(root, context, state) {
  const path = resolve(root, "post-seal-conflict.json");
  if (!await optional(path)) {
    let closed;
    try { validateState(state, context); closed = state; } catch { closed = undefined; }
    await writeNew(path, { schema: "worker-post-seal-conflict-v1", context_sha256: digest(JSON.stringify(context)),
      state_sha256: digest(JSON.stringify(state)), ...(closed ? { state: closed } : {}) });
  }
  requireCondition(false, "post_seal_conflict");
}
export async function guardLateRecord(root, context, state) {
  await missing(resolve(root, "post-seal-conflict.json"));
  const result = await optional(resolve(root, "result.json"));
  const intent = result ? undefined : await optional(resolve(root, "sample-seal-intent.json"));
  if (!result && !intent) return false;
  const finalState = result?.receipt.final_state ?? intent.final_state;
  if (!benignClose(state, finalState)) await conflict(root, context, state);
  return true;
}
export async function writeSealedSamples(root, bytes) {
  const file = await open(resolve(root, SEALED), "wx", 0o600);
  try { await file.writeFile(bytes); await file.sync(); } finally { await file.close(); }
  return { samples_file: SEALED, samples_sha256: digest(bytes) };
}
function verifyTail(journal, prefix, receipt) {
  requireCondition(journal.subarray(0, prefix.length).equals(prefix), "sealed_journal_prefix_changed");
  const records = parseSamples(journal, receipt.context), count = parseSamples(prefix, receipt.context).length;
  requireCondition(records.slice(count).every((record) => benignClose(record.state, receipt.final_state)), "sample_seal_nonclosing_suffix");
  return { prefix_rows: count, journal_rows: records.length };
}
export function uniquePrefix(journal, expected, maybeHash) {
  const matches = [], rolling = createHash("sha256");
  let from = 0, rows = 0;
  for (let index = 0; index < journal.length; index += 1) {
    if (journal[index] !== 10) continue;
    requireCondition(++rows <= 512, "sample_sequence");
    rolling.update(journal.subarray(from, index + 1)); from = index + 1;
    const candidate = maybeHash ? maybeHash(journal.subarray(0, index + 1)) : rolling.copy().digest("hex");
    if (candidate === expected) matches.push(index + 1);
  }
  requireCondition(matches.length === 1, "sample_seal_prefix_not_unique");
  return journal.subarray(0, matches[0]);
}

export async function resultSamples(root, receipt) {
  await missing(resolve(root, "post-seal-conflict.json"));
  const journalPath = resolve(root, "iterative.samples.jsonl");
  const journal = await readBounded(journalPath);
  if (receipt.schema === "worker-iterative-result-v2") {
    requireCondition(receipt.samples_file === SEALED, "sample_seal_filename");
    const sealed = await readBounded(resolve(root, SEALED));
    requireCondition(digest(sealed) === receipt.samples_sha256, "iterative_result_samples_changed");
    verifyTail(journal, sealed, receipt);
    return parseSamples(sealed, receipt.context);
  }
  const supplement = await optional(resolve(root, "sample-seal-recovery.json"));
  if (!supplement && digest(journal) === receipt.samples_sha256) return parseSamples(journal, receipt.context);
  requireCondition(supplement, "iterative_result_samples_changed");
  exactObject(supplement, ["schema", "result_sha256", "journal_sha256", "prefix_sha256", "samples_file", "prefix_rows", "journal_rows"]);
  requireCondition(supplement.schema === "worker-sample-seal-recovery-v1" && supplement.samples_file === SEALED &&
    supplement.result_sha256 === await fileDigest(resolve(root, "result.json")) && supplement.journal_sha256 === digest(journal) &&
    supplement.prefix_sha256 === receipt.samples_sha256, "sample_seal_recovery_changed");
  const prefix = await readBounded(resolve(root, SEALED));
  requireCondition(uniquePrefix(journal, receipt.samples_sha256).equals(prefix), "sample_seal_prefix_changed");
  const counts = verifyTail(journal, prefix, receipt);
  requireCondition(counts.prefix_rows === supplement.prefix_rows && counts.journal_rows === supplement.journal_rows, "sample_seal_counts");
  return parseSamples(prefix, receipt.context);
}
export async function recoverSampleSeal(root) {
  await protectedPath(root, true); await missing(resolve(root, "post-seal-conflict.json"));
  const resultPath = resolve(root, "result.json"); await protectedPath(resultPath);
  const record = await readJson(resultPath), receipt = record.receipt;
  requireCondition(record.sha256 === digest(JSON.stringify(receipt)) && receipt.schema === "worker-iterative-result-v1", "sample_seal_legacy_result");
  const journalPath = resolve(root, "iterative.samples.jsonl"); await protectedPath(journalPath);
  const journal = await readBounded(journalPath), prefix = uniquePrefix(journal, receipt.samples_sha256);
  const counts = verifyTail(journal, prefix, receipt);
  const { validateCompletedReceipt } = await import("./iterative-preflight.mjs");
  await validateCompletedReceipt(resultPath, receipt, parseSamples(prefix, receipt.context));
  await missing(resolve(root, SEALED)); await missing(resolve(root, "sample-seal-recovery.json"));
  const resultHash = await fileDigest(resultPath);
  await writeSealedSamples(root, prefix);
  requireCondition(await fileDigest(journalPath) === digest(journal) && await fileDigest(resultPath) === resultHash, "sample_seal_concurrent_change");
  await writeNew(resolve(root, "sample-seal-recovery.json"), { schema: "worker-sample-seal-recovery-v1", result_sha256: resultHash,
    journal_sha256: digest(journal), prefix_sha256: receipt.samples_sha256, samples_file: SEALED, ...counts });
  return { recovered: true, original_result_unchanged: true, original_journal_unchanged: true, ...counts };
}
