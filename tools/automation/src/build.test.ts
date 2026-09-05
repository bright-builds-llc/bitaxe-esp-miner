import assert from "node:assert/strict";
import test from "node:test";

import { rejectUnknownKconfigWarnings, requireResolvedUsbMemoryContract } from "./build.js";

const resolved = [
  "CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304",
  "CONFIG_SPIRAM_TRY_ALLOCATE_WIFI_LWIP=y",
  "CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM=6",
  "CONFIG_ESP_WIFI_STATIC_TX_BUFFER_NUM=6",
  "CONFIG_ESP_WIFI_TX_BUFFER_TYPE=0",
  "CONFIG_ESP_WIFI_DYNAMIC_RX_BUFFER_NUM=32",
  "CONFIG_ESP_WIFI_AMPDU_RX_ENABLED=y",
  "CONFIG_ESP_WIFI_RX_BA_WIN=12",
  "",
].join("\n");

test("firmware build rejects unknown Kconfig symbols", () => {
  // Arrange / Act / Assert
  assert.throws(
    () => rejectUnknownKconfigWarnings("warning: unknown kconfig symbol 'BOGUS' assigned to '1'"),
    /unknown Kconfig/u,
  );
  assert.doesNotThrow(() => rejectUnknownKconfigWarnings("warning: ordinary compiler warning"));
});

test("resolved coexistence profile preserves static DMA TX and the internal reserve", () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireResolvedUsbMemoryContract(resolved));
  for (const [before, after] of [["98304", "65536"], ["TX_BUFFER_TYPE=0", "TX_BUFFER_TYPE=1"]]) {
    assert.throws(() => requireResolvedUsbMemoryContract(resolved.replace(before!, after!)), /USB memory contract/u);
  }
});

test("packaging rejects stale large Wi-Fi pools instead of trusting requested defaults", () => {
  // Arrange
  const stale = resolved.replace("STATIC_RX_BUFFER_NUM=6", "STATIC_RX_BUFFER_NUM=16")
    .replace("STATIC_TX_BUFFER_NUM=6", "STATIC_TX_BUFFER_NUM=16");
  // Act / Assert
  assert.throws(() => requireResolvedUsbMemoryContract(stale), /USB memory contract/u);
});

test("missing or duplicate resolved buffer fields fail closed", () => {
  // Arrange
  const field = "CONFIG_ESP_WIFI_RX_BA_WIN=12\n";
  // Act / Assert
  for (const candidate of [resolved.replace(field, ""), resolved + field]) {
    assert.throws(() => requireResolvedUsbMemoryContract(candidate), /USB memory contract/u);
  }
});
