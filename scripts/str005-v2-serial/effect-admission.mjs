import { spawnSync } from "node:child_process";
import { proof } from "../str005-noise-serial/files.mjs";
import { checkedOwner, parseListenerInventory, processSnapshot, sameProcess } from "./host-resources.mjs";
import { check, object, port, sha256, uint } from "./values.mjs";

async function liveOwner(expected, operations) {
  const rows = await (operations.processSnapshot ?? processSnapshot)();
  check(rows.some(row => sameProcess(row, expected) && typeof row.state === "string" && row.state.length > 0 &&
    !/^[ZT]/u.test(row.state)), "v2_effect_supervisor_not_live");
}

function requireSupervisorListener(expected, listenerPort, operations) {
  let result;
  try {
    result = (operations.spawnSync ?? spawnSync)("/usr/sbin/lsof", ["-nP", "-a", "-p", String(expected.pid),
      `-iTCP:${listenerPort}`, "-sTCP:LISTEN", "-Fpn"], { encoding: "utf8", timeout: 5000, maxBuffer: 65536,
      stdio: ["ignore", "pipe", "pipe"], env: { PATH: "/usr/bin:/bin", LANG: "C", LC_ALL: "C" } });
    check(result && !result.error && result.status === 0 && result.signal == null && String(result.stderr ?? "") === "",
      "v2_effect_supervisor_listener");
    const rows = parseListenerInventory(String(result.stdout ?? ""));
    check(rows.length > 0 && rows.every(row => row.pid === expected.pid && row.port === listenerPort), "v2_effect_supervisor_listener");
  } catch { check(false, "v2_effect_supervisor_listener"); }
}

async function readSupervisorState(origin, context, operations) {
  const abort = new AbortController(); let timer;
  const timeout = new Promise((_, reject) => { timer = setTimeout(() => {
    abort.abort(); reject(Object.assign(Error("v2_effect_supervisor_unavailable"), { code: "v2_effect_supervisor_unavailable" }));
  }, 5000); });
  try {
    const reading = (async () => {
      const response = await (operations.fetch ?? fetch)(`${origin}/supervisor-state`, {
        method: "GET", redirect: "error", cache: "no-store", signal: abort.signal,
      });
      check(response.ok && response.body, "v2_effect_supervisor_unavailable");
      const reader = response.body.getReader(), chunks = []; let size = 0;
      try {
        for (;;) {
          const chunk = await reader.read(); if (chunk.done) break;
          size += chunk.value.byteLength; check(size <= 4096, "v2_effect_supervisor_unavailable"); chunks.push(chunk.value);
        }
      } finally { reader.releaseLock(); }
      const value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks)));
      object(value, ["scope", "phase", "failed"]);
      check(value.scope === context.scope && ["before", "candidate"].includes(value.phase) && value.failed === false,
        "v2_effect_supervisor_unavailable");
    })();
    await Promise.race([reading, timeout]);
  } catch { check(false, "v2_effect_supervisor_unavailable"); }
  finally { clearTimeout(timer); abort.abort(); }
}

/** Only an exact live supervisor created after full admission may sponsor the bounded child path. */
export async function requireLiveSupervisor(root, context, operations = {}) {
  const claim = await proof(root, "server.claim.json"), owner = await proof(root, "server-owner.json"), hash = sha256(JSON.stringify(context));
  object(claim.value, ["schema", "contextSha256"]);
  object(owner.value, ["schema", "contextSha256", "owner", "origin", "port", "atHostMs"]);
  port(owner.value.port); uint(owner.value.atHostMs); const expected = checkedOwner(owner.value.owner);
  check(claim.value.schema === "str005-v2-server-claim-v1" && claim.value.contextSha256 === hash &&
    owner.value.schema === "str005-v2-server-owner-v1" && owner.value.contextSha256 === hash &&
    owner.value.origin === `http://127.0.0.1:${owner.value.port}`, "v2_effect_supervisor_binding");
  await liveOwner(expected, operations);
  await readSupervisorState(owner.value.origin, context, operations);
  await liveOwner(expected, operations);
  requireSupervisorListener(expected, owner.value.port, operations);
  check((await proof(root, "server.claim.json")).sha256 === claim.sha256 && (await proof(root, "server-owner.json")).sha256 === owner.sha256,
    "v2_effect_supervisor_changed");
}
