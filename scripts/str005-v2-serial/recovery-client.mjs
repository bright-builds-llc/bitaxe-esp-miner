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

/** Preserve Gate's required before-session baseline, close, then candidate reconnect. */
export function createRecoveryBootstrap({ gate, published, candidateConfiguration, collect }) {
  let consumed = false, prepared = false, maybeBaseline;
  const admitted = () => {
    const state = published(), preservation = state?.preservation;
    return state?.status === "ready" && state.connected === true && state.running === false &&
      state.serialOwnershipReleased === false && state.deviceBaselineConfirmed === true && state.deviceLeaseInactive === true &&
      state.expectedFirmwareSourceCommit === candidateConfiguration.expectedFirmwareSourceCommit &&
      state.expectedAppElfSha256 === candidateConfiguration.expectedAppElfSha256 &&
      state.gateCommit === candidateConfiguration.expectedGateCommit && preservation?.settings_match === true &&
      preservation.device_identity_match === true && preservation.authorization_high_water_match === true && preservation.mine_on_boot === false;
  };
  const canCollect = () => prepared && admitted() && published().preservation.baseline_id === maybeBaseline;
  return {
    canPrepare: () => !consumed && admitted(), canCollect,
    async prepare() {
      if (consumed || !admitted()) throw Error("recovery_before_baseline_required");
      consumed = true; maybeBaseline = published().preservation.baseline_id;
      let timer;
      try {
        await Promise.race([gate.close(), new Promise((_, reject) => { timer = setTimeout(() => reject(Error("recovery_close_timeout")), 150000); })]);
      } finally { clearTimeout(timer); }
      const closed = published();
      if (closed.status !== "closed" || closed.connected !== false || closed.running !== false || closed.serialOwnershipReleased !== true)
        throw Error("recovery_before_release_required");
      await gate.configure(candidateConfiguration);
      if (published().status !== "configured" || published().connected !== false) throw Error("recovery_candidate_configuration_failed");
      prepared = true;
      return { prepared: true, native_reconnect_required: true };
    },
    async collect() {
      if (!canCollect()) throw Error("recovery_candidate_reconnect_required");
      prepared = false;
      return collect();
    },
  };
}

export function installRecoveryControls(document, collect, maybeBootstrap) {
  const button = document.createElement("button"), output = document.createElement("pre");
  let used = false, preparing = false;
  button.id = "recover"; button.textContent = "Collect failure recovery and release";
  output.id = "recovery-result"; output.setAttribute("role", "status");
  const maybePrepare = maybeBootstrap ? document.createElement("button") : undefined;
  function update() {
    button.disabled = used || preparing || (maybeBootstrap ? !maybeBootstrap.canCollect() : false);
    if (maybePrepare) maybePrepare.disabled = preparing || !maybeBootstrap.canPrepare();
  }
  if (maybePrepare) {
    maybePrepare.id = "prepare-recovery"; maybePrepare.textContent = "Prepare recovery after baseline connection";
    maybePrepare.addEventListener("click", async () => {
      if (maybePrepare.disabled) return;
      preparing = true; update();
      try { await maybeBootstrap.prepare(); output.textContent = "Recovery configured. Use Connect Worker again, then collect."; }
      catch { output.textContent = "Recovery preparation failed. Retain evidence; Stop and Close remain available."; }
      finally { preparing = false; update(); }
    });
    document.body.append(maybePrepare);
  }
  button.addEventListener("click", async () => {
    if (button.disabled) return;
    used = true; update(); output.textContent = "Collecting recovery evidence";
    const maybeConnect = document.getElementById?.("connect");
    if (maybeConnect) maybeConnect.disabled = true;
    try { output.textContent = JSON.stringify(await collect()); }
    catch { output.textContent = "Recovery evidence incomplete; retain this page and artifacts."; }
  });
  document.body.append(button, output);
  update();
  return { update };
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
  const collect = createRecoveryCollector({ gate: allowed, campaignId: context.campaignId, attemptId: context.attemptId,
    published: () => gate.state(), save: (stage, value) => stage === "diagnostics" ? Promise.resolve() : request("/part", { stage, value }) });
  const bootstrap = createRecoveryBootstrap({ gate, published: () => gate.state(), candidateConfiguration: context.candidateConfiguration, collect });
  const controls = installRecoveryControls(document, () => bootstrap.collect(), bootstrap);
  const output = document.getElementById("state");
  if (output) new MutationObserver(controls.update).observe(output, { childList: true, subtree: true, characterData: true });
}
