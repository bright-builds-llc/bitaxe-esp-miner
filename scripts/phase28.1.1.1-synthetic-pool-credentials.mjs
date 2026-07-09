#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.1-synthetic-pool-credentials.mjs --ready-json <path> --host <host> --out <path>

Writes ignored synthetic pool credentials for a Phase 28.1.1.1 fake-pool session.
No credential values are printed.
`);
  process.exit(exitCode);
}

function parseArgs(argv) {
  const args = new Map();
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      usage(0);
    }
    if (!arg.startsWith("--")) {
      usage(1);
    }

    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      usage(1);
    }
    args.set(arg.slice(2), value);
    index += 1;
  }

  for (const key of ["ready-json", "host", "out"]) {
    if (!args.get(key)) {
      usage(1);
    }
  }

  return args;
}

function fail(message) {
  console.error(`phase28_synthetic_pool_credentials_error: ${message}`);
  process.exit(1);
}

function requiredHost(value) {
  if (typeof value !== "string" || value.trim() === "") {
    fail("--host must be non-empty");
  }
  if (/[\u0000-\u001F\u007F]/u.test(value)) {
    fail("--host must not contain control characters");
  }
  return value.trim();
}

function requiredPort(value) {
  if (!Number.isInteger(value) || value < 1 || value > 65535) {
    fail("ready JSON bound_port must be an integer from 1 to 65535");
  }
  return value;
}

const args = parseArgs(process.argv.slice(2));
const host = requiredHost(args.get("host"));
const outPath = args.get("out");
let ready;
try {
  ready = JSON.parse(fs.readFileSync(args.get("ready-json"), "utf8"));
} catch {
  fail("ready JSON could not be read or parsed");
}

const poolCredentials = {
  poolURL: host,
  poolPort: requiredPort(ready.bound_port),
  poolUser: "phase28-source-work-user",
  poolPassword: "phase28-source-work-password",
};

fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(poolCredentials, null, 2)}\n`, "utf8");
