import { createWebUsbWorkerControllerV03 } from "./gate.js";
import { requiresPhysicalReacquisition } from "./browser-contract.mjs";

const token = location.pathname.split("/").filter(Boolean)[0];
const endpoint = (name) => `/${token}/${name}`;
const config = await fetch(endpoint("config"), { cache: "no-store" }).then((response) => {
  if (!response.ok) throw new Error("configuration unavailable");
  return response.json();
});
const status = document.querySelector("#status");
const scenario = document.querySelector("#scenario");
const connect = document.querySelector("#connect");
const run = document.querySelector("#run");
const reacquire = document.querySelector("#reacquire");
scenario.textContent = `Scenario: ${config.scenario}`;
status.textContent = "Ready for an explicit WebUSB permission gesture.";
let controller;
let active = false;
let admitted = false;
let leaseStartedAt;

async function post(name, value = {}) {
  const response = await fetch(endpoint(name), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(value),
  });
  if (!response.ok) throw new Error(`${name} rejected`);
  return response.json();
}

async function retainedDeviceIdentityFingerprint() {
  const challenge = config.controller.continuityScope.challengeId;
  const bytes = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(`bwg-worker-continuity/0.1:${challenge}`),
  );
  const binding = btoa(String.fromCharCode(...new Uint8Array(bytes)))
    .replaceAll("+", "-").replaceAll("/", "_").replaceAll("=", "");
  const database = await new Promise((resolve, reject) => {
    const request = indexedDB.open("bwg-worker", 2);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(new Error("continuity unavailable"));
  });
  try {
    const record = await new Promise((resolve, reject) => {
      const request = database.transaction("continuity", "readonly")
        .objectStore("continuity").get(binding);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(new Error("continuity unavailable"));
    });
    if (
      typeof record !== "object" || record === null ||
      record.challengeBindingSha256 !== binding ||
      !/^[A-Za-z0-9_-]{43}$/.test(record.deviceIdentityFingerprint)
    ) throw new Error("continuity invalid");
    return record.deviceIdentityFingerprint;
  } finally {
    database.close();
  }
}

function safeStatus(value) {
  return {
    state: value.state,
    restorationStatus: value.restoration?.status,
    restorationReason: value.restoration?.reason,
  };
}

async function authorization(operation) {
  const context = await controller.prepareWorkerLeaseAuthorizationContext(operation);
  return post("authorize", { operation, ...context });
}

async function expectRejected(event, action) {
  try {
    await action();
  } catch {
    await post("event", { event });
    return;
  }
  throw new Error(`${event} was accepted`);
}

async function startLease() {
  const artifact = await authorization("start");
  const value = await controller.startLease(artifact.request);
  active = true;
  leaseStartedAt = performance.now();
  await post("event", { event: "lease_started", ...safeStatus(value) });
  return value;
}

async function finish(value) {
  const terminal = safeStatus(value);
  await post("event", { event: "baseline_confirmed", ...terminal });
  await controller.close("tab_closed");
  active = false;
  admitted = false;
  await post("complete", { outcome: "complete", cleanup: "confirmed", ...terminal });
  status.textContent = `Complete: ${terminal.restorationReason ?? "baseline"}`;
  run.disabled = true;
  reacquire.disabled = true;
}

async function failClosed(category) {
  let cleanupCategory;
  if (controller && (active || admitted)) {
    try {
      await controller.close("control_failed");
    } catch {
      cleanupCategory = "cleanup_failed";
    }
    active = false;
    admitted = false;
  }
  await post("failed", { category, ...(cleanupCategory ? { cleanupCategory } : {}) });
}

connect.addEventListener("click", async () => {
  connect.disabled = true;
  try {
    controller = createWebUsbWorkerControllerV03(config.controller);
    navigator.usb.addEventListener("disconnect", async () => {
      if (!active || leaseStartedAt === undefined) {
        await failClosed("scenario_failed");
        return;
      }
      await post("event", { event: "transport_disconnected", reason: "connectivity_lost" });
      reacquire.disabled = false;
      status.textContent = config.physicalInstruction;
    });
    controller.subscribeDisconnect(async (reason) => {
      await post("event", { event: "disconnect_handled", reason });
    });
    const connection = await controller.requestPermission();
    admitted = true;
    await post("identity", {
      deviceIdentityFingerprint: await retainedDeviceIdentityFingerprint(),
    });
    const admittedDevices = (await navigator.usb.getDevices()).filter((device) =>
      device.vendorId === config.controller.deviceFilter.vendorId &&
      device.productId === config.controller.deviceFilter.productId
    );
    if (admittedDevices.length !== 1) throw new Error("device_count_invalid");
    await post("event", { event: "worker_admitted", mode: connection.mode, count: "1" });
    const capability = await controller.discover();
    if (
      capability.board.model !== "bitaxe-ultra" || capability.board.revision !== "205" ||
      capability.firmware.version !== "0.1.0" ||
      capability.transportProfile !== "bwg-worker-usb/0.2"
    ) {
      throw new Error("runtime_capability_invalid");
    }
    await post("event", { event: "capability_admitted" });
    status.textContent = "Exact Worker admitted. Run the bounded scenario.";
    run.disabled = false;
  } catch {
    status.textContent = "Admission failed closed.";
    await failClosed("admission_failed");
  }
});

run.addEventListener("click", async () => {
  run.disabled = true;
  try {
    await startLease();
    if (config.scenario === "authorization_negatives") {
      await controller.pause();
      const stale = await authorization("start");
      await new Promise((resolve) => setTimeout(resolve, config.contextExpiryMilliseconds));
      await expectRejected(
        "authorization_expired_rejected",
        () => controller.startLease(stale.request),
      );
      await controller.prepareWorkerLeaseAuthorizationContext("start");
      await expectRejected(
        "cross_context_rejected",
        () => controller.startLease(stale.request),
      );
      const fresh = await authorization("start");
      await controller.startLease(fresh.request);
      const renewal = await authorization("renew");
      await controller.renewLease(renewal.request);
      await expectRejected(
        "replay_rejected",
        () => controller.renewLease(renewal.request),
      );
      return finish(await controller.restore("control_failed"));
    }
    if (config.scenario !== "expiry") {
      await new Promise((resolve) => setTimeout(resolve, config.minimumActiveMilliseconds));
    }
    if (config.scenario === "pause") return finish(await controller.pause());
    if (config.scenario === "cancel") return finish(await controller.cancel());
    if (config.scenario === "completion") {
      const renewal = await authorization("renew");
      await controller.renewLease(renewal.request);
      return finish(await controller.restore("challenge_satisfied"));
    }
    if (config.scenario === "expiry") {
      for (;;) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        const value = await controller.status();
        if (value.state === "baseline") return finish(value);
      }
    }
    status.textContent = config.physicalInstruction;
    reacquire.disabled = !requiresPhysicalReacquisition(config.scenario);
    await post("event", { event: "physical_checkpoint_required", scenario: config.scenario });
  } catch {
    status.textContent = "Scenario failed closed.";
    await failClosed("scenario_failed");
  }
});

reacquire.addEventListener("click", async () => {
  reacquire.disabled = true;
  try {
    const restored = await controller.reacquire();
    await finish(restored);
  } catch {
    status.textContent = "Reacquisition remains pending.";
    reacquire.disabled = false;
    await post("event", { event: "reacquisition_retryable", category: "reacquisition_failed" });
  }
});

addEventListener("beforeunload", () => {
  if (active) {
    navigator.sendBeacon(
      endpoint("failed"),
      new Blob([JSON.stringify({ category: "browser_closed_active" })], {
        type: "application/json",
      }),
    );
  }
});
