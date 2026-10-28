// Every test file must run: a `src/*.test.ts` file that no js_test entry point imports, directly
// or through an aggregator such as `all.test.ts`, is silently skipped by `bazel test`.
import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

/**
 * Test files run only by `bazel run` of a js_binary, by design: the launcher checks need
 * `BUILD_WORKSPACE_DIRECTORY`, which only `bazel run` sets. Each must stay a js_binary entry point.
 */
export const MANUAL_TESTS: ReadonlyMap<string, string> = new Map([
  ["stratum-v2-campaign-launcher.test.ts", "stratum_v2_campaign_launcher_test"],
]);

type EntryPoint = { readonly kind: string; readonly name: string; readonly file: string };

/** Each top-level rule's kind, name and `test-dist` entry point, in `build`. */
export function entryPoints(build: string): EntryPoint[] {
  const starts = [...build.matchAll(/^([a-z_]+)\($/gmu)];
  return starts.flatMap((start, index) => {
    const block = build.slice(start.index, starts[index + 1]?.index ?? build.length);
    const name = /^\s+name = "([^"]+)",$/mu.exec(block)?.[1];
    const entry = /^\s+entry_point = "test-dist\/([a-z0-9.-]+\.test)\.js",$/mu.exec(block)?.[1];
    return name === undefined || entry === undefined ? [] : [{ kind: start[1] ?? "", name, file: `${entry}.ts` }];
  });
}

/** The test files no js_test entry point in `build` reaches through `import "./x.test.js"` lines. */
export function unreachableTests(build: string, sources: ReadonlyMap<string, string>): string[] {
  const pending = entryPoints(build).filter(entry => entry.kind === "js_test").map(entry => entry.file);
  const reached = new Set<string>();
  while (pending.length > 0) {
    const file = pending.pop();
    if (file === undefined || reached.has(file)) continue;
    reached.add(file);
    for (const match of (sources.get(file) ?? "").matchAll(/^import "\.\/([a-z0-9.-]+\.test)\.js";$/gmu)) {
      pending.push(`${match[1]}.ts`);
    }
  }
  return [...sources.keys()].filter(file => file.endsWith(".test.ts") && !reached.has(file)).sort();
}

async function packageRoot(): Promise<string> {
  const maybeRunfiles = process.env["RUNFILES_DIR"];
  const candidates = [
    ...(maybeRunfiles === undefined ? [] : [path.join(maybeRunfiles, "_main", "tools/automation")]),
    path.join(process.cwd(), "tools/automation"),
    process.cwd(),
  ];
  for (const candidate of candidates) {
    try {
      await readFile(path.join(candidate, "BUILD.bazel"), "utf8");
      return candidate;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  throw new Error("missing tools/automation/BUILD.bazel");
}

/** The package's build file and every `src/**` TypeScript source, keyed by path under `src`. */
async function packageSources() {
  const root = await packageRoot();
  const build = await readFile(path.join(root, "BUILD.bazel"), "utf8");
  const names = (await readdir(path.join(root, "src"), { recursive: true })).filter(name => name.endsWith(".ts"));
  const sources = new Map<string, string>();
  for (const name of names) sources.set(name, await readFile(path.join(root, "src", name), "utf8"));
  return { build, sources };
}

test("an orphaned test file is reported", () => {
  // Arrange
  const build = 'js_test(\n    name = "all",\n    entry_point = "test-dist/all.test.js",\n)\n';
  const sources = new Map([
    ["all.test.ts", 'import "./a.test.js";\n'],
    ["a.test.ts", 'import "./b.test.js";\n'],
    ["b.test.ts", ""],
    ["orphan.test.ts", ""],
    ["helper.ts", ""],
  ]);
  // Act
  const orphans = unreachableTests(build, sources);
  // Assert
  assert.deepEqual(orphans, ["orphan.test.ts"]);
});

test("a js_binary entry point runs nothing under bazel test", () => {
  // Arrange
  const build = [
    'js_binary(\n    name = "launcher",\n    entry_point = "test-dist/launcher.test.js",\n)\n',
    'js_test(\n    name = "all",\n    entry_point = "test-dist/all.test.js",\n)\n',
  ].join("\n");
  const sources = new Map([["all.test.ts", ""], ["launcher.test.ts", ""]]);
  // Act
  const orphans = unreachableTests(build, sources);
  // Assert
  assert.deepEqual(orphans, ["launcher.test.ts"]);
});

test("every automation test file runs under a js_test entry point, or is a named manual launcher", async () => {
  // Arrange
  const { build, sources } = await packageSources();
  // Act
  const orphans = unreachableTests(build, sources);
  // Assert
  assert.ok(sources.has("all.test.ts"), "the source tree was found");
  assert.ok(entryPoints(build).some(entry => entry.kind === "js_test" && entry.file === "all.test.ts"));
  assert.deepEqual(orphans, [...MANUAL_TESTS.keys()].sort());
});

test("each manual launcher test is still a js_binary entry point", async () => {
  // Arrange
  const { build } = await packageSources();
  const binaries = entryPoints(build).filter(entry => entry.kind === "js_binary");
  // Act / Assert
  for (const [file, target] of MANUAL_TESTS) {
    assert.ok(binaries.some(entry => entry.file === file && entry.name === target), `${target} runs ${file}`);
  }
});
