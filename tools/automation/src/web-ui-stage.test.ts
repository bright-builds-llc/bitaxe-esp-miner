import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
import test from "node:test";

import { budgetViolations, sizeHistoryRows, spiffsObjectPages, spiffsUsage, variantSize } from "./web-ui-sizes.js";
import { deterministicGzip, stageWebUi, verifyGzipSiblings, wantsGzipSibling, type StagedFile } from "./web-ui-stage.js";
import { parseStageArguments } from "./web-ui-stage-cli.js";

function file(path: string, text: string): StagedFile {
  return { path, bytes: Buffer.from(text) };
}

const css = "body { color: white; }\n".repeat(40);

test("deterministic gzip is byte-reproducible and round-trips", () => {
  // Arrange
  const source = Buffer.from(css);

  // Act
  const first = deterministicGzip(source);
  const second = deterministicGzip(source);

  // Assert
  assert.ok(first.equals(second));
  assert.equal(first.readUInt32LE(4), 0);
  assert.equal(first[9], 3);
  assert.ok(gunzipSync(first).equals(source));
});

test("current keeps only its evidence-bound gzip sibling and solid compresses hashed assets", () => {
  // Arrange
  const paths = ["index.html", "assets/app.css", "assets/app.js", "assets/index-AbCd1234.js", "assets/release.json"];

  // Act
  const current = paths.filter((path) => wantsGzipSibling("current", path));
  const solid = paths.filter((path) => wantsGzipSibling("solid", path));

  // Assert
  assert.deepEqual(current, ["assets/app.css"]);
  assert.deepEqual(solid, ["assets/app.css", "assets/app.js", "assets/index-AbCd1234.js"]);
});

test("staging adds verified gzip siblings and digests for every staged file", () => {
  // Arrange
  const sources = [file("index.html", "<!doctype html>"), file("assets/app.css", css)];

  // Act
  const { files, metadata } = stageWebUi("current", sources);

  // Assert
  assert.deepEqual(files.map((staged) => staged.path), ["assets/app.css", "assets/app.css.gz", "index.html"]);
  assert.equal(metadata.variant, "current");
  assert.equal(metadata.files[1]?.gzip_of, "assets/app.css");
  assert.match(metadata.files[2]?.sha256 ?? "", /^[0-9a-f]{64}$/u);
  assert.doesNotThrow(() => verifyGzipSiblings(files));
});

test("staging rejects committed gzip files, reserved names and over-long SPIFFS names", () => {
  // Arrange
  const index = file("index.html", "<!doctype html>");
  const longName = `assets/${"a".repeat(60)}.js`;

  // Act
  const attempts = [
    () => stageWebUi("current", [index, file("assets/app.css.gz", "stale")]),
    () => stageWebUi("current", [index, file("version.txt", "label\n")]),
    () => stageWebUi("solid", [index, file(longName, "x")]),
    () => stageWebUi("solid", [file("assets/app.js", "x")]),
    () => stageWebUi("solid", [index, file("../escape.js", "x")]),
  ];

  // Assert
  for (const attempt of attempts) assert.throws(attempt);
});

test("a gzip file that no longer matches its source is rejected", () => {
  // Arrange
  const stale = [file("assets/app.css", css), { path: "assets/app.css.gz", bytes: deterministicGzip(Buffer.from("old")) }];

  // Act
  const check = () => verifyGzipSiblings(stale);

  // Assert
  assert.throws(check, /does not match its source/u);
});

test("variant sizes count gzip siblings as served bytes and fail an exceeded budget", () => {
  // Arrange
  const { files } = stageWebUi("current", [file("index.html", "<!doctype html>"), file("assets/app.css", css)]);

  // Act
  const size = variantSize("current", files);
  const violations = budgetViolations([size], { schema_version: "bitaxe-web-ui-budget-v1", max_gzip_bytes: { current: 10, solid: 10 } });

  // Assert
  assert.equal(size.assets.length, 2);
  assert.equal(size.rawBytes, css.length + 15);
  assert.ok(size.servedBytes < size.rawBytes);
  assert.equal(violations.length, 1);
  assert.match(sizeHistoryRows("2026-10-07T00:00:00.000Z", "0123456789ab", false, [size], new Map())[0] ?? "", /^2026-10-07T00:00:00.000Z,0123456789ab,false,current,2,/u);
});

test("SPIFFS page accounting reads object lookup pages", () => {
  // Arrange
  const image = Buffer.alloc(2 * 4096, 0xff);
  image.writeUInt16LE(0x8001, 0);
  image.writeUInt16LE(0x0001, 2);
  image.writeUInt16LE(0x0000, 4);

  // Act
  const usage = spiffsUsage(image);

  // Assert
  assert.deepEqual(usage, { imageBytes: 8192, blocks: 2, usedPages: 2, usablePages: 30, usedBytes: 512, usableBytes: 7680 });
  assert.equal(spiffsObjectPages(0), 1);
  assert.equal(spiffsObjectPages(251 * 88), 90);
});

test("stage arguments accept repeated inputs and inline values", () => {
  // Arrange
  const argv = ["--variant=solid", "--out", "out", "--input", "a", "--input=b"];

  // Act
  const parsed = parseStageArguments(argv);

  // Assert
  assert.deepEqual(parsed, { variant: "solid", out: "out", stripPrefix: "", inputs: ["a", "b"] });
});
