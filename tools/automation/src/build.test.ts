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

test("firmware build requires the resolved TinyUSB stack budget", () => {
  assert.doesNotThrow(() =>
    requireResolvedUsbMemoryContract("CONFIG_TINYUSB_TASK_STACK_SIZE=3072\n"),
  );
  assert.throws(
    () => requireResolvedUsbMemoryContract("CONFIG_TINYUSB_TASK_STACK_SIZE=4096\n"),
    /qualified memory budget/u,
  );
  assert.throws(
    () => requireResolvedUsbMemoryContract("CONFIG_TINYUSB_CDC_RX_BUFSIZE=512\n"),
    /qualified memory budget/u,
  );
});
