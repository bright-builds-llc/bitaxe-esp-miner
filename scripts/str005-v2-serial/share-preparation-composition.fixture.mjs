// Software-only device/cycle observations; actual local fixture IPC and fresh test-key signing.
import {execFile} from "node:child_process";
import {promisify} from "node:util";
import {mkdtemp,realpath,rm,readFile} from "node:fs/promises";
import {resolve,dirname} from "node:path";
import {tmpdir,networkInterfaces} from "node:os";
import {createShareRoutes} from "./share-routes.mjs";
import {createSigner} from "./signing.mjs";
import {startFixture} from "./fixture-owner.mjs";
import {selectInterface} from "../str005-noise-serial/fixture-owner.mjs";
import {createJournal} from "./journal.mjs";
import {writeNew,proof as readProof} from "../str005-noise-serial/files.mjs";
import {requireGone,requireLsofAbsent} from "../str005-noise-serial/host-resources.mjs";
import {state as baseState,ledger,original} from "../str005-noise-serial/test-fixture.mjs";
import {sha256,check} from "./values.mjs";
const run=promisify(execFile);
export const binding=Buffer.alloc(32,9).toString("base64url"), attemptId=Buffer.alloc(16,1).toString("base64url");
const proof = { schema: "worker-cooling-proof-v1", fan_duty_percent: 100, fan_rpm: 3000, post_command_fan_proven: true, asic_effects: false, budget_reserved: false };
const restoration = { schema: "worker-cooling-baseline-v1", fan_duty_percent: 30, cooling_proven: true, asic_effects: false, budget_reserved: false };
function qualification() {
  return { schema: "worker-qualification-v1", generation: 7, active_ms: 0, generation_elapsed_ms: 0, budget_reserved_ms: 0,
    submitted: 0, accepted: 0, rejected: 0, nonce_work_correlations: 0, work_dispatched: 0, last_valid_heartbeat_ms: 0,
    budget_complete: false, safe_stop_complete: false, voltage_fresh: true, power_fresh: true, temperature_fresh: true, fan_fresh: true, watchdog_alive: true, mine_on_boot: false,
    voltage_volts: 5, power_watts: 3, chip_temp_celsius: 35, fan_rpm: 3000, gate_closed_ms: null, shutdown_started_ms: null,
    safe_stop_stage: "not_started", revocation_reason: "none", active_limit_ms: null, shutdown_budget_ms: 15550, work_gate_remaining_ms: null };
}
export function idle() {
  return { schema: "worker-stratum-v2-status-v1", scope: "share", state: "idle", connection: null, record: null,
    observation: { bootOrdinal: 10, workerGeneration: 7, serialTransportEpoch: 8, observedAtUs: 1000000,
      clockValid: true, stationIpv4: "192.168.1.10", wifiConnected: true, socket: null } };
}
export async function preparationFixture(t) {
  const root=await realpath(await mkdtemp(resolve(tmpdir(),"v2-share-prep-")));
  const gateInput=process.argv[2]; const gateRoot=gateInput ? dirname(resolve(gateInput)) : resolve(process.cwd(),"../bitaxe-turnstile-system");
  const fixtureBinary=resolve(process.argv[3] ?? "bazel-bin/tools/stratum-v2-fixture/stratum_v2_fixture");
  const authority=resolve(root,"generated-test-authority");
  // Keys are generated outside every worktree; no existing private input is read.
  try { await run("bun",[resolve(gateRoot,"scripts/worker-development-authority.ts"),"init","--directory",authority],{cwd:gateRoot,timeout:10000,maxBuffer:4096}); }
  catch (error) {
    const launchCode = typeof error.code === "number" ? error.code : /^[A-Z0-9_]+$/u.test(error.code ?? "") ? error.code : "unknown";
    const scriptPresent = await readFile(resolve(gateRoot,"scripts/worker-development-authority.ts")).then(()=>true,()=>false);
    await rm(root,{recursive:true,force:true}); throw Error(`test_authority_initialization_failed code=${launchCode} script_present=${scriptPresent}`);
  }
  let owner;
  t.after(async()=>{try{if(owner) {await owner.close();const row=(await readProof(root,"fixture-owner.json")).value;await requireGone([row.owner]);requireLsofAbsent(["-nP",`-iTCP:${owner.ready.listenPort}`,"-sTCP:LISTEN","-t"]);}} finally{await rm(root,{recursive:true,force:true});}});
  const context = { scope: "share", attemptId, gate_root: gateRoot, fixture_binary: fixtureBinary, fixture_sha256: sha256(await readFile(fixtureBinary)), gate_commit: "a".repeat(40), firmware_commit: "b".repeat(40), app_elf_sha256: "c".repeat(64),
    client_sha256: "f".repeat(64), qualificationAttempt: { schema: "worker-qualification-attempt-v1", id: attemptId, ordinal: 18, purpose: "normal", maximumActiveMilliseconds: 180000 }, expectedLedgerBefore: ledger };
  const contextSha256 = sha256(JSON.stringify(context)), state = { ...baseState(context), qualification: qualification() }, journal = await createJournal(root, context);
  let offset=0; const now=()=>Math.floor(performance.now())+offset;
  for (let sequence = 1; sequence <= 9; sequence++) await journal.state("candidate", state, now());
  for (let index = 1; index <= 4; index++) await writeNew(resolve(root, `cycle-${index}.json`), { schema: "str005-v2-cycle-v1", contextSha256,
    beforeSequence: index * 2 - 1, afterSequence: index * 2, installReviewSha256: "d".repeat(64),
    report: { schema: "fixed-usb-cycle-report-v1", cycle: index, firmware_commit: context.firmware_commit, app_elf_sha256: context.app_elf_sha256,
      baseline_id: state.preservation.baseline_id, browser_released: true, flash_success: true, runtime_identity_match: true, cleanup_complete: true,
      device_identity_match: true, settings_match: true, authorization_high_water_match: true, probe_request_bytes: 65536, probe_response_bytes: 65536, mine_on_boot: false } });
  const sign=await createSigner(root,context,authority,()=>false); await sign("public-trust");
  let signatures=0, maybeSignHook;
  const operations={now,ready(){},failed:()=>false,journal,fixture:()=>owner,activeScope:()=>({challengeId:"challenge_synthetic"}),verify:async()=>{},
    requireObserver(_observation,supplied){check(supplied===binding,"v2_observer_binding");},
    async sign(operation,input){signatures++;const artifact=await sign(operation,input);await maybeSignHook?.();return artifact;}};
  const routes=createShareRoutes(root,context,operations);
  await writeNew(resolve(root,"accounting-before.json"),{schema:"str005-v2-accounting-v1",contextSha256,observedSequence:journal.lastState().sequence,stage:"before",state,ledger,original_budget:original});
  const cooling=await routes.handle("/cooling-review-context",{});
  await routes.handle("/cooling-review",{nonce:cooling.nonce,proof,restoration,budget_before:ledger,budget_after:ledger,state});
  const review=await routes.handle("/budget-review-context",{});
  await routes.handle("/budget-review",{nonce:review.nonce,report:ledger,controlSessionBindingSha256:binding,state});
  let station;const interfaces=networkInterfaces();
  for(const rows of Object.values(interfaces))for(const row of rows??[])if(row.family==="IPv4"&&!row.internal){try{station=selectInterface(row.address,interfaces).address;}catch{ /* Not an admitted private interface. */ }}
  check(station,"test_private_interface_required");
  const status=()=>{const value=idle();value.observation.stationIpv4=station;return value;};
  const failures=[];
  return {root,context,routes,state,journal,binding,status,signatures:()=>signatures,advance(ms){offset+=ms;},onSign(fn){maybeSignHook=fn;},owner:()=>owner,
    async launch(){owner=await startFixture(root,context,station,code=>failures.push(code),{now});return owner;},
    async network(){return routes.handle("/start/network",{status:status(),controlSessionBindingSha256:binding});},
    async authorize(){return routes.handle("/authorization-context",{controlSessionBindingSha256:binding});},
    async deliver(){return routes.handle("/window-artifacts",undefined,"GET");},failures};
}
