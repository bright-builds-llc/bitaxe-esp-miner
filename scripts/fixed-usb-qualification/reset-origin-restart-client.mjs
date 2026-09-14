const note = document.createElement("p");
note.id = "restart-supervisor-status";
note.textContent = "Configure this page for the installed firmware before recording the installation baseline.";
document.body.append(note);
let stage = "initial",
  busy = false,
  baselineId,
  cleanupFailed = false;
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
async function request(path, input, timeout = 60000) {
  const controller = new AbortController(),
    timer = setTimeout(() => controller.abort(), timeout);
  try {
    const response = await fetch(path, {
      method: input === undefined ? "GET" : "POST",
      headers: input === undefined ? {} : { "Content-Type": "application/json" },
      cache: "no-store",
      ...(input === undefined ? {} : { body: JSON.stringify(input) }),
      signal: controller.signal,
    });
    if (!response.ok) throw Error("restart_request_rejected");
    return await response.json();
  } finally {
    clearTimeout(timer);
  }
}
async function bounded(operation) {
  let timer;
  try {
    return await Promise.race([
      operation,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(Error("restart_operation_timeout")), 60000);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}
function baseline() {
  const state = window.workerAcceptance.state();
  if (
    state.status !== "ready" ||
    !state.connected ||
    state.running ||
    state.failure ||
    !state.deviceLeaseInactive ||
    state.deviceBaselineConfirmed !== true ||
    !state.preservation?.device_identity_match ||
    !state.preservation.settings_match ||
    !state.preservation.authorization_high_water_match ||
    state.preservation.mine_on_boot !== false ||
    (baselineId !== undefined && state.preservation.baseline_id !== baselineId)
  )
    throw Error("restart_preservation_failed");
  return state;
}
async function configureBeforeInstall() {
  if (stage !== "initial" || busy) throw Error("restart_client_phase");
  const state = await request("/supervisor-state"),
    configuration = await request("/context");
  if (state.phase !== "before-install" || configuration.restartQualification !== true) throw Error("restart_configuration_phase");
  await bounded(
    (async () => {
      while (true) {
        const observed = window.workerAcceptance.state();
        if (observed.failure) throw Error("restart_configuration_failed");
        if (
          ["configured", "ready"].includes(observed.status) &&
          observed.gateCommit === configuration.expectedGateCommit &&
          observed.expectedFirmwareSourceCommit === configuration.expectedFirmwareSourceCommit &&
          observed.expectedAppElfSha256 === configuration.expectedAppElfSha256
        )
          break;
        await delay(250);
      }
    })(),
  );
  stage = "before-install";
  note.textContent = "Connect the Worker, then record the baseline and release USB for the single installation.";
  return { configured: true };
}
async function prepareInstallation() {
  if (stage !== "before-install" || busy) throw Error("restart_client_phase");
  busy = true;
  try {
    baselineId = baseline().preservation.baseline_id;
    await bounded(window.noMiningSupervisor.recordAccounting("before"));
    await window.workerAcceptance.close();
    await bounded(window.noMiningSupervisor.flush());
    const state = window.workerAcceptance.state();
    if (state.status !== "closed" || state.connected || !state.serialOwnershipReleased) throw Error("restart_install_ownership");
    stage = "installation-released";
    note.textContent = "USB is released. Keep this page open during the admitted installation.";
    return { installation_baseline_recorded: true, serial_released: true };
  } catch (error) {
    stage = "failed";
    const failures = [error];
    try {
      await request("/restart/failure", { code: "restart_client_failed" });
    } catch (failure) {
      failures.push(failure);
    }
    try {
      await window.workerAcceptance.close();
      await bounded(window.noMiningSupervisor.flush());
    } catch (failure) {
      cleanupFailed = true;
      failures.push(failure);
    }
    throw new AggregateError(failures, "restart_installation_preparation_failed");
  } finally {
    busy = false;
  }
}
async function configureAfterInstall() {
  if (stage !== "installation-released" || busy) throw Error("restart_client_phase");
  busy = true;
  try {
    await bounded(window.noMiningSupervisor.flush());
    await request("/restart/installed", {});
    const state = await request("/supervisor-state"),
      configuration = await request("/context");
    if (state.phase !== "after-install" || configuration.restartQualification !== true) throw Error("restart_configuration_phase");
    window.workerAcceptance.configure(configuration);
    stage = "after-install";
    note.textContent = "Reconnect the new firmware on this same page, then observe and perform the single controlled restart.";
    return { configured: true };
  } finally {
    busy = false;
  }
}
async function exportDiagnostics() {
  const text = document.querySelector("#diagnostics")?.textContent;
  if (!text) throw Error("restart_diagnostics_missing");
  return request("/diagnostic-export", { schema: "worker-diagnostic-export-v1", observations: JSON.parse(text) }, 5000);
}
async function diagnosticsReady() {
  const began = performance.now();
  while (performance.now() - began < 15000) {
    baseline();
    const text = document.querySelector("#diagnostics")?.textContent;
    if (text) {
      const values = JSON.parse(text);
      if (
        ["boot", "runtime_identity", "storage_http_status"].every((category) => values.some((value) => value.category === category)) &&
        values.some((value) => value.category === "startup" && value.stage === "runtime_ready" && value.state === "complete")
      )
        return;
    }
    await delay(250);
  }
  throw Error("restart_diagnostics_missing");
}
async function observeAndRestart() {
  if (stage !== "after-install" || busy) throw Error("restart_client_phase");
  busy = true;
  stage = "observing";
  const failures = [];
  try {
    baseline();
    await diagnosticsReady();
    await bounded(window.noMiningSupervisor.recordAccounting("before"));
    await exportDiagnostics();
    await bounded(window.noMiningSupervisor.flush());
    await request("/restart/observation-start", {});
    const began = performance.now();
    let refreshed = began;
    while (performance.now() - began < 130000) {
      baseline();
      if (performance.now() - refreshed >= 1000) {
        await window.workerAcceptance.refresh();
        refreshed = performance.now();
        baseline();
      }
      await exportDiagnostics();
      await delay(250);
    }
    await bounded(window.noMiningSupervisor.flush());
    await request("/restart/observation-end", {});
    await window.workerAcceptance.refresh();
    baseline();
    await bounded(window.noMiningSupervisor.flush());
    const permit = await request("/restart/consume", {});
    stage = "restarting";
    const evidence = await window.workerAcceptance.qualificationRestart(permit);
    baseline();
    await bounded(window.noMiningSupervisor.flush());
    await request("/restart/result", evidence, 5000);
    await window.workerAcceptance.refresh();
    baseline();
    await bounded(window.noMiningSupervisor.flush());
    await bounded(window.noMiningSupervisor.recordAccounting("after"));
    await window.workerAcceptance.close();
    await bounded(window.noMiningSupervisor.flush());
    await request("/restart/finish", {});
    stage = "closed";
    note.textContent = "Capture complete and Worker closed. Independent verification remains required.";
    return { captured: true, independent_review_required: true };
  } catch (error) {
    failures.push(error);
    stage = "failed";
    try {
      const evidence = await window.workerAcceptance.exportRestartEvidence?.();
      if (evidence?.summary) await request("/restart/failed-observation", evidence);
    } catch (evidenceError) {
      failures.push(evidenceError);
    }
    try {
      await request("/restart/failure", { code: "restart_client_failed" });
    } catch (failureError) {
      failures.push(failureError);
    }
  } finally {
    if (stage !== "closed") {
      try {
        await window.workerAcceptance.close();
        await bounded(window.noMiningSupervisor.flush());
      } catch (closeError) {
        failures.push(closeError);
        cleanupFailed = true;
      }
    }
    busy = false;
  }
  throw new AggregateError(failures, "restart_qualification_failed");
}
Object.assign(window, {
  restartSupervisor: {
    configureBeforeInstall,
    prepareInstallation,
    configureAfterInstall,
    observeAndRestart,
    state: () => ({ stage, busy, cleanupFailed }),
  },
});
