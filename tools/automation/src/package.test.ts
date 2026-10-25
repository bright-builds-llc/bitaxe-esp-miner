import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import type { CommandSpec } from "./contracts.generated.js";
import { FirmwareBoardMarkerError, type FirmwareBoard } from "./firmware-board.js";
import {
  FirmwareVariantMarkerError,
  GAMMA601_CHIP_CHECK_MARKER,
  GAMMA601_HASH_CHECK_MARKER,
  GAMMA601_MINING_MARKER,
  GAMMA601_OVERHEAT_TRIP_TEST_MARKER,
  GAMMA601_VOUT_CHECK_MARKER,
  OVERHEAT_TRIP_TEST_MARKER,
  type FirmwareVariant,
} from "./firmware-variant.js";
import { packageFirmware } from "./package.js";
import { createFakeProcessPort, type ProcessOutcome } from "./process.js";
import { maybeTypedFailureCategory, maybeTypedFailurePublicValue } from "./typed-failure.js";

const buildLabel = "0123456789ab-dev";
const productionElf = Buffer.from("\x7fELF production firmware");
const testElf = Buffer.from(`\x7fELF ${OVERHEAT_TRIP_TEST_MARKER} local_overheat_trip_c=55`);
const gammaElf = Buffer.from("\x7fELF firmware_board=601 safe boot");
const gammaCheckElf = Buffer.from(`\x7fELF firmware_board=601 ${GAMMA601_VOUT_CHECK_MARKER}`);
const gammaMiningElf = Buffer.from(`\x7fELF firmware_board=601 ${GAMMA601_MINING_MARKER}`);
const gammaChipCheckElf = Buffer.from(`\x7fELF firmware_board=601 ${GAMMA601_CHIP_CHECK_MARKER}`);
const gammaHashCheckElf = Buffer.from(`\x7fELF firmware_board=601 ${GAMMA601_HASH_CHECK_MARKER}`);
const gammaOverheatElf = Buffer.from(
  `\x7fELF firmware_board=601 ${GAMMA601_OVERHEAT_TRIP_TEST_MARKER} local_overheat_trip_c=55`,
);

/** A workspace holding the package inputs and the managed ESP-IDF tool paths packaging resolves. */
async function packageWorkspace(elf: Buffer): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "bitaxe-package-"));
  const idf = path.join(root, ".embuild/espressif/esp-idf/v5.5.4/components");
  await mkdir(path.join(idf, "spiffs"), { recursive: true });
  await mkdir(path.join(idf, "esptool_py/esptool"), { recursive: true });
  await writeFile(path.join(idf, "spiffs/spiffsgen.py"), "");
  await writeFile(path.join(idf, "esptool_py/esptool/esptool.py"), "");
  await mkdir(path.join(root, "in/www"), { recursive: true });
  await writeFile(path.join(root, "in/www/index.html"), "<!doctype html>");
  await writeFile(path.join(root, "in/firmware.elf"), elf);
  await writeFile(path.join(root, "in/stamp.txt"), `build_label=${buildLabel}\n`);
  await writeFile(path.join(root, "in/sdkconfig"), `CONFIG_APP_PROJECT_VER="${buildLabel}"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n`);
  for (const name of ["bootloader.bin", "partition-table.bin", "otadata.bin"]) await writeFile(path.join(root, "in", name), "x");
  return root;
}

function argAfter(spec: CommandSpec<unknown>, flag: string): string {
  const value = spec.args[spec.args.indexOf(flag) + 1];
  if (value === undefined) throw new Error(`fake process missing ${flag}`);
  return value;
}

/** Plays every packaging tool: writes each declared output and records every command. */
function fakePackagingTools(runs: CommandSpec<unknown>[]) {
  return createFakeProcessPort(async (spec): Promise<ProcessOutcome> => {
    runs.push(spec);
    const ok = { exitCode: 0, stdout: "", stderr: "", timedOut: false };
    if (spec.program === "python3") {
      const [, , , , staging, image] = spec.args;
      if (staging === undefined || image === undefined) throw new Error("fake spiffsgen arguments");
      const label = await readFile(path.join(staging, "version.txt"), "utf8");
      await writeFile(image, label.padEnd(64, "\0"));
    } else if (spec.args.includes("image_info")) {
      return { ...ok, stdout: `App version: ${buildLabel}\nELF file SHA256: ${"ab".repeat(32)}\n` };
    } else if (spec.args.includes("-o")) {
      await writeFile(argAfter(spec, "-o"), "image");
    } else if (spec.args[0] === "package-firmware") {
      await writeFile(argAfter(spec, "--manifest"), JSON.stringify({ source_commit: "s", reference_commit: "r" }));
    }
    return ok;
  });
}

async function packageWith(
  root: string,
  firmwareVariant: FirmwareVariant,
  runs: CommandSpec<unknown>[],
  board: FirmwareBoard = "205",
): Promise<unknown> {
  const request = {
    firmwareElf: "in/firmware.elf", buildProvenanceStamp: "in/stamp.txt", espIdfSdkconfig: "in/sdkconfig",
    bootloaderBin: "in/bootloader.bin", partitionTableBin: "in/partition-table.bin", otadataInitialBin: "in/otadata.bin",
    outDir: "out", manifest: "out/manifest.json", webUi: { kind: "legacy-tree", path: "in/www" }, firmwareVariant, board,
  } as const;
  return packageFirmware(root, request, fakePackagingTools(runs), "xtask").then(() => undefined, (failure: unknown) => failure);
}

test("packaging records the selected firmware variant in the package manifest", async () => {
  // Arrange
  const root = await packageWorkspace(testElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "overheat-trip-test", runs);

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--firmware-variant"), "overheat-trip-test");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a production package whose ELF carries the test marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(testElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs);

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.equal(maybeTypedFailureCategory(error), "package_invalid");
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_in_production" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("an overheat-trip-test package whose ELF lacks the test marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(productionElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "overheat-trip-test", runs);

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.equal(maybeTypedFailureCategory(error), "package_invalid");
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_missing" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a production package whose ELF lacks the test marker is packaged as production", async () => {
  // Arrange
  const root = await packageWorkspace(productionElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs);

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--firmware-variant"), "production");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 package passes its board and file names to the manifest generator", async () => {
  // Arrange
  const root = await packageWorkspace(gammaElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs, "601");

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--board"), "601");
    assert.equal(path.basename(argAfter(manifestGenerator, "--default-flash-image")), "bitaxe-gamma601.elf");
    assert.equal(argAfter(manifestGenerator, "--release-name"), "bitaxe-gamma601");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 package whose ELF lacks the 601 marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(productionElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs, "601");

    // Assert
    assert.ok(error instanceof FirmwareBoardMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_board_marker_missing" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 205 package whose ELF carries the 601 marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareBoardMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 core-voltage check package records its variant", async () => {
  // Arrange
  const root = await packageWorkspace(gammaCheckElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-vout-check", runs, "601");

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--firmware-variant"), "gamma601-vout-check");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 mining package records its variant", async () => {
  // Arrange
  const root = await packageWorkspace(gammaMiningElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-mining", runs, "601");

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--firmware-variant"), "gamma601-mining");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("the chip check variant is refused for the Ultra 205 before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaChipCheckElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-chip-check", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("the hash check variant is refused for the Ultra 205 before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaHashCheckElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-hash-check", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("the mining variant is refused for the Ultra 205 before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaMiningElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-mining", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("the core-voltage check variant is refused for the Ultra 205 before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaCheckElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-vout-check", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 production package whose ELF carries the check marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaCheckElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "production", runs, "601");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_in_production" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a test package whose ELF also carries another test variant's marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(Buffer.concat([gammaCheckElf, Buffer.from(` ${OVERHEAT_TRIP_TEST_MARKER}`)]));
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-vout-check", runs, "601");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_foreign" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 overheat package records its variant", async () => {
  // Arrange
  const root = await packageWorkspace(gammaOverheatElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-overheat-trip-test", runs, "601");

    // Assert
    assert.equal(error, undefined);
    const manifestGenerator = runs.find((spec) => spec.args[0] === "package-firmware");
    assert.ok(manifestGenerator !== undefined);
    assert.equal(argAfter(manifestGenerator, "--firmware-variant"), "gamma601-overheat-trip-test");
  } finally {
    await rm(root, { recursive: true });
  }
});

test("the 601 overheat variant is refused for the Ultra 205 before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaOverheatElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-overheat-trip-test", runs, "205");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_board_mismatch" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 overheat package whose ELF also carries the mining marker fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(Buffer.concat([gammaOverheatElf, Buffer.from(` ${GAMMA601_MINING_MARKER}`)]));
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-overheat-trip-test", runs, "601");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_foreign" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});

test("a 601 mining package whose ELF carries the 601 overheat marker instead fails before any tool runs", async () => {
  // Arrange
  const root = await packageWorkspace(gammaOverheatElf);
  const runs: CommandSpec<unknown>[] = [];

  try {
    // Act
    const error = await packageWith(root, "gamma601-mining", runs, "601");

    // Assert
    assert.ok(error instanceof FirmwareVariantMarkerError);
    assert.deepEqual(maybeTypedFailurePublicValue(error), { failure: "firmware_variant_marker_missing" });
    assert.equal(runs.length, 0);
  } finally {
    await rm(root, { recursive: true });
  }
});
