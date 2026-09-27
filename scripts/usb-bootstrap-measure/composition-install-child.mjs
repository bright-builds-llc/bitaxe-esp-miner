// Fixed synthetic hardware boundary. Root/marker checks prevent use with real attempts.
import { once } from "node:events";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { proof, writeNew } from "../str005-noise-serial/files.mjs";
import { hostTimingFile, CONTEXT_V3 } from "./values.mjs";
import { admit } from "./install.mjs";
import { testOperations } from "./test-fixture.mjs";
import { syntheticCapture, syntheticTiming } from "./measurement.fixture.mjs";
const [root, mode, index] = process.argv.slice(2), operations = await testOperations(root);
if (index !== "0" || !process.connected) throw Error("test child scope");
process.send({ ready: true }); const [permit] = await once(process, "message");
const { context } = await admit(root, mode, permit, operations);
if (mode === "detect") process.stdout.write(`port: /dev/cu.synthetic\nusb_profile: serial_jtag_runtime\nphysical_identity_sha256: ${"c".repeat(64)}\n`);
else {
  const mode = (await proof(root, "test-seam.json")).value.captureMode;
  const { log, verdict } = syntheticCapture(context, mode === "failed");
  await mkdir(resolve(root, "install-0"), { mode: 0o700 });
  const { retain } = await import("../str005-noise-serial/files.mjs");
  await retain(resolve(root, "install-0/flash-monitor.log"), Buffer.from(log));
  await writeNew(resolve(root, "install-0/flash-command-evidence.json"), verdict);
  if (mode !== "missing") await writeNew(resolve(root, `install-0/${hostTimingFile(context)}`), syntheticTiming([CONTEXT_V3, "usb-bootstrap-measure-context-v4"].includes(context.schema) ? 2 : 1));
  if (mode === "failed") process.exitCode = 1;
}
process.disconnect();
