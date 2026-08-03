import assert from "node:assert/strict";
import test from "node:test";

import { uniqueRuntimeOrigin } from "./http.js";

test("runtime origin requires one origin-only candidate", () => {
  // Arrange
  const line = "runtime_origin session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa boot_ordinal=1 device_url=http://device.test redacted=true";

  // Act / Assert
  assert.equal(uniqueRuntimeOrigin(`${line}\n${line}\n`).origin, "http://device.test");
  assert.throws(() => uniqueRuntimeOrigin("runtime_origin session=x device_url=http://device.test/path redacted=true\n"));
});
