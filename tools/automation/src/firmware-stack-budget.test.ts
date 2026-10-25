import assert from "node:assert/strict";
import test from "node:test";

import { verifyFirmwareStackBudget, verifyGamma601StackBudget } from "./firmware-stack-budget.js";

const operatorSymbol = "bitaxe_firmware::operator_sensor_runtime::run";
const readinessSymbol = "bitaxe_firmware::runtime_snapshot::collect_platform_readiness_snapshot";
const publishSymbol = "bitaxe_firmware::gamma601::observations::Producer::publish";
const screenSymbol = "bitaxe_firmware::runtime_snapshot::screen::collect_screen_snapshot";

function symbol(symbolName: string, frame: string): string {
  return `42000000 <${symbolName}>:\n42000000:\t000000\tentry\ta1, ${frame}\n42000003:\t000000\tnop\n`;
}

function disassembly(operatorFrame: string, readinessFrame: string, screenFrame: string): string {
  return `${symbol(operatorSymbol, operatorFrame)}\n${symbol(readinessSymbol, readinessFrame)}\n${symbol(screenSymbol, screenFrame)}`;
}

test("firmware stack audit accepts the bounded operator screen path", () => {
  // Arrange
  const input = disassembly("0x800", "0x1e0", "0x3c0");

  // Act
  const result = verifyFirmwareStackBudget(input);

  // Assert
  assert.deepEqual(result, {
    operatorSensorFrameBytes: 2_048,
    platformReadinessFrameBytes: 480,
    screenCollectorFrameBytes: 960,
    combinedFrameBytes: 3_008,
  });
});

test("firmware stack audit rejects a missing required symbol", () => {
  // Arrange
  const input = `${symbol(operatorSymbol, "0x800")}\n${symbol(readinessSymbol, "0x1e0")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /exactly one/u);
});

test("firmware stack audit decodes an Xtensa entry grouped as a raw word", () => {
  // Arrange
  const rawOperator =
    `42000000 <${operatorSymbol}>:\n42000000:\t5211c136 \n42000004:\t000000\tnop\n`;
  const input = `${rawOperator}\n${symbol(readinessSymbol, "0x1e0")}\n` +
    symbol(screenSymbol, "0x3c0");

  // Act
  const budget = verifyFirmwareStackBudget(input);

  // Assert
  assert.equal(budget.operatorSensorFrameBytes, 0x8e0);
});

test("firmware stack audit rejects a raw word without the exact Xtensa entry opcode", () => {
  // Arrange
  const rawOperator =
    `42000000 <${operatorSymbol}>:\n42000000:\t5211c137 \n42000004:\t000000\tnop\n`;
  const input = `${rawOperator}\n${symbol(readinessSymbol, "0x1e0")}\n` +
    symbol(screenSymbol, "0x3c0");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /one entry frame/u);
});

test("firmware stack audit rejects a duplicated required symbol", () => {
  // Arrange
  const input = `${disassembly("0x800", "0x1e0", "0x3c0")}\n${symbol(operatorSymbol, "0x800")}`;

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /exactly one/u);
});

test("firmware stack audit rejects a malformed entry frame", () => {
  // Arrange
  const input = disassembly("invalid", "0x1e0", "0x3c0");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /one entry frame/u);
});

test("firmware stack audit rejects an oversized individual frame", () => {
  // Arrange
  const input = disassembly("0xc10", "0x1e0", "0x3c0");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /oversized individual frame/u);
});

test("firmware stack audit rejects an oversized combined path", () => {
  // Arrange
  const input = disassembly("0x800", "0x1e0", "0xa00");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /oversized operator screen path/u);
});

test("firmware stack audit rejects an oversized platform readiness frame", () => {
  // Arrange
  const input = disassembly("0x800", "0x410", "0x3c0");

  // Act / Assert
  assert.throws(() => verifyFirmwareStackBudget(input), /oversized platform readiness frame/u);
});

test("the 601 stack audit checks its safe-boot thread in place of the sensor owner", () => {
  // Arrange
  const input = `${symbol("bitaxe_firmware::gamma601::run", "0x200")}\n${symbol(publishSymbol, "0x3e0")}\n${symbol(readinessSymbol, "0x1e0")}`;

  // Act
  const result = verifyGamma601StackBudget(input);

  // Assert
  assert.deepEqual(result, { safeBootFrameBytes: 512, publishFrameBytes: 992, platformReadinessFrameBytes: 480 });
});

test("the 601 stack audit refuses an oversized safe-boot frame", () => {
  // Arrange
  const input = `${symbol("bitaxe_firmware::gamma601::run", "0x1000")}\n${symbol(publishSymbol, "0x3e0")}\n${symbol(readinessSymbol, "0x1e0")}`;

  // Act / Assert
  assert.throws(() => verifyGamma601StackBudget(input), /safe-boot frame/u);
});

test("the 601 stack audit refuses a publish path over the combined budget", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x900"),
    symbol(publishSymbol, "0x900"),
    symbol(readinessSymbol, "0x1e0"),
  ].join("\n");

  // Act / Assert
  assert.throws(() => verifyGamma601StackBudget(input), /observation publish path/u);
});

test("the core-voltage check variant also bounds its check's frame", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x200"),
    symbol(publishSymbol, "0x3e0"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::vout_check::run_check", "0x300"),
  ].join("\n");

  // Act
  const result = verifyGamma601StackBudget(input, { voutCheck: true });

  // Assert
  assert.equal(result.maybeVoutCheckFrameBytes, 768);
});

test("the core-voltage check variant refuses a check path over the combined budget", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x900"),
    symbol(publishSymbol, "0x100"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::vout_check::run_check", "0x900"),
  ].join("\n");

  // Act / Assert
  assert.throws(() => verifyGamma601StackBudget(input, { voutCheck: true }), /core-voltage check path/u);
});

test("the chip check variant also bounds its check's frame", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x200"),
    symbol(publishSymbol, "0x3e0"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::chip_check::run_check", "0x300"),
  ].join("\n");

  // Act
  const result = verifyGamma601StackBudget(input, { chipCheck: true });

  // Assert
  assert.equal(result.maybeChipCheckFrameBytes, 768);
});

test("the chip check variant refuses a check path over the combined budget", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x900"),
    symbol(publishSymbol, "0x100"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::chip_check::run_check", "0x900"),
  ].join("\n");

  // Act / Assert
  assert.throws(() => verifyGamma601StackBudget(input, { chipCheck: true }), /chip check path/u);
});

test("the hash check variant also bounds its check's frame", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x200"),
    symbol(publishSymbol, "0x3e0"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::hash_check::run_check", "0x300"),
  ].join("\n");

  // Act
  const result = verifyGamma601StackBudget(input, { hashCheck: true });

  // Assert
  assert.equal(result.maybeHashCheckFrameBytes, 768);
});

test("the hash check variant refuses a check path over the combined budget", () => {
  // Arrange
  const input = [
    symbol("bitaxe_firmware::gamma601::run", "0x900"),
    symbol(publishSymbol, "0x100"),
    symbol(readinessSymbol, "0x1e0"),
    symbol("bitaxe_firmware::gamma601::hash_check::run_check", "0x900"),
  ].join("\n");

  // Act / Assert
  assert.throws(() => verifyGamma601StackBudget(input, { hashCheck: true }), /hash check path/u);
});
