import assert from "node:assert/strict";
import test from "node:test";
import { applyDeviceSettings, preflightProfile, profileSettings, SOAK_DEVICE_SETTINGS } from "./device-settings.mjs";

function fakeDevice(info, ok = true) {
  const calls = [];
  const fetchImpl = async (url, init = {}) => {
    calls.push({ url, method: init.method ?? "GET", body: init.body });
    return { ok, json: async () => info };
  };
  return { calls, fetchImpl };
}
const endpoint = { ipv4: "192.168.1.10", httpPort: 80 };

test("the soak patches its settings and records the matching read-back", async () => {
  // Arrange
  const device = fakeDevice({ frequency: 485, coreVoltage: 1200, autofanspeed: 1 });
  // Act
  const record = await applyDeviceSettings(endpoint, SOAK_DEVICE_SETTINGS, device.fetchImpl);
  // Assert
  assert.deepEqual([device.calls[0].method, JSON.parse(device.calls[0].body), record.observed.frequency],
    ["PATCH", { ...SOAK_DEVICE_SETTINGS }, 485]);
});

test("a read-back that differs from the contract refuses the soak", async () => {
  // Arrange
  const device = fakeDevice({ frequency: 400, coreVoltage: 1200, autofanspeed: 1 });
  // Act / Assert
  await assert.rejects(applyDeviceSettings(endpoint, SOAK_DEVICE_SETTINGS, device.fetchImpl), /soak_device_settings_readback/u);
});

test("a refused PATCH refuses the soak", async () => {
  // Arrange
  const device = fakeDevice({}, false);
  // Act / Assert
  await assert.rejects(applyDeviceSettings(endpoint, SOAK_DEVICE_SETTINGS, device.fetchImpl), /soak_device_settings_patch/u);
});

test("an endpoint that is not a private IPv4 address is refused before any request", async () => {
  // Arrange
  const device = fakeDevice({});
  // Act
  const named = applyDeviceSettings({ ipv4: "device.example", httpPort: 80 }, SOAK_DEVICE_SETTINGS, device.fetchImpl);
  const public_ = applyDeviceSettings({ ipv4: "8.8.8.8", httpPort: 80 }, SOAK_DEVICE_SETTINGS, device.fetchImpl);
  // Assert
  await assert.rejects(named, /soak_device_endpoint/u);
  await assert.rejects(public_, /soak_device_endpoint/u);
  assert.equal(device.calls.length, 0);
});

test("a named profile gives its settings", () => {
  // Arrange / Act / Assert
  assert.deepEqual(profileSettings("catalog-maximum-frequency"), { frequency: 575, coreVoltage: 1200, autofanspeed: 1 });
});

test("an unknown profile is refused", () => {
  // Arrange / Act / Assert
  assert.throws(() => profileSettings("catalog-maximum"), /soak_profile_unknown/u);
});

test("preflight defaults to the upstream-default profile", () => {
  // Arrange / Act / Assert
  assert.equal(preflightProfile(undefined, undefined), "upstream-default");
});

test("preflight refuses live changes from the catalog-maximum profile", () => {
  // Arrange / Act / Assert
  assert.throws(() => preflightProfile("catalog-maximum-frequency", "upstream-default"), /soak_live_changes_profile/u);
});

test("preflight refuses an unknown profile", () => {
  // Arrange / Act / Assert
  assert.throws(() => preflightProfile("overclock", undefined), /soak_profile_unknown/u);
});

test("a live plan supplies its own profile when none is named", () => {
  // Arrange / Act / Assert
  assert.equal(preflightProfile(undefined, "upstream-default-bap"), "upstream-default-bap");
});
