// Synthetic network producer for host lifecycle tests. Real Noise/TCP has Rust tests.
import { link, mkdir, readFile, unlink, writeFile } from "node:fs/promises";
const [root, attemptId] = process.argv.slice(2);
await mkdir(`${root}/fixture-run`, { mode: 0o700 });
await writeFile(`${root}/fixture-run/ready.json.pending`, JSON.stringify({ schema: "noise-serial-fixture-ready-v1", attemptId,
  listenIpv4: "192.168.1.20", listenPort: 32124, authorityPublicKey: "A".repeat(43) }), { flag: "wx", mode: 0o600 });
await link(`${root}/fixture-run/ready.json.pending`, `${root}/fixture-run/ready.json`);
await unlink(`${root}/fixture-run/ready.json.pending`);
while (true) {
  try { await readFile(`${root}/synthetic-proof.ready`); break; }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  await new Promise((done) => setTimeout(done, 10));
}
await writeFile(`${root}/fixture-run/terminal.json`, JSON.stringify({ schema: "noise-serial-fixture-terminal-v1", attemptId, outcome: "accepted",
  failure: null, elapsedMs: 1000, expectedPeerConnectionCount: 1, unexpectedPeerCount: 0, candidateOverflow: false,
  selectedIndex: 0, candidates: [{ remotePort: 54321, actOneBytes: 64, readOutcome: "complete" }], actTwoBytesWritten: 234,
  proofBytesReceived: 22, extraBytesReceived: 0, encryptedProofExact: true, peerClosed: true, socketClosed: true }), { flag: "wx", mode: 0o600 });
