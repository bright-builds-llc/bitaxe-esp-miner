import assert from "node:assert/strict";
import { chmod, mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { ApiCommandEffectsError, captureApiCommandEffects } from "./api-command-effects.js";
import { createFakeProcessPort } from "./process.js";

test("wrapper capture inside the private root fails before any child launches", async () => {
  // Arrange
  const root = await mkdtemp(path.join(os.tmpdir(), "api-command-effects-preflight-"));
  const manifest = path.join(root, "package.json");
  const credentials = path.join(root, "wifi.json");
  const privateRoot = path.join(root, "attempt");
  await writeFile(manifest, "{}\n");
  await writeFile(credentials, "{}\n");
  await mkdir(privateRoot, { mode: 0o700 });
  await chmod(privateRoot, 0o700);
  await writeFile(path.join(privateRoot, "wrapper.stdout"), "", { mode: 0o600 });
  let childLaunchCount = 0;
  const processPort = createFakeProcessPort(async () => {
    childLaunchCount += 1;
    return { exitCode: 0, stdout: "", stderr: "", timedOut: false };
  });

  // Act
  const error = await captureApiCommandEffects(root, {
    privateRoot,
    packageManifest: manifest,
    wifiCredentials: credentials,
    port: "/dev/private-sensitive-port",
    projection: path.join(root, "projection.json"),
    durationSeconds: 600,
  }, processPort, "fixture", "flash", "device-session", () => undefined)
    .then(() => undefined, (caught: unknown) => caught);

  // Assert
  assert(error instanceof ApiCommandEffectsError);
  assert.equal(error.category, "evidence_invalid");
  assert.equal(childLaunchCount, 0);
});
