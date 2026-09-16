import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { fixture } from "./test-fixture.mjs";
import { pipelineBrowser } from "./pipeline-browser.test-helper.mjs";
import { installCandidate, observeOwnedExit } from "./operator.mjs";
import { processSnapshot } from "./host-resources.mjs";
import { readJournal } from "./journal.mjs";
import { recordCleanup } from "./cleanup.mjs";
import { finalize, review } from "./finalize.mjs";
import { digest } from "./files.mjs";
import { quoteJustArgument } from "./operator-execution.mjs";

test("real client/server/operator lifecycle reaches the independent reader with synthetic device data", { timeout: 30000 }, async (t) => {
  const f = await fixture(t), bin = resolve(f.base, "bin"); await mkdir(bin, { mode: 0o700 });
  const script = resolve(bin, "fake-just.mjs");
  await writeFile(script, await readFile(new URL("./fake-just.test-helper.mjs", import.meta.url)), { mode: 0o600 });
  await writeFile(resolve(bin, "just"), `#!/bin/sh\nexec ${quoteJustArgument(process.execPath)} ${quoteJustArgument(script)} "$@"\n`, { mode: 0o700 });
  f.operations.path = `${bin}:${process.env.PATH}`;
  f.operations.childProgram = fileURLToPath(new URL("./operator-child.test-helper.mjs", import.meta.url));
  f.operations.processSnapshot = processSnapshot;
  const child = spawn(process.execPath, [fileURLToPath(new URL("./pipeline-server.test-helper.mjs", import.meta.url)), f.root],
    { detached: true, stdio: ["ignore", "pipe", "pipe", "ipc"] });
  let stderr = ""; child.stderr.on("data", (bytes) => { stderr += bytes.toString("utf8"); });
  t.after(() => { if (child.exitCode === null && child.signalCode === null) process.kill(-child.pid, "SIGKILL"); });
  const [{ origin }] = await once(child, "message");
  const owner = JSON.parse(await readFile(resolve(f.root, "server-owner.json"))).owner;
  const parentExit = observeOwnedExit(child, digest(JSON.stringify(f.context)), owner, "supervisor");
  const page = await pipelineBrowser(f, origin); await page.connect();
  await page.driver.recordAccounting("before-install");
  await page.api.close(); await page.driver.flush();
  async function install(index) {
    try { await installCandidate(f.root, index, f.operations); }
    catch (error) {
      const logs = await Promise.all(["failure.json", `install-${index}.detect.stderr.log`, `install-${index}.stderr.log`].map(async (name) => {
        try { return await readFile(resolve(f.root, name), "utf8"); }
        catch (readError) { if (readError.code !== "ENOENT") throw readError; return "absent"; }
      }));
      assert.fail(`${error.code}: ${logs.join("\n")}`);
    }
  }
  await install(0);
  await page.driver.configureCandidate(); await page.connect();
  for (let index = 1; index <= 4; index++) {
    await page.api.close(); await page.driver.flush();
    await install(index);
    await page.driver.configureCandidate(); await page.connect(); await page.driver.recordCycle(index);
  }
  try { await page.driver.run(); }
  catch (error) { assert.fail(`${error.message}: ${stderr}`); }
  await page.connect(); await page.driver.restoreAndRecord();
  await page.api.close(); await page.driver.flush();
  parentExit.markStopRequested(); child.send("stop");
  const supervisor = await parentExit.receipt(); assert.equal(supervisor.code, 0, stderr);
  const last = (await readJournal(f.root, f.context)).at(-1);
  await recordCleanup(f.root, f.context, { supervisor, browser: { schema: "noise-serial-browser-closure-v2", source: "parent-observed",
    contextSha256: digest(JSON.stringify(f.context)), closed: true, lastSequence: last.sequence,
    lastStateSha256: digest(JSON.stringify(last)), observedAtUnixMs: Date.now() } }, f.operations);
  const result = await finalize(f.root, `${f.root}.cleanup/receipt.json`, f.operations);
  assert.equal(result.status, "passed");
  assert.deepEqual(await review(f.root, f.operations), result);
});
