import { closeSync, writeSync } from "node:fs";

const args = process.argv.slice(2), option = (key) => args[args.indexOf(key) + 1];
const mode = option("--test-behavior");
let input = "";
for await (const chunk of process.stdin) input += chunk;
const value = JSON.parse(input), instanceId = Buffer.alloc(16, 3).toString("base64url");
const ready = { schema: "str005-v2-fixture-ready-runtime-v1", scope: value.scope, attemptId: value.attemptId,
  instanceId, listenIpv4: value.listenIpv4, listenPort: 33123, authorityPublicKey: Buffer.alloc(32, 4).toString("base64url") };
const connection = { schema: "str005-v2-fixture-connection-runtime-v1", scope: value.scope, attemptId: value.attemptId,
  instanceId, connectionId: Buffer.alloc(16, 5).toString("base64url"), observedAtFixtureUs: 100,
  localIpv4: value.listenIpv4, localPort: 33123, peerIpv4: value.expectedPeerIpv4, peerPort: 42123 };
if (mode === "stderr") process.stderr.write(`${value.listenIpv4}:${ready.listenPort} ${value.userIdentity}`);
process.stdout.end(`${JSON.stringify(ready)}\n${mode === "trailing" ? '{}\n' : ''}`);
writeSync(3, `${JSON.stringify(connection)}\n`); closeSync(3);
setTimeout(() => process.exit(0), 200);
