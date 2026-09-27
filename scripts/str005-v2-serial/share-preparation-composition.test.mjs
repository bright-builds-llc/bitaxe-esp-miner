import test from "node:test";
import assert from "node:assert/strict";
import {lstat,readFile} from "node:fs/promises";
import {resolve} from "node:path";
import {preparationFixture} from "./share-preparation-composition.fixture.mjs";
import {createV2Coordinator} from "./client.mjs";
import {clientFixture} from "./client.test-helper.mjs";
import {proof} from "../str005-noise-serial/files.mjs";

async function coordinatorFixture(t, mode) {
  const f=await preparationFixture(t), device=clientFixture("share");let pageOffset=0, maybeRequestAt, maybeStartAt, loads=0, starts=0;
  const now=()=>performance.now()+pageOffset;
  const originalLoad=device.gate.loadSignedWindow;
  device.gate.stratumV2Status=async()=>f.status();
  device.gate.stratumV2TelemetryEndpoint=async()=>({controlSessionBindingSha256:f.binding});
  device.gate.prepareStartAuthorization=async()=>{await f.authorize();if(mode==="late-sign")pageOffset+=10001;return{controlSessionBindingSha256:f.binding};};
  device.gate.loadSignedWindow=async()=>{const artifacts=await f.deliver();assert.equal(artifacts.renewals.length,9);loads++;await originalLoad();if(mode==="late-delivery")pageOffset+=10001;};
  device.gate.startWindow=async()=>{f.owner().requireStartWindow();starts++;maybeStartAt=now();throw Error("simulated-device-start-boundary");};
  const request=async(path,input,method)=>{
    if(path==="/fixture/start"){maybeRequestAt=now();await f.launch();return{fixture_ready:true,attemptId:f.context.attemptId};}
    if(path==="/start/network"){const result=await f.network();if(mode==="page-expired")pageOffset+=10001;return result;}
    return device.request(path,input,method);
  };
  const coordinator=createV2Coordinator({gate:device.gate,request,published:()=>device.state,now,sleep:async()=>{},notice:()=>{}});
  return{...f,run:()=>coordinator.supervisor.run(),counts:()=>({loads,starts}),pageElapsed:()=>maybeStartAt-maybeRequestAt};
}
test("actual fixture and generated test-key signer prepare within both clock windows and release all resources",{timeout:30000},async t=>{
  // Arrange: Gate/device boundary is simulated; fixture IPC, signing, claims and cleanup are production.
  const f=await coordinatorFixture(t,"pass");
  // Act: stop exactly at simulated Start, never contact or control a device.
  await assert.rejects(f.run());
  // Assert
  assert.deepEqual(f.counts(),{loads:1,starts:1});assert.ok(f.pageElapsed()<=10000);assert.equal(f.signatures(),10);
  const issued=(await proof(f.root,"issued.json")).value,delivered=(await proof(f.root,"consumed.json")).value;
  assert.equal(issued.authorizationCount,10);assert.equal(issued.privatePayloadPersisted,false);assert.equal(delivered.deliveryAttempted,true);assert.equal(delivered.deviceReservationObserved,false);
});
test("page clock already expired blocks issuance before real signing",{timeout:30000},async t=>{
  const f=await coordinatorFixture(t,"page-expired");await assert.rejects(f.run());
  assert.equal(f.signatures(),0);assert.deepEqual(f.counts(),{loads:0,starts:0});await assert.rejects(lstat(resolve(f.root,"issuance.claim.json")),{code:"ENOENT"});
});
test("fixture clock expiry rejects a fresh network review before issuance",{timeout:30000},async t=>{
  const f=await preparationFixture(t);await f.launch();f.advance(10001);await f.network();
  await assert.rejects(f.authorize(),{code:"v2_fixture_start_deadline"});assert.equal(f.signatures(),0);
  await assert.rejects(lstat(resolve(f.root,"issuance.claim.json")),{code:"ENOENT"});
});
for(const mode of ["late-sign","late-delivery"])test(`${mode} retains real issued/delivery facts but prohibits simulated Start`,{timeout:30000},async t=>{
  const f=await coordinatorFixture(t,mode);await assert.rejects(f.run());assert.equal(f.signatures(),10);assert.equal(f.counts().starts,0);
  assert.equal((await proof(f.root,"issued.json")).value.authorizationCount,10);await readFile(resolve(f.root,"issuance.claim.json"));
  if(mode==="late-sign"){assert.equal(f.counts().loads,0);await assert.rejects(lstat(resolve(f.root,"consumed.json")),{code:"ENOENT"});}
  else{assert.equal(f.counts().loads,1);assert.equal((await proof(f.root,"consumed.json")).value.deliveryAttempted,true);}
});
test("late server-side signature retains consumed claim and actual signer exit without pretending issuance finished",{timeout:30000},async t=>{
  const f=await preparationFixture(t);await f.launch();await f.network();f.onSign(()=>f.advance(10001));
  await assert.rejects(f.authorize());assert.equal(f.signatures(),1);await readFile(resolve(f.root,"issuance.claim.json"));
  assert.equal((await proof(f.root,"signer-02.exit.json")).value.observation.code,0);
  for(const name of ["issued.json","consumed.json"])await assert.rejects(lstat(resolve(f.root,name)),{code:"ENOENT"});
});
