import { readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { proof } from "../str005-noise-serial/files.mjs";
import { check, list, object } from "./values.mjs";
import { selectedFacts } from "./execution-receipts.mjs";

/** Read atomically published fixture completions; absence is not peer delivery. */
export async function completedShares(root) {
  const names = (await readdir(resolve(root, "fixture-run"))).filter(name => /^share-.*\.json$/u.test(name)).sort();
  check(names.length <= 1024, "v2_live_share_bound");
  const values = [];
  for (const [index, name] of names.entries()) {
    check(name === `share-${String(index + 1).padStart(4, "0")}.json`, "v2_live_share_gap");
    const row = await proof(root, `fixture-run/${name}`);
    object(row.value, ["connectionId", "shares"]); list(row.value.shares, 1);
    check(row.value.shares.length === 1, "v2_live_share_shape");
    values.push({ file: name, ...row, share: row.value.shares[0] });
  }
  return values;
}
/** Selection is possible only from a native observed ACK and an independent completed fixture row. */
export async function inspectLiveSelection(root, context, record) {
  check(context.scope === "share" && record.scope === "share" && record.attemptId === context.attemptId &&
    record.firstFailure === null && record.secondaryFailures.length === 0, "v2_live_selection_scope");
  const candidates = record.shareFacts.filter(fact => fact.ackAtDeviceUs !== null).sort((a, b) => a.ackAtDeviceUs - b.ackAtDeviceUs || a.submissionSequence - b.submissionSequence);
  if (candidates.length === 0) return null;
  const jobProof = await proof(root, "fixture-run/job.json"), rows = await completedShares(root);
  for (const fact of candidates) {
    const matches = rows.filter(row => row.share.submission?.sequenceNumber === fact.submissionSequence);
    check(matches.length <= 1, "v2_live_share_duplicate");
    if (!matches.length) return null; // Do not skip the first ACK just because its atomic file has not appeared yet.
    const row = matches[0]; check(row.value.connectionId === jobProof.value.connectionId, "v2_live_share_connection");
    selectedFacts(record, jobProof.value, fact, row.share);
    return { record, fact, share: row.share, job: jobProof.value, jobSha256: jobProof.sha256, fixtureShareFile: row.file, fixtureShareSha256: row.sha256 };
  }
  return null;
}
