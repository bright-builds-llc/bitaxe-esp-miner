/** Failure evidence has its own one-shot lifetime, independent of qualification success. */
export function createRecoveryCollector({ gate, save, published, campaignId, attemptId, timeoutMs = 30000 }) {
  let used = false;
  async function bounded(operation, limit = timeoutMs) {
    let timer;
    try { return await Promise.race([Promise.resolve().then(operation), new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error("timeout")), limit);
    })]); } finally { clearTimeout(timer); }
  }
  return async () => {
    if (used) throw Error("recovery_consumed");
    used = true;
    const failures = [];
    async function collect(stage, operation, limit) {
      try { await bounded(async () => save(stage, await operation()), limit); }
      catch { failures.push(stage); }
    }
    try {
      // Each read reuses Gate's authenticated controller; no signed grant is requested.
      await collect("ledger", () => gate.reviewQualificationAttempts());
      await collect("original_budget", () => gate.reviewBudget(campaignId));
      await collect("diagnostics", async () => { await gate.exportDiagnostics(); return { exported: true }; });
    } finally {
      // A missing retained record, disk failure, or test failure cannot bypass restoration/release.
      await collect("state", async () => { await gate.stop(); await gate.refresh(); return published(); }, 150000);
      await collect("status", async () => {
        const binding = await gate.stratumV2Possession();
        return gate.stratumV2Status("share", attemptId, binding);
      });
      await collect("closed", async () => { await gate.close(); return published(); }, 150000);
    }
    await bounded(() => save("finished", { failures }));
    return { complete: failures.length === 0, failures };
  };
}

export function installRecoveryControls(document, collect) {
  const button = document.createElement("button"), output = document.createElement("pre");
  button.id = "recover"; button.textContent = "Collect failure recovery and release";
  output.id = "recovery-result"; output.setAttribute("role", "status");
  button.addEventListener("click", async () => {
    if (button.disabled) return;
    button.disabled = true; output.textContent = "Collecting recovery evidence";
    const maybeConnect = document.getElementById?.("connect");
    if (maybeConnect) maybeConnect.disabled = true;
    try { output.textContent = JSON.stringify(await collect()); }
    catch { output.textContent = "Recovery evidence incomplete; retain this page and artifacts."; }
  });
  document.body.append(button, output);
}

if (typeof window !== "undefined" && typeof document !== "undefined") {
  const request = async (path, value) => {
    const response = await fetch(path, { method: "POST", headers: { "Content-Type": "application/json" },
      cache: "no-store", redirect: "error", body: JSON.stringify(value) });
    if (!response.ok) throw Error("recovery_record_failed");
    return response.json();
  };
  const gate = window.workerAcceptance;
  // Capture the narrow capability set before removing all effectful public page operations.
  const allowed = Object.fromEntries(["reviewQualificationAttempts", "reviewBudget", "exportDiagnostics",
    "stratumV2Possession", "stratumV2Status", "stop", "refresh", "close"].map(key => [key, gate[key].bind(gate)]));
  for (const key of ["prepareStartAuthorization", "loadWindow", "loadSignedWindow", "startWindow", "stratumV2ChannelStart",
    "suppressHeartbeats", "armForegroundLoss", "submitCoolingReview", "proveCoolingForQualification", "restoreCoolingBaseline"])
    delete gate[key];
  for (const id of ["prepare", "load", "start", "arm-foreground", "suppress", "authorization-context", "probe", "configuration"]) document.getElementById(id)?.remove();
  const context = await request("/recovery-context", {});
  installRecoveryControls(document, createRecoveryCollector({ gate: allowed, campaignId: context.campaignId, attemptId: context.attemptId,
    published: () => gate.state(), save: (stage, value) => stage === "diagnostics" ? Promise.resolve() : request("/part", { stage, value }) }));
}
