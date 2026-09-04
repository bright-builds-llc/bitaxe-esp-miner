import assert from "node:assert/strict";
import test from "node:test";

import {
  rejectUnknownKconfigWarnings,
  requireResolvedUsbMemoryContract,
} from "./build.js";

test("firmware build rejects unknown Kconfig symbols", () => {
  assert.throws(
    () => rejectUnknownKconfigWarnings("warning: unknown kconfig symbol 'BOGUS' assigned to '1'"),
    /unknown Kconfig/u,
  );
  assert.doesNotThrow(() => rejectUnknownKconfigWarnings("warning: ordinary compiler warning"));
});

test("firmware build requires the resolved USB and internal-memory budgets", () => {
  const resolved = [
    "CONFIG_TINYUSB_TASK_STACK_SIZE=3072",
    "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=65536",
    "",
  ].join("\n");
  assert.doesNotThrow(() =>
    requireResolvedUsbMemoryContract(resolved),
  );
  assert.throws(
    () => requireResolvedUsbMemoryContract(resolved.replace("3072", "4096")),
    /USB memory contract/u,
  );
  assert.throws(
    () => requireResolvedUsbMemoryContract(resolved.replace("65536", "32768")),
    /USB memory contract/u,
  );
  assert.throws(
    () => requireResolvedUsbMemoryContract("CONFIG_TINYUSB_TASK_STACK_SIZE=3072\n"),
    /USB memory contract/u,
  );
});
