import assert from "node:assert/strict";
import test from "node:test";
import { deviceClient } from "./device.mjs";

const ENDPOINT = { ipv4: "192.168.1.20", httpPort: 80 };

test("a request that runs out of time reports soak_device_timeout", async () => {
  // Arrange
  const client = deviceClient(ENDPOINT, async () => { throw new DOMException("timed out", "TimeoutError"); });
  // Act
  const error = await client.info().then(() => null, (failure) => failure);
  // Assert
  assert.equal(error.code, "soak_device_timeout");
});

test("a device that cannot be reached reports soak_device_unreachable", async () => {
  // Arrange
  const client = deviceClient(ENDPOINT, async () => { throw new TypeError("fetch failed"); });
  // Act
  const error = await client.command("pause").then(() => null, (failure) => failure);
  // Assert
  assert.equal(error.code, "soak_device_unreachable");
});

test("a response that is not ok keeps its own category", async () => {
  // Arrange
  const client = deviceClient(ENDPOINT, async () => ({ ok: false }));
  // Act
  const error = await client.info().then(() => null, (failure) => failure);
  // Assert
  assert.equal(error.code, "soak_device_info");
});
