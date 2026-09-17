import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { startFixture } from "./fixture-owner.mjs";
import { sha256 } from "./values.mjs";
import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";

const helper = fileURLToPath(new URL("fixture-child.test-helper.mjs", import.meta.url));
async function fixtureCase(behavior, run) {
  const root = await mkdtemp(join(tmpdir(), "v2-fixture-owner-")), failures = [], spawns = [];
  const context = { scope: "channel", attemptId: Buffer.alloc(16, 1).toString("base64url"), fixture_binary: helper,
    fixture_sha256: sha256(await readFile(helper)) };
  const operations = {
    networkInterfaces: () => ({ test: [{ family: "IPv4", internal: false, address: "192.168.20.3", netmask: "255.255.255.0" }] }),
    spawn: (program, args, options) => {
      spawns.push({ program, args, options });
      return spawn(process.execPath, [helper, "--test-behavior", behavior, ...args], {
        ...options, env: { ...options.env, ...nodeRuntimeEnvironment() },
      });
    },
  };
  let maybeOwner;
  try {
    const launch = async () => { maybeOwner = await startFixture(root, context, "192.168.20.4", (code) => failures.push(code), operations); return maybeOwner; };
    await run({ launch, root, failures, spawns });
  } finally {
    if (maybeOwner) await maybeOwner.close();
    await rm(root, { recursive: true });
  }
}
async function written(root) {
  const contents = [];
  for (const name of await readdir(root)) contents.push(await readFile(join(root, name), "utf8"));
  return contents.join("\n");
}

test("actual child receives fixture pool inputs only through bounded private pipes", async () => {
  await fixtureCase("normal", async ({ launch, root, failures, spawns }) => {
    // Arrange / Act
    const owner = await launch(), connection = await owner.connection();
    await owner.finish();
    const persisted = await written(root), launched = JSON.stringify(spawns[0].args) + JSON.stringify(spawns[0].options.env);
    // Assert
    assert.equal(connection.peerPort, 42123);
    for (const value of ["192.168.20.3", "192.168.20.4", "33123", "42123", owner.stratum.userIdentity, owner.stratum.endpoint]) {
      assert.equal(persisted.includes(value), false);
      assert.equal(launched.includes(value), false);
    }
    assert.deepEqual(failures, []);
    assert.equal(JSON.parse(await readFile(join(root, "fixture-reap.json"))).kind, "natural_exit");
  });
});

test("raw child stderr cannot enter durable evidence even when it contains pool inputs", async () => {
  await fixtureCase("stderr", async ({ launch, root }) => {
    // Arrange / Act
    const owner = await launch();
    await assert.rejects(owner.finish(), { code: "v2_fixture_exit_unproved" });
    await owner.close();
    const persisted = await written(root);
    // Assert
    assert.equal(persisted.includes(owner.stratum.userIdentity), false);
    assert.equal(persisted.includes("192.168.20.3"), false);
    assert.equal(JSON.parse(await readFile(join(root, "fixture-exit.json"))).stderrBytes > 0, true);
  });
});

test("extra readiness records fail startup and the actual child is reaped", async () => {
  await fixtureCase("trailing", async ({ launch, root, failures }) => {
    // Act
    await assert.rejects(launch(), { code: "v2_fixture_ready_unobserved" });
    // Assert
    assert.equal(failures.includes("v2_fixture_ready_unobserved"), true);
    assert.equal(JSON.parse(await readFile(join(root, "fixture-reap.json"))).kind, "requested_stop");
  });
});
