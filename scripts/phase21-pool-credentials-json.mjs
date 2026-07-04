#!/usr/bin/env node
import { readFileSync } from "node:fs";

const [, , credentialsPath] = process.argv;

if (!credentialsPath) {
  fail("pool credentials JSON path is required");
}

let credentials;
try {
  credentials = JSON.parse(readFileSync(credentialsPath, "utf8"));
} catch {
  fail("pool credentials JSON could not be read or parsed");
}

if (!credentials || typeof credentials !== "object" || Array.isArray(credentials)) {
  fail("pool credentials JSON must be an object");
}

const poolURL = requiredString("poolURL");
const poolPort = requiredPort("poolPort");
const poolUser = requiredString("poolUser");
const poolPassword = requiredString("poolPassword");

console.log(`BITAXE_POOL_URL=${shellQuote(poolURL)}`);
console.log(`BITAXE_POOL_PORT=${shellQuote(String(poolPort))}`);
console.log(`BITAXE_POOL_USER=${shellQuote(poolUser)}`);
console.log(`BITAXE_POOL_PASSWORD=${shellQuote(poolPassword)}`);

function requiredString(fieldName) {
  const value = credentials[fieldName];
  if (typeof value !== "string" || value.trim() === "") {
    fail(`pool credentials JSON field ${fieldName} must be a non-empty string`);
  }

  if (/[\u0000-\u001F\u007F]/u.test(value)) {
    fail(`pool credentials JSON field ${fieldName} must not contain control characters`);
  }

  return value;
}

function requiredPort(fieldName) {
  const value = credentials[fieldName];
  if (!Number.isInteger(value) || value < 1 || value > 65535) {
    fail(`pool credentials JSON field ${fieldName} must be an integer from 1 to 65535`);
  }

  return value;
}

function shellQuote(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

function fail(message) {
  console.error(`pool_credentials_json_error: ${message}`);
  process.exit(1);
}
