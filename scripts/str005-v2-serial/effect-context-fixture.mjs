// Explicit synthetic process/HTTP observations for context checks, never hardware evidence.
import { resolve } from "node:path";
import { writeNew } from "../str005-noise-serial/files.mjs";
import { sha256 } from "./values.mjs";
export async function syntheticSupervisor(f) {
  const owner = { pid: 68000, pgid: 68000, ppid: 1, startedAt: "synthetic-live-supervisor", state: "S", cpuPercent: 0 };
  const contextSha256 = sha256(JSON.stringify(f.context));
  await writeNew(resolve(f.root, "server.claim.json"), { schema: "str005-v2-server-claim-v1", contextSha256 });
  await writeNew(resolve(f.root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256,
    owner, origin: "http://127.0.0.1:32125", port: 32125, atHostMs: 1 });
  const snapshot = f.operations.processSnapshot;
  f.operations.processSnapshot = async () => [...await snapshot(), owner];
  const spawn = f.operations.spawnSync;
  f.operations.spawnSync = (program, args, options) => program === "/usr/sbin/lsof"
    ? { status: 0, signal: null, stdout: `p${owner.pid}\nf10\nn127.0.0.1:32125\n`, stderr: "" }
    : spawn(program, args, options);
  f.operations.fetch = async () => new Response(JSON.stringify({ scope: f.context.scope, phase: "before", failed: false }));
  return owner;
}
