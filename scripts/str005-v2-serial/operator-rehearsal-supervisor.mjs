import { createServer } from "node:net";
import { once } from "node:events";
import { link, unlink } from "node:fs/promises";
import { resolve } from "node:path";
import { loadOperatorContext } from "./context.mjs";
import { admitOperatorParent, requireOperatorParent } from "./operator-parent.mjs";
import { processSnapshot } from "../str005-noise-serial/host-resources.mjs";
import { writeNew, digest, proof } from "../str005-noise-serial/files.mjs";
const root = resolve(process.argv[2]);
process.once("message", async input => {
  const context = await loadOperatorContext(root); await admitOperatorParent(root, input.binding);
  let startupLoss = false;
  try { startupLoss = (await proof(root, "synthetic-startup-loss.json")).value.enabled === true; }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (startupLoss) {
    // The polling parent must not observe a partially written readiness receipt.
    const pending = resolve(root, "synthetic-startup-owner.pending.json");
    await writeNew(pending, (await processSnapshot()).find(row => row.pid === process.pid));
    await link(pending, resolve(root, "synthetic-startup-owner.json"));
    await unlink(pending);
    const deadline = setTimeout(() => process.exit(1), 5000);
    await once(process, "disconnect"); clearTimeout(deadline);
  }
  await requireOperatorParent(root, context);
  const server = createServer(socket => socket.end("synthetic-supervisor-live\n"));
  await new Promise(done => server.listen(0, "127.0.0.1", done));
  const owner = (await processSnapshot()).find(row => row.pid === process.pid), port = server.address().port;
  await writeNew(resolve(root, "server-owner.json"), { schema: "str005-v2-server-owner-v1", contextSha256: digest(JSON.stringify(context)), owner,
    origin: `http://127.0.0.1:${port}`, port, atHostMs: 0 });
  process.stdout.write(`qualification_url=http://127.0.0.1:${port}/\n`);
  const watchdog = setTimeout(() => { server.close(); process.disconnect(); }, 15000);
  process.once("SIGTERM", () => { clearTimeout(watchdog); server.close(() => { process.disconnect(); }); });
});
