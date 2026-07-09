#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

function usage(exitCode = 0) {
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`usage: phase28.1.1.1-upstream-nvs-csv.mjs --base-csv <path> --wifi-credentials <path> --pool-credentials <path> --out <path>

Builds an upstream ESP-Miner NVS CSV from local runtime credential files.
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

  const required = ["base-csv", "wifi-credentials", "pool-credentials", "out"];
  for (const key of required) {
    if (!args.get(key)) {
      usage(1);
    }
  }

  return args;
}

function readJsonObject(filePath, label) {
  let parsed;
  try {
    parsed = JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch {
    fail(`${label} file could not be read or parsed`);
  }

  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    fail(`${label} file must contain a JSON object`);
  }

  return parsed;
}

function requiredString(object, field, label) {
  const value = object[field];
  if (typeof value !== "string" || value.trim() === "") {
    fail(`${label} field ${field} must be a non-empty string`);
  }

  if (/[\u0000-\u001F\u007F]/u.test(value)) {
    fail(`${label} field ${field} must not contain control characters`);
  }

  return value;
}

function requiredPort(object, field, label) {
  const value = object[field];
  if (!Number.isInteger(value) || value < 1 || value > 65535) {
    fail(`${label} field ${field} must be an integer from 1 to 65535`);
  }

  return String(value);
}

function parseCsv(text) {
  return text
    .trimEnd()
    .split(/\r?\n/)
    .map((line) => parseCsvLine(line));
}

function parseCsvLine(line) {
  const fields = [];
  let field = "";
  let quoted = false;

  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quoted) {
      if (character === '"' && line[index + 1] === '"') {
        field += '"';
        index += 1;
      } else if (character === '"') {
        quoted = false;
      } else {
        field += character;
      }
      continue;
    }

    if (character === '"') {
      quoted = true;
    } else if (character === ",") {
      fields.push(field);
      field = "";
    } else {
      field += character;
    }
  }

  fields.push(field);
  return fields;
}

function renderCsv(rows) {
  return `${rows.map((row) => row.map(csvCell).join(",")).join("\n")}\n`;
}

function csvCell(value) {
  if (!/[",\r\n]/u.test(value)) {
    return value;
  }

  return `"${value.replaceAll('"', '""')}"`;
}

function setRowValue(rows, key, value) {
  const row = rows.find((candidate) => candidate[0] === key);
  if (!row) {
    fail(`base CSV missing key ${key}`);
  }

  row[3] = value;
}

function fail(message) {
  console.error(`upstream_nvs_csv_error: ${message}`);
  process.exit(1);
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const baseCsvPath = args.get("base-csv");
  const wifiPath = args.get("wifi-credentials");
  const poolPath = args.get("pool-credentials");
  const outPath = args.get("out");

  const rows = parseCsv(fs.readFileSync(baseCsvPath, "utf8"));
  const wifi = readJsonObject(wifiPath, "Wi-Fi credentials");
  const pool = readJsonObject(poolPath, "pool credentials");

  setRowValue(rows, "wifissid", requiredString(wifi, "ssid", "Wi-Fi credentials"));
  setRowValue(rows, "wifipass", requiredString(wifi, "wifiPass", "Wi-Fi credentials"));
  setRowValue(rows, "stratumurl", requiredString(pool, "poolURL", "pool credentials"));
  setRowValue(rows, "stratumport", requiredPort(pool, "poolPort", "pool credentials"));
  setRowValue(rows, "stratumuser", requiredString(pool, "poolUser", "pool credentials"));
  setRowValue(rows, "stratumpass", requiredString(pool, "poolPassword", "pool credentials"));
  setRowValue(rows, "fbstratumurl", requiredString(pool, "poolURL", "pool credentials"));
  setRowValue(rows, "fbstratumport", requiredPort(pool, "poolPort", "pool credentials"));
  setRowValue(rows, "fbstratumuser", requiredString(pool, "poolUser", "pool credentials"));
  setRowValue(rows, "fbstratumpass", requiredString(pool, "poolPassword", "pool credentials"));

  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, renderCsv(rows), "utf8");
}

main();
