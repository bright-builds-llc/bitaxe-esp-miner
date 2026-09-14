import { createServer } from "node:http";
import { readFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { isDeepStrictEqual as equal } from "node:util";
import {
  BUNDLE,
  digest,
  exactObject,
  fileDigest,
  nonce,
  QualificationError,
  readJson,
  requireCondition as check,
  writeNew,
} from "./contract.mjs";
import { body, send } from "./http.mjs";
import { baseline, proof } from "./cadence-premining-evidence.mjs";
import {
  validateRestartState as validateState,
  saveRestartAccounting as saveNoMiningAccounting,
  readRestartAccounting,
} from "./reset-origin-restart-state.mjs";
import { readRestartStates as readNoMiningStates } from "./reset-origin-restart-state.mjs";
import { readFrozenCampaign } from "./no-mining-accounting.mjs";
import { selectResetOriginDiagnostics } from "./reset-origin-server.mjs";
import { inspectResetOriginObservation } from "./reset-origin-observation-review.mjs";
import { loadRestartContext, rejectRestartCredentials, restartInnerContext } from "./reset-origin-restart-context.mjs";
import { requireRestartInstallation } from "./reset-origin-restart-install.mjs";
import { saveRestartFailure } from "./reset-origin-restart-failure.mjs";
import { validateRestartEvidence, validateFailedRestartEvidence } from "./reset-origin-restart-evidence.mjs";

/** One private phase owner keeps claims, metadata ordering and both firmware scopes in the same closure. */
export async function createRestartSupervisor(options, operations = {}) {
  rejectRestartCredentials(options);
  const root = resolve(options.privateRoot),
    context = await loadRestartContext(root, { operations });
  const hash = digest(JSON.stringify(context)),
    now = operations.now ?? (() => Math.floor(performance.now()));
  await writeNew(resolve(root, "restart-server-claim.json"), { schema: "fixed-usb-restart-server-claim-v1", context_sha256: hash });
  const trust = await readJson(resolve(context.firmware_root, "firmware/bitaxe/bwg/deployment-trust.json"));
  let scope = "before-install",
    phase = "before-install",
    terminalFailure = false,
    queue = Promise.resolve(),
    sequence = 0,
    prime,
    primedAt,
    startedAt,
    timer;
  const inner = () => restartInnerContext(root, context, scope),
    directory = () => resolve(root, scope);
  const assertActive = () => check(!terminalFailure, "restart_failed");
  const transition = (next) => {
    assertActive();
    phase = next;
  };
  async function guarded(operation) {
    assertActive();
    const result = await operation();
    assertActive();
    return result;
  }
  const verify = async () => check(equal(context, await guarded(() => loadRestartContext(root, { operations }))), "restart_frozen_changed");
  async function current(closed = false) {
    const rows = await guarded(() => readNoMiningStates(directory(), inner()));
    baseline(rows.at(-1).state, closed);
    return rows.at(-1);
  }
  async function fail(code) {
    if (terminalFailure) return;
    terminalFailure = true;
    phase = "failed";
    clearTimeout(timer);
    await saveRestartFailure(root, context, code);
  }
  async function record(input) {
    exactObject(input, ["state"]);
    const state = validateState(input.state, inner());
    check(
      (state.status !== "restarting" || phase === "restarting") &&
        (state.restart === undefined || ["restarting", "restarted", "finished", "failed"].includes(phase)),
      "restart_unclaimed_state",
    );
    if (state.restart !== undefined) {
      const claim = (await proof(resolve(root, "restart-consumed.json"))).value;
      check(claim.context_sha256 === hash, "restart_unclaimed_state");
    }
    check(!state.running && state.renewalsConfirmed === 0 && state.status !== "window_loaded", "restart_work_forbidden");
    const count = (await readdir(directory())).filter((name) => /^no-mining-state-[0-9]{4}\.json$/u.test(name)).length;
    check(count < 512, "restart_state_bound");
    await writeNew(resolve(directory(), `no-mining-state-${String(count + 1).padStart(4, "0")}.json`), {
      schema: "fixed-usb-restart-state-v1",
      context_sha256: digest(JSON.stringify(inner())),
      sequence: count + 1,
      state,
    });
    return { recorded: true, sequence: count + 1 };
  }
  async function mutate(path, input) {
    if (path === "/restart/failed-observation") {
      const claim = (await proof(resolve(root, "restart-consumed.json"))).value;
      const evidence = validateFailedRestartEvidence(input, context, claim.expected_boot_ordinal);
      await writeNew(resolve(root, "restart-failed-observation.json"), {
        schema: "fixed-usb-restart-failed-observation-v1",
        context_sha256: hash,
        evidence,
      });
      await fail("restart_client_failed");
      return { failed_evidence_saved: true, qualified: false };
    }
    if (path === "/record") return record(input);
    if (path === "/restart/failure") {
      exactObject(input, ["code"]);
      check(input.code === "restart_client_failed", "restart_failure_shape");
      await fail(input.code);
      return { failure_recorded: true };
    }
    assertActive();
    if (["/activate", "/original-budget-context", "/accounting", "/diagnostic-export"].includes(path)) return metadata(path, input);
    if (path === "/restart/result") return receiveRestart(path, input);
    exactObject(input, []);
    if (["/restart/installed", "/restart/observation-start", "/restart/observation-end"].includes(path)) return observationAction(path);
    return restartAction(path);
  }
  async function metadata(path, input) {
    if (path === "/activate") {
      exactObject(input, []);
      await verify();
      return { challengeId: `challenge_${nonce()}`, retentionExpiryUnixSeconds: Math.floor(Date.now() / 1000) + 86400 };
    }
    if (path === "/original-budget-context") {
      exactObject(input, []);
      await verify();
      return { campaignId: await readFrozenCampaign(inner()) };
    }
    if (path === "/accounting") {
      check(
        (scope === "before-install" && input.stage === "before") ||
          (scope === "after-install" &&
            ((phase === "after-install" && input.stage === "before") || (phase === "restarted" && input.stage === "after"))),
        "restart_accounting_phase",
      );
      await verify();
      if (scope === "after-install") {
        const prior = await readRestartAccounting(
          resolve(root, "before-install"),
          restartInnerContext(root, context, "before-install"),
          "before",
        );
        check(
          equal(input.ledger, prior.ledger) &&
            equal(input.original_budget, prior.original_budget) &&
            input.state?.preservation?.baseline_id === prior.state.preservation.baseline_id,
          "restart_install_preservation_changed",
        );
      }
      return saveNoMiningAccounting(directory(), inner(), input);
    }
    if (path === "/diagnostic-export") {
      check(scope === "after-install" && ["after-install", "observing"].includes(phase), "restart_capture_phase");
      check(sequence <= 1024, "restart_batch_bound");
      const observations = selectResetOriginDiagnostics(input),
        at = now();
      if (phase === "after-install") {
        const installed = await requireRestartInstallation(root, context),
          boots = observations.filter((d) => d.category === "boot");
        check(
          boots.length === 1 &&
            boots[0].boot_ordinal === installed.startup.boot_ordinal &&
            boots[0].reset_reason === installed.startup.initial_reset_category,
          "restart_install_boot_changed",
        );
      }
      check(phase !== "observing" || at - startedAt <= 135000, "restart_capture_bound");
      await writeNew(resolve(directory(), `diagnostic-export-${String(sequence).padStart(4, "0")}.json`), {
        schema: "fixed-usb-reset-origin-batch-v1",
        context_sha256: hash,
        sequence,
        hostMonotonicMs: at,
        observations,
      });
      const file = `diagnostic-export-${String(sequence++).padStart(4, "0")}.json`;
      if (phase === "after-install") {
        prime = observations;
        primedAt = at;
        transition("primed");
      }
      return { diagnostic_export_saved: true, review_file: file };
    }
  }
  async function receiveRestart(path, input) {
    if (path === "/restart/result") {
      check(phase === "restarting", "restart_result_phase");
      const claim = (await proof(resolve(root, "restart-consumed.json"))).value;
      check(now() - claim.hostMonotonicMs <= 30000, "restart_host_timeout");
      const evidence = validateRestartEvidence(input, context, claim.expected_boot_ordinal),
        after = await current();
      check(
        after.sequence > claim.before_sequence &&
          after.state.preservation.baseline_id === claim.baseline_id &&
          equal(after.state.restart, evidence.summary),
        "restart_fresh_admission_missing",
      );
      await writeNew(resolve(root, "restart-observation.json"), {
        schema: "fixed-usb-restart-observation-v1",
        context_sha256: hash,
        claim_sha256: await fileDigest(resolve(root, "restart-consumed.json")),
        after_sequence: after.sequence,
        hostMonotonicMs: now(),
        evidence,
      });
      clearTimeout(timer);
      transition("restarted");
      return { restart_observed: true, independent_review_required: true };
    }
  }
  async function observationAction(path) {
    if (path === "/restart/installed") {
      check(phase === "before-install", "restart_install_phase");
      await current(true);
      await verify();
      await requireRestartInstallation(root, context);
      await writeNew(resolve(root, "install-phase-advanced.json"), { schema: "fixed-usb-restart-phase-v1", context_sha256: hash });
      transition("after-install");
      scope = "after-install";
      return { configure_phase: scope };
    }
    if (path === "/restart/observation-start") {
      check(phase === "primed" && now() - primedAt <= 6000, "restart_prime_required");
      const before = await readJson(resolve(directory(), "no-mining-accounting-before.json")),
        row = await current();
      check(
        row.sequence === before.observed_sequence &&
          prime.some((d) => d.category === "boot") &&
          prime.some(
            (d) =>
              d.category === "runtime_identity" &&
              d.firmware_commit === context.firmware_commit &&
              d.app_elf_sha256 === context.app_elf_sha256,
          ) &&
          prime.some(
            (d) => d.category === "startup" && d.stage === "runtime_ready" && d.state === "complete" && d.first_failure === "none",
          ) &&
          prime.some((d) => d.category === "storage_http_status" && d.http_ready === "true" && d.spiffs_available === "true"),
        "restart_prime_incomplete",
      );
      startedAt = now();
      await writeNew(resolve(directory(), "reset-origin-start.json"), {
        schema: "fixed-usb-reset-origin-start-v1",
        context_sha256: hash,
        hostMonotonicMs: startedAt,
        observed_sequence: row.sequence,
        primeObservations: prime,
      });
      transition("observing");
      timer = setTimeout(() => {
        void fail("restart_preobservation_timeout").catch(() => server.emit("error", Error("restart_failure_persistence")));
      }, 135000);
      return { observation_started: true };
    }
    if (path === "/restart/observation-end") {
      check(phase === "observing" && now() - startedAt >= 120000 && now() - startedAt <= 135000, "restart_capture_bound");
      clearTimeout(timer);
      const at = now(),
        row = await current();
      const end = { schema: "fixed-usb-reset-origin-end-v1", context_sha256: hash, hostMonotonicMs: at, observed_sequence: row.sequence };
      await writeNew(resolve(directory(), "reset-origin-end.json"), end);
      const observed = await inspectResetOriginObservation(
        directory(),
        context,
        await readJson(resolve(directory(), "reset-origin-start.json")),
        end,
      );
      await verify();
      await writeNew(resolve(root, "pre-restart-observation.json"), {
        schema: "fixed-usb-pre-restart-observation-v1",
        context_sha256: hash,
        observed,
      });
      transition("observed");
      return { observation_reviewed: true };
    }
  }
  async function restartAction(path) {
    if (path === "/restart/consume") {
      assertActive();
      check(phase === "observed", "restart_not_ready");
      await verify();
      const row = await current();
      const end = await guarded(() => readJson(resolve(directory(), "reset-origin-end.json")));
      check(row.sequence > end.observed_sequence, "restart_fresh_ready_required");
      const reviewed = await guarded(() => readJson(resolve(root, "pre-restart-observation.json")));
      const start = await guarded(() => readJson(resolve(directory(), "reset-origin-start.json")));
      const observed = await guarded(() => inspectResetOriginObservation(directory(), context, start, end));
      check(
        equal(reviewed, { schema: "fixed-usb-pre-restart-observation-v1", context_sha256: hash, observed }),
        "restart_preobservation_changed",
      );
      const accounting = await guarded(() => readRestartAccounting(directory(), inner(), "before"));
      check(row.state.preservation.baseline_id === accounting.state.preservation.baseline_id, "restart_baseline_changed");
      const ordinal = reviewed.observed.summary.initialBootOrdinal;
      check(Number.isSafeInteger(ordinal) && ordinal > 0 && ordinal < Number.MAX_SAFE_INTEGER, "restart_boot_ordinal");
      // A failure during this exclusive write consumes the attempt, but must never release a permit.
      await guarded(() =>
        writeNew(resolve(root, "restart-consumed.json"), {
          schema: "fixed-usb-restart-consumed-v1",
          context_sha256: hash,
          request_nonce_sha256: digest(context.request_nonce),
          expected_boot_ordinal: ordinal,
          before_sequence: row.sequence,
          baseline_id: row.state.preservation.baseline_id,
          hostMonotonicMs: now(),
        }),
      );
      transition("restarting");
      timer = setTimeout(() => {
        void fail("restart_transition_timeout").catch(() => server.emit("error", Error("restart_failure_persistence")));
      }, 30000);
      assertActive();
      return { requestNonce: context.request_nonce, expectedBootOrdinal: ordinal };
    }
    check(path === "/restart/finish" && phase === "restarted", "restart_finish_phase");
    const row = await current(true),
      after = await readJson(resolve(directory(), "no-mining-accounting-after.json"));
    check(row.sequence > after.observed_sequence, "restart_final_journal_order");
    await writeNew(resolve(root, "restart-finished.json"), {
      schema: "fixed-usb-restart-finished-v1",
      context_sha256: hash,
      final_sequence: row.sequence,
    });
    transition("finished");
    return { captured: true, independent_review_required: true };
  }
  const server = createServer((request, response) => {
    handle(request, response).catch(async (error) => {
      const code = error instanceof QualificationError ? error.code : "restart_local_operation_failed";
      try {
        await fail(code);
      } catch {
        response.destroy();
        return;
      }
      if (response.headersSent) response.destroy();
      else send(response, 400, { error: code });
    });
  });
  async function handle(request, response) {
    const origin = `http://127.0.0.1:${server.address().port}`;
    check(request.headers.host === `127.0.0.1:${server.address().port}`, "host_rejected");
    if (request.method === "POST")
      check(
        request.headers.origin === origin || (request.headers.origin === undefined && request.headers["sec-fetch-site"] === "same-origin"),
        "origin_rejected",
      );
    const path = new URL(request.url, origin).pathname;
    if (request.method === "GET" && path === "/context") {
      await verify();
      const c = inner();
      return send(response, 200, {
        expectedGateCommit: c.gate_commit,
        expectedFirmwareSourceCommit: c.firmware_commit,
        expectedAppElfSha256: c.app_elf_sha256,
        trust,
        restartQualification: true,
      });
    }
    if (request.method === "GET" && path === "/supervisor-state")
      return send(response, 200, { mode: "restart-qualification", phase, mining_authorized: false });
    if (request.method === "GET" && ["/no-mining-client.mjs", "/reset-origin-restart-client.mjs"].includes(path)) {
      const bytes = await readFile(resolve(root, "host-clients", path.slice(1)));
      check(
        digest(bytes) === (path === "/no-mining-client.mjs" ? context.driver.no_mining_client_sha256 : context.driver.client_sha256),
        "restart_client_changed",
      );
      return send(response, 200, bytes, "text/javascript");
    }
    if (request.method === "GET" && ["/", `/${context.gate_page_relative_path}`, `/${BUNDLE}`].includes(path)) {
      const isPage = path !== `/${BUNDLE}`;
      let bytes = await readFile(resolve(root, "qualified-artifacts/gate", isPage ? context.gate_page_relative_path : BUNDLE));
      check(digest(bytes) === (isPage ? context.gate_page_sha256 : context.gate_bundle_sha256), "restart_asset_changed");
      if (isPage)
        bytes = Buffer.from(
          `${bytes}\n<script type="module" src="/no-mining-client.mjs"></script>\n<script type="module" src="/reset-origin-restart-client.mjs"></script>`,
        );
      return send(response, 200, bytes, isPage ? "text/html" : "text/javascript");
    }
    const post = [
      "/record",
      "/activate",
      "/original-budget-context",
      "/accounting",
      "/diagnostic-export",
      "/restart/failure",
      "/restart/installed",
      "/restart/observation-start",
      "/restart/observation-end",
      "/restart/consume",
      "/restart/result",
      "/restart/finish",
      "/restart/failed-observation",
    ];
    if (request.method === "POST" && post.includes(path)) {
      const input = await (["/restart/result", "/restart/failed-observation"].includes(path) ? evidenceBody(request) : body(request)),
        pending = queue.then(() => mutate(path, input));
      queue = pending.then(
        () => undefined,
        () => undefined,
      );
      const result = await pending;
      if (!["/record", "/restart/failure", "/restart/failed-observation"].includes(path)) assertActive();
      return send(response, 200, result);
    }
    send(response, 404, { error: "route_unavailable" });
  }
  server.closeQualificationResources = async () => {
    await queue;
    clearTimeout(timer);
    if (!["finished", "failed"].includes(phase)) await fail("restart_server_closed_incomplete");
  };
  server.once("close", () => clearTimeout(timer));
  return server;
}

async function evidenceBody(request) {
  const chunks = [];
  let length = 0;
  for await (const chunk of request) {
    length += chunk.length;
    check(length <= 524288, "restart_evidence_body_bound");
    chunks.push(chunk);
  }
  try {
    return JSON.parse(Buffer.concat(chunks).toString("utf8"));
  } catch {
    throw new QualificationError("request_json");
  }
}
