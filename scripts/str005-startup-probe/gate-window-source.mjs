// Actual Gate window parsing for the load/start compatibility harnesses. Since Gate 26ab3ab, `loadWindow`
// delegates to `parseAcceptanceWindow` in `web/worker-acceptance-window.ts`; older Gate sources inline it.
import { readFile } from 'node:fs/promises';
import { stripTypeScriptTypes } from 'node:module';
import { resolve } from 'node:path';
import { check, sha256 } from '../str005-v2-serial/values.mjs';

export const WINDOW_SOURCE = 'web/worker-acceptance-window.ts';
const WINDOW_FUNCTIONS = ['exactSixtyTwenty', 'parseAcceptanceWindow'];

/** Extract one named function, including an `async` prefix, by balanced braces. */
export function functionBody(source, name) {
  const start = source.indexOf(`function ${name}(`); check(start >= 0, 'startup_gate_function_missing');
  const opening = source.indexOf('{', source.indexOf(')', start)); let depth = 0;
  for (let index = opening; index < source.length; index++) {
    if (source[index] === '{') depth++;
    if (source[index] === '}' && --depth === 0) return source.slice(source.slice(Math.max(0, start - 6), start) === 'async ' ? start - 6 : start, index + 1);
  }
  check(false, 'startup_gate_function_shape');
}

/** The window parser functions a delegating `loadWindow` needs; none for an older inline source. */
export function windowFunctions(acceptanceSource, maybeWindowSource) {
  if (!acceptanceSource.includes('parseAcceptanceWindow(')) return [];
  check(typeof maybeWindowSource === 'string', 'startup_gate_window_source_missing');
  const converted = stripTypeScriptTypes(maybeWindowSource, { mode: 'transform' });
  return WINDOW_FUNCTIONS.map(name => functionBody(converted, name));
}

/** Test doubles for the window parser's imports; renewal bounds mirror the Gate's general and soak limits. */
export const windowContext = { maximumWindowRenewals: grant => grant.soakAllowance ? 36 : 16 };

export async function maybeReadWindowSource(gateRoot) {
  try { return await readFile(resolve(gateRoot, WINDOW_SOURCE), 'utf8'); } catch (error) {
    if (error.code === 'ENOENT') return undefined;
    throw error;
  }
}

export const windowSourceDigest = maybeWindowSource => maybeWindowSource === undefined ? {} : { windowSourceSha256: sha256(maybeWindowSource) };
