// Runs the pinned Gate acceptance page, exactly as the supervisor serves it, over a virtual device.
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { childPort, releaseDevice, startDevice } from '../gate-serial/child-port.mjs';
import { installBrowser } from './browser-shims.mjs';

const options = Object.fromEntries(process.argv.slice(2).map(argument => {
  const match = /^--([a-z-]+)=(.*)$/u.exec(argument);
  if (!match) throw Error('fixture_arguments');
  return [match[1], match[2]];
}));
// The no-mining supervisor injects its own client; the lease supervisor the production one.
const LEASE = options.mode.startsWith('lease');
const SERVED_SCRIPTS = ['/dist/worker-serial-acceptance/worker-serial-acceptance.js', LEASE ? '/client.mjs' : '/no-mining-client.mjs'];
const outcomePath = join(options.work, 'outcome.json');
const device = startDevice({ binary: options.device, seed: 7, capability: options.capability, trust: options['device-trust'], outcome: outcomePath,
  stderrPath: join(options.work, 'device.stderr') });
const browser = installBrowser({ origin: options.origin, port: childPort(device) });

async function until(predicate, boundMs, category) {
  for (const started = performance.now(); !predicate();) {
    if (performance.now() - started > boundMs) throw Error(category);
    await new Promise(resolve => setTimeout(resolve, 20));
  }
}
const state = () => globalThis.workerAcceptance.state();
// What the page last published, as the supervisor sees it; connect publishes only once
// its baseline status has settled, while `state()` reports `connected` before that.
const published = () => {
  const text = browser.byId('state').textContent;
  return text ? JSON.parse(text) : {};
};
const click = id => browser.byId(id).dispatch('click', { isTrusted: true });

// Load the page's module graph from the served bytes, in document order, from a directory
// created now: Bun's module resolver caches a directory listing (here, its startup cwd).
const servedRoot = join(options.work, 'served');
await mkdir(servedRoot, { mode: 0o700 });
const page = await (await fetch('/')).text();
const scripts = [...page.matchAll(/<script type="module" src="([^"]+)"><\/script>/gu)].map(match => match[1]);
if (JSON.stringify(scripts) !== JSON.stringify(SERVED_SCRIPTS)) throw Error('served_page_scripts');
// Every file is written before the first import: Bun lists a directory once it resolves from it.
const files = [];
for (const [index, path] of scripts.entries()) {
  const response = await fetch(path);
  if (!response.ok) throw Error('served_script_unavailable');
  const file = join(servedRoot, `${index}.mjs`);
  await writeFile(file, Buffer.from(await response.arrayBuffer()), { flag: 'wx', mode: 0o600 });
  files.push(file);
}
for (const file of files) await import(pathToFileURL(file).href);

const result = { mode: options.mode, statuses: [] };
/**
 * Prepare, load and start a signed window as the operator's buttons do; then wait for the
 * page's own first renewal and stop (`lease`), or leave the lease to Close (`lease-close`).
 */
async function lease() {
  const page = globalThis.workerAcceptance;
  await page.prepareStartAuthorization();
  await page.loadSignedWindow();
  observe();
  await page.startWindow();
  observe();
  if (options.mode === 'lease-close') return;
  // A refresh publishes after each renewal; the next tick is a second away, so Stop never races it.
  await until(() => published().renewalsConfirmed >= 1 || !published().running, 40000, 'renewal_unconfirmed');
  result.renewals = published().renewalsConfirmed;
  observe();
  await page.stop();
}
const observe = () => { const status = state().status; if (result.statuses.at(-1) !== status) result.statuses.push(status); };
const watcher = setInterval(observe, 5);
try {
  await until(() => state().status === 'configured', 10000, 'page_not_configured');
  observe();
  click('connect');
  await until(() => published().connected || published().failure !== undefined, 20000, 'connect_unsettled');
  if (published().connected && options.mode === 'loss') {
    process.kill(-device.child.pid, 'SIGKILL');
    await globalThis.workerAcceptance.refresh().catch(() => {});
    await until(() => !state().connected, 10000, 'loss_unobserved');
  } else if (published().connected && LEASE) {
    await lease();
  } else if (published().connected) {
    await globalThis.workerAcceptance.refresh();
  }
  observe();
  click('close');
  await until(() => state().serialOwnershipReleased || state().status === 'closed', 20000, 'close_unsettled');
  observe();
  await (globalThis.noMiningSupervisor ?? globalThis.recoverySupervisor).flush();
  result.records_flushed = true;
} finally {
  clearInterval(watcher);
  const final = state();
  result.final = Object.fromEntries(['status', 'connected', 'serialOwnershipReleased', 'deviceLeaseInactive', 'failure',
    'admissionFailureStage', 'serialFailureCategory'].map(key => [key, final[key] ?? null]));
  result.lock_held = browser.lockHeld();
  result.device = await releaseDevice(device);
}
try { result.outcome = JSON.parse(await readFile(outcomePath, 'utf8')); } catch (error) {
  // A killed device writes no outcome; anything else is a harness failure.
  if (error.code !== 'ENOENT') throw error;
  result.outcome = null;
}
process.stdout.write(`${JSON.stringify(result)}\n`);
