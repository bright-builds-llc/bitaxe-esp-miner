import { readdir } from 'node:fs/promises';
import { proof } from '../str005-noise-serial/files.mjs';
import { sealed } from '../str005-startup-probe/capture.mjs';
import { inspectShare } from '../str005-share-probe/inspect.mjs';
import { judge } from '../str005-share-probe/evidence.mjs';
import { requireGone, requireLsofAbsent } from '../str005-v2-serial/host-resources.mjs';
import { check } from '../str005-v2-serial/values.mjs';
/** A reset may retire retained share facts only after the actual accepted-share evidence is sealed and rejudged. */
export async function inspectPredecessor(root) {
  const seal = await sealed(root), context = (await proof(root, 'context.json')).value;
  const retained = (await proof(root, 'result.json')).value;
  check(retained.schema === 'str005-share-result-v1' && retained.complete === true && retained.accepted_share_verified === true,
    'heartbeat_share_prerequisite');
  const before = (await proof(root, 'before.json')).value, run = (await proof(root, 'run.json')).value;
  const rounds = (await readdir(root)).filter(name => /^recovery-[1-4]-session\.json$/u.test(name)).sort();
  const round = rounds.at(-1)?.split('-')[1]; check(round, 'heartbeat_share_recovery');
  const recovery = {};
  for (const key of ['state', 'ledger', 'original_budget', 'status', 'diagnostics', 'closed', 'finished'])
    recovery[key] = (await proof(root, `recovery-${round}-${key}.json`)).value;
  const owners = [];
  for (const name of ['server-owner.json', 'fixture-owner.json']) owners.push((await proof(root, name)).value.owner);
  await requireGone(owners);
  const server = (await proof(root, 'server-owner.json')).value;
  requireLsofAbsent(['-nP', `-iTCP:${server.port}`, '-sTCP:LISTEN', '-t']);
  const parts = { before, run, recovery, hostReleased: true };
  parts.shareVerified = (await inspectShare(root, context, parts)).verified;
  check(judge(parts, context).complete, 'heartbeat_share_rejudgment');
  return { context, seal, recovery, before, run };
}
