import assert from "node:assert/strict";
import test from "node:test";

import { verifyFirmwareStackBudget } from "./firmware-stack-budget.js";

const operatorSymbol = "bitaxe_firmware::operator_sensor_runtime::run";
const screenSymbol = "bitaxe_firmware::runtime_snapshot::screen::collect_screen_snapshot";

function symbol(symbolName: string, frame: string): string {
  return `42000000 <${symbolName}>:\n42000000:\t000000\tentry\ta1, ${frame}\n42000003:\t000000\tnop\n`;
}

test("firmware stack audit accepts the bounded operator screen path", () => {
  // Arrange
  const disassembly = `${symbol(operatorSymbol, "0x800")}\n${symbol(screenSymbol, "0x3c0")}`;

  // Act
  const result = verifyFirmwareStackBudget(disassembly);

  // Assert
  assert.deepEqual(result, {
    operatorSensorFrameBytes: 2_048,
    screenCollectorFrameBytes: 960,
    combinedFrameBytes: 3_008,
  });
});

test("firmware stack audit rejects a missing required symbol", () => {
  // Arrange
  const disassembly = symbol(operatorSymbol, "0x800");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(disassembly), /exactly one/u);
});

test("firmware stack audit rejects a duplicated required symbol", () => {
  // Arrange
  const disassembly = `${symbol(operatorSymbol, "0x800")}\n${symbol(operatorSymbol, "0x800")}\n${symbol(screenSymbol, "0x3c0")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(disassembly), /exactly one/u);
});

test("firmware stack audit rejects a malformed entry frame", () => {
  // Arrange
  const disassembly = `${symbol(operatorSymbol, "invalid")}\n${symbol(screenSymbol, "0x3c0")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(disassembly), /one entry frame/u);
});

test("firmware stack audit rejects an oversized individual frame", () => {
  // Arrange
  const disassembly = `${symbol(operatorSymbol, "0xc10")}\n${symbol(screenSymbol, "0x3c0")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(disassembly), /oversized individual frame/u);
});

test("firmware stack audit rejects an oversized combined path", () => {
  // Arrange
  const disassembly = `${symbol(operatorSymbol, "0x800")}\n${symbol(screenSymbol, "0xa00")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(disassembly), /oversized operator screen path/u);
});
