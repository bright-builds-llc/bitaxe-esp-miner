import { contextFixture } from "./context-fixtures.mjs";
import { main } from "./main.mjs";
import { resolve } from "node:path";
const cleanups = [];
try {
  const f = await contextFixture({ after(fn) { cleanups.push(fn); } }, { scope: "share" });
  try {
    await main(["serve", "--private-root", f.root, "--authority-directory", resolve(f.root, "missing-authority")], f.operations);
    process.exitCode = 1;
  } catch (error) {
    if (error.code !== "v2_operator_owner") throw error;
    process.stdout.write(JSON.stringify({ rejected: true, code: error.code }) + "\n");
  }
} finally { for (const cleanup of cleanups.reverse()) await cleanup(); }
