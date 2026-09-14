import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createNoMiningSupervisor } from "./no-mining-server.mjs";
import { readNoMiningStates } from "./no-mining-records.mjs";
import { body, send } from "./http.mjs";
import { digest, exactObject, readJson, requireCondition, writeNew, QualificationError } from "./contract.mjs";
import { RESET_ORIGIN_CATEGORIES, parseResetOriginDiagnostic } from "./reset-origin-observation.mjs";
import { loadResetOriginContext, verifyResetOriginFrozen } from "./reset-origin-context.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const FAILURE_CATEGORIES = new Set(["control_failure", "serial_rx_failure", "serial_tx_failure", "network_failure", "startup_failure", "storage_http_failure"]);
export function selectResetOriginDiagnostics(input) {
  exactObject(input, ["schema", "observations"]);
  requireCondition(input.schema === "worker-diagnostic-export-v1" && Array.isArray(input.observations) && input.observations.length <= 40, "reset_origin_export_shape");
  const selected = [];
  for (const value of input.observations) {
    requireCondition(value && typeof value === "object" && !Array.isArray(value), "reset_origin_export_shape");
    requireCondition(!FAILURE_CATEGORIES.has(value.category), "reset_origin_failure_diagnostic");
    if (RESET_ORIGIN_CATEGORIES.includes(value.category)) {
      const parsed = parseResetOriginDiagnostic(value);
      requireCondition(parsed.category !== "startup" || (parsed.state !== "failed" && parsed.first_failure === "none"), "reset_origin_startup_failure");
      selected.push(parsed); continue;
    }
    if (value.category === "memory") continue;
    if (value.category === "worker_admission") {
      requireCondition(value.stage === "idle" && value.first_failure === "none", "reset_origin_unexpected_worker_activity");
      continue;
    }
    if (value.category === "worker_preparation_receipt") {
      requireCondition(["corrupt", "unavailable"].includes(value.status), "reset_origin_preparation_receipt_review_required");
      continue;
    }
    throw new QualificationError("reset_origin_unsupported_diagnostic");
  }
  return selected;
}
function ready(state) {
  requireCondition(state.status === "ready" && state.connected && !state.running && !state.failure && state.deviceLeaseInactive &&
    state.deviceBaselineConfirmed === true && !state.serialOwnershipReleased && state.renewalsConfirmed === 0 &&
    state.preservation?.device_identity_match && state.preservation.settings_match && state.preservation.authorization_high_water_match &&
    state.preservation.mine_on_boot === false, "reset_origin_baseline");
}

/** Read-only browser observation; there is no reset, flash, signer or work route. */
export async function createResetOriginSupervisor(options, operations = {}) {
  requireCondition(options.authorityDirectory === undefined && options.poolCredentials === undefined && options.context === undefined, "reset_origin_credentials_forbidden");
  const root = resolve(options.privateRoot), context = await loadResetOriginContext(root, { operations });
  await writeNew(resolve(root, "reset-origin-server-claim.json"), {
    schema: "fixed-usb-reset-origin-server-claim-v1", context_sha256: digest(JSON.stringify(context)),
  });
  const verify = () => verifyResetOriginFrozen(root, context, operations);
  const inner = context.no_mining_context;
  const server = await createNoMiningSupervisor({ privateRoot: root, context: inner, bun: options.bun }, { verifyFrozen: verify });
  const [baseHandler] = server.listeners("request"); server.removeListener("request", baseHandler);
  let phase = "unprimed", prime, primedAt = 0, sequence = 0, startedAt = 0, queue = Promise.resolve(), captureTimer;
  const now = operations.now ?? (() => Math.floor(performance.now()));
  const contextHash = digest(JSON.stringify(context));
  async function fail(code) {
    if (phase === "failed") return;
    phase = "failed";
    clearTimeout(captureTimer);
    await writeNew(resolve(root, "reset-origin-failure.json"), { schema: "fixed-usb-reset-origin-failure-v1", context_sha256: contextHash, code });
  }
  async function current() {
    const rows = await readNoMiningStates(root, inner), last = rows.at(-1); ready(last.state); return last;
  }
  async function mutate(path, input) {
    if (path === "/reset-origin/failure") {
      exactObject(input, ["code"]); requireCondition(input.code === "observer_client_failed", "reset_origin_failure_shape");
      await fail(input.code); return { failure_recorded: true };
    }
    requireCondition(phase !== "failed", "reset_origin_failed");
    if (path === "/diagnostic-export") {
      requireCondition(["unprimed", "active"].includes(phase), "reset_origin_export_phase");
      const observations = selectResetOriginDiagnostics(input), hostMonotonicMs = now();
      requireCondition(sequence <= context.observation_policy.maximum_batches &&
        (phase !== "active" || hostMonotonicMs - startedAt <= context.observation_policy.maximum_span_ms), "reset_origin_capture_bound");
      const file = `diagnostic-export-${String(sequence).padStart(4, "0")}.json`;
      await writeNew(resolve(root, file), { schema: "fixed-usb-reset-origin-batch-v1", context_sha256: contextHash, sequence, hostMonotonicMs, observations });
      if (phase === "unprimed") { prime = observations; primedAt = hostMonotonicMs; phase = "primed"; }
      sequence++;
      return { diagnostic_export_saved: true, review_file: file };
    }
    exactObject(input, []);
    if (path === "/reset-origin/start") {
      // The just-recorded accounting boundary already verified frozen sources;
      // don't age the prime snapshot by rehashing the historical chain again.
      requireCondition(phase === "primed", "reset_origin_start_phase");
      const before = await readJson(resolve(root, "no-mining-accounting-before.json"));
      await current(); startedAt = now();
      requireCondition(startedAt - primedAt <= context.observation_policy.maximum_gap_ms &&
        prime.some(d => d.category === "boot") && prime.some(d => d.category === "runtime_identity" && d.firmware_commit === context.firmware_commit && d.app_elf_sha256 === context.app_elf_sha256) &&
        prime.some(d => d.category === "startup" && d.stage === "runtime_ready" && d.state === "complete" && d.first_failure === "none") &&
        prime.some(d => d.category === "storage_http_status" && d.spiffs_available === "true" && d.http_ready === "true"), "reset_origin_prime_incomplete");
      await writeNew(resolve(root, "reset-origin-start.json"), { schema: "fixed-usb-reset-origin-start-v1", context_sha256: contextHash,
        hostMonotonicMs: startedAt, observed_sequence: before.observed_sequence, primeObservations: prime });
      phase = "active";
      captureTimer = setTimeout(() => {
        void fail("reset_origin_capture_timeout").catch(() => console.error("reset_origin_failure_persistence_failed"));
      }, context.observation_policy.maximum_span_ms);
      return { observation_started: true, target_ms: 130000, maximum_ms: 135000 };
    }
    requireCondition(path === "/reset-origin/end" && phase === "active", "reset_origin_end_phase");
    const endedAt = now(); requireCondition(endedAt - startedAt >= 120000 && endedAt - startedAt <= 135000, "reset_origin_capture_bound");
    phase = "ending"; clearTimeout(captureTimer);
    await verify(); const last = await current();
    await writeNew(resolve(root, "reset-origin-end.json"), { schema: "fixed-usb-reset-origin-end-v1", context_sha256: contextHash,
      hostMonotonicMs: endedAt, observed_sequence: last.sequence });
    phase = "ended"; clearTimeout(captureTimer);
    return { observation_finished: true, measured_batches: sequence - 1, recovery_authorized: false };
  }
  server.on("request", (request, response) => {
    handle(request, response).catch(async error => {
      const code = error instanceof QualificationError ? error.code : "reset_origin_operation_failed";
      try { await fail(code); } catch { console.error("reset_origin_failure_persistence_failed"); response.destroy(); return; }
      if (response.headersSent) response.destroy(); else send(response, 400, { error: code });
    });
  });
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    requireCondition(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    if (request.method === "POST") requireCondition(request.headers.origin === origin ||
      (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"), "origin_rejected");
    const path = new URL(request.url, origin).pathname;
    if (request.method === "GET" && ["/", `/${context.gate_page_relative_path}`].includes(path)) {
      const bytes = await readFile(resolve(inner.gate_root, context.gate_page_relative_path));
      requireCondition(digest(bytes) === context.gate_page_sha256, "served_asset_drift");
      return send(response, 200, `${bytes}\n<script type="module" src="/no-mining-client.mjs"></script>\n<script type="module" src="/reset-origin-client.mjs"></script>`, "text/html");
    }
    if (request.method === "GET" && path === "/reset-origin-client.mjs") {
      const bytes = await readFile(resolve(HERE, "reset-origin-client.mjs"));
      requireCondition(digest(bytes) === context.qualification_driver.client_sha256, "served_asset_drift");
      return send(response, 200, bytes, "text/javascript");
    }
    if (request.method === "POST" && ["/diagnostic-export", "/reset-origin/start", "/reset-origin/end", "/reset-origin/failure"].includes(path)) {
      const input = await body(request), pending = queue.then(() => mutate(path, input));
      queue = pending.then(() => undefined, () => undefined);
      return send(response, 200, await pending);
    }
    const allowedGet = ["/context", "/supervisor-state", "/no-mining-client.mjs", "/dist/worker-serial-acceptance/worker-serial-acceptance.js"];
    const allowedPost = ["/activate", "/record", "/original-budget-context", "/accounting"];
    if ((request.method === "GET" ? allowedGet : request.method === "POST" ? allowedPost : []).includes(path)) return baseHandler(request, response);
    send(response, 404, { error: "route_unavailable" });
  }
  server.closeQualificationResources = async () => {
    await queue;
    clearTimeout(captureTimer);
    if (!["ended", "failed"].includes(phase)) await fail("reset_origin_server_closed_before_end");
  };
  server.once("close", () => clearTimeout(captureTimer));
  return server;
}
