// Test-only protocol model; process, socket, ephemeral port and exit are real.
// Device/protocol observations from this helper are never hardware evidence.
import { createServer } from "node:net";
import { mkdir, open } from "node:fs/promises";
import { closeSync, writeSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
if (!process.send) throw Error("rehearsal_fixture_ipc_required");
const args = process.argv.slice(2), option = key => args[args.indexOf(key) + 1];
const directory = option("--private-root"), attemptId = option("--attempt-id");
const fakeRepo = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const { syntheticRoot } = await import(pathToFileURL(resolve(fakeRepo, "scripts/str005-v2-serial/cleanup-rehearsal-guard.mjs")));
await syntheticRoot(dirname(directory));
const { channelFixture } = await import(pathToFileURL(resolve(fakeRepo, "scripts/str005-v2-serial/protocol-judge.test-helper.mjs")));
const vector = channelFixture(), instanceId = vector.fixtureTerminal.instanceId, connectionId = vector.fixtureTerminal.connectionId;
let server, maybeSocket, done = false, selected = false;
const terminate = () => {
  if (done) return; done = true;
  maybeSocket?.destroy(); server?.close(); process.exit(1);
};
process.once("SIGTERM", terminate); process.once("disconnect", terminate);
setTimeout(terminate, 90000).unref();
const save = async (name, value) => {
  const file = await open(resolve(directory, name), "wx", 0o600);
  try { await file.writeFile(`${JSON.stringify(value)}\n`); await file.sync(); } finally { await file.close(); }
};
try {
  let bytes = Buffer.alloc(0);
  for await (const chunk of process.stdin) { bytes = Buffer.concat([bytes, chunk]); if (bytes.length > 4096) throw Error("input_bound"); }
  const input = JSON.parse(bytes.toString("utf8")); bytes.fill(0);
  if (input.schema !== "str005-v2-fixture-input-v1" || input.scope !== "channel" || input.attemptId !== attemptId) throw Error("input_invalid");
  await mkdir(directory, { mode: 0o700 });
  let began;
  const at = () => Math.max(0, Math.floor((performance.now() - began) * 1000));
  server = createServer(socket => {
    if (selected || socket.remoteAddress !== input.expectedPeerIpv4) { socket.destroy(); terminate(); return; }
    selected = true; maybeSocket = socket;
    const observed = { schema: "str005-v2-fixture-connection-runtime-v1", scope: "channel", attemptId,
      instanceId, connectionId, observedAtFixtureUs: at(), localIpv4: socket.localAddress, localPort: socket.localPort,
      peerIpv4: socket.remoteAddress, peerPort: socket.remotePort };
    writeSync(3, `${JSON.stringify(observed)}\n`); closeSync(3);
    socket.resume(); socket.once("error", terminate);
    socket.once("end", () => {
      const endedAt = at(); socket.end();
      server.close(async () => {
        try {
          const closedAt = at();
          vector.fixtureEvents.events.forEach((event, i) => { event.atFixtureUs = i < 6 ? Math.min(endedAt, observed.observedAtFixtureUs) : i === 6 ? endedAt : closedAt; });
          vector.fixtureTerminal.elapsedMs = Math.ceil(closedAt / 1000);
          for (const [name, value] of Object.entries({ "job.json": vector.job, "fixture-events.json": vector.fixtureEvents,
            "shares.json": vector.fixtureShares, "connection-facts.json": vector.fixtureConnectionFacts,
            "fixture-terminal.json": vector.fixtureTerminal })) await save(name, value);
          done = true; process.exit(0);
        } catch { terminate(); }
      });
    });
  });
  server.once("error", terminate);
  await new Promise(done => server.listen(0, input.listenIpv4, done));
  const address = server.address(); began = performance.now();
  process.stdout.end(`${JSON.stringify({ schema: "str005-v2-fixture-ready-runtime-v1", scope: "channel", attemptId,
    instanceId, listenIpv4: address.address, listenPort: address.port, authorityPublicKey: Buffer.alloc(32, 4).toString("base64url") })}\n`);
} catch { terminate(); }
