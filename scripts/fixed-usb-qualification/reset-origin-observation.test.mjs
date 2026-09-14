import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import test from 'node:test';
import { createResetOriginObservation } from './reset-origin-observation.mjs';

const firmwareCommit = 'a'.repeat(40), appElfSha256 = 'b'.repeat(64);
const policy = { firmwareCommit, appElfSha256, minimumSpanMs: 120000, maximumGapMs: 6000, startedAtHostMonotonicMs: 0 };
const boot = (uptime_ms, boot_ordinal = 2, reset_reason = 'panic') => ({ category: 'boot', authoritative: false, uptime_ms, boot_ordinal, reset_reason });
const startup = uptime_ms => ({ category: 'startup', authoritative: false, stage: 'runtime_ready', state: 'complete', first_failure: 'none', uptime_ms });
const identity = () => ({ category: 'runtime_identity', authoritative: false, firmware_commit: firmwareCommit, app_elf_sha256: appElfSha256 });
function stream(change = (_time,values)=>values) {
  const reducer = createResetOriginObservation(policy); let sequence = 0;
  for (let time=0;time<=120000;time+=1000) for (const diagnostic of change(time,[boot(time+2000),startup(time+2000),...(time===0?[identity()]:[])])) {
    reducer.observe({ sequence: ++sequence, hostMonotonicMs: time, diagnostic });
  }
  return reducer;
}

test('initial panic reset history is not counted as repeated in-window resets', () => {
  // Arrange / Act
  const result = stream().finish({hostMonotonicMs:120000});
  // Assert
  assert.equal(result.initialResetReason,'panic'); assert.equal(result.observedTransitionCount,0);
  assert.equal(result.priorResetAttribution,'unknown'); assert.equal(result.bootAdvances,120);
  assert.equal(result.healthyStartupAdvances,120); assert.equal(result.segments[0].boot.spanMs,120000);
  assert.equal(result.coverageComplete,true); assert.deepEqual(result.issues,[]); assert.equal(result.authoritative,false);
});

test('real ordinal change is reported separately and does not attribute the preceding reset', () => {
  // Arrange
  const reducer=stream((time,values)=>time<60000?values:[boot(time-60000,3,'software_cpu'),startup(time-60000)]);
  // Act
  const result=reducer.finish({hostMonotonicMs:120000});
  // Assert
  assert.equal(result.observedTransitionCount,1); assert.equal(result.transitions[0].fromOrdinal,2); assert.equal(result.transitions[0].toOrdinal,3);
  assert.equal(result.transitions[0].resetReason,'software_cpu'); assert.equal(result.priorResetAttribution,'unknown');
  assert(!result.issues.includes('startup_uptime_regression'));
});

test('startup reset immediately before its new boot marker is bracketed by that transition', () => {
  // Arrange
  const reducer=createResetOriginObservation(policy);
  [boot(100000),startup(100000),startup(100),boot(200,3,'software_cpu'),startup(300)].forEach((diagnostic,index)=>
    reducer.observe({sequence:index+1,hostMonotonicMs:index*100,diagnostic}));
  // Act
  const result=reducer.finish({hostMonotonicMs:500});
  // Assert
  assert.equal(result.observedTransitionCount,1); assert.equal(result.transitionAssociatedStartupRestarts,1);
  assert(!result.issues.includes('startup_uptime_regression')); assert.equal(result.coverageComplete,false);
});

test('same-boot reverse uptime or reset-reason conflict is not silently treated as a reboot', () => {
  // Arrange
  const reducer=stream((time,values)=>time===60000?[boot(10,2,'brownout'),startup(10)]:values);
  // Act
  const result=reducer.finish({hostMonotonicMs:120000});
  // Assert
  assert.equal(result.observedTransitionCount,0);
  for(const issue of ['boot_uptime_regression','startup_uptime_regression','same_boot_reset_reason_conflict']) assert(result.issues.includes(issue));
});

test('short capture and interior or trailing gaps remain missing coverage', () => {
  // Arrange
  const short=createResetOriginObservation(policy); short.observe({sequence:1,hostMonotonicMs:0,diagnostic:boot(1000)});
  const gap=stream((time,values)=>time>50000&&time<70000?[]:values);
  // Act / Assert
  assert.equal(short.finish({hostMonotonicMs:1000}).coverageComplete,false);
  const interior=gap.finish({hostMonotonicMs:120000}); assert.equal(interior.coverageComplete,false); assert(interior.gaps.bootRecordsMs>6000);
  assert.equal(stream().finish({hostMonotonicMs:127000}).coverageComplete,false);
});

test('identity-only traffic cannot hide missing healthy or boot coverage', () => {
  // Arrange / Act
  const result=stream((_time,_values)=>[identity()]).finish({hostMonotonicMs:120000});
  // Assert
  assert.equal(result.coverageComplete,false); assert(result.issues.includes('boot_observations_missing')); assert(result.issues.includes('healthy_startup_missing'));
});

test('identity conflicts and startup failure are independent closed observations', () => {
  // Arrange
  const reducer=stream((time,values)=>time===60000?[...values,{...identity(),app_elf_sha256:'c'.repeat(64)},
    {...startup(time+2000),first_failure:'network'}]:values);
  // Act / Assert
  const result=reducer.finish({hostMonotonicMs:120000}); assert.equal(result.identityConflicts,1);
  assert.equal(result.startupFailureRecords,1); assert(result.issues.includes('identity_conflict'));
});

test('record sequence and host clock failures stay visible', () => {
  // Arrange
  const reducer=createResetOriginObservation(policy);
  reducer.observe({sequence:1,hostMonotonicMs:1000,diagnostic:boot(1000)});
  reducer.observe({sequence:3,hostMonotonicMs:500,diagnostic:startup(1100)});
  // Act / Assert
  const result=reducer.finish({hostMonotonicMs:120000}); assert(result.issues.includes('record_sequence_gap'));
  assert(result.issues.includes('host_clock_regression')); assert.equal(result.coverageComplete,false);
});

test('retained panic/allocation receipts stay unattributed and Gate string false is handled literally', () => {
  // Arrange
  const reducer=createResetOriginObservation(policy);
  const values=[{category:'panic',authoritative:false,file_hash:'1234abcd',line:8},
    {category:'allocation_failure',authoritative:false,requested_bytes:1024,capabilities:'00000008'},
    {category:'allocation_context',authoritative:false,requested_bytes:1024,capabilities:'00000008',source_hash:'a'.repeat(16),stage:'runtime_ready'},
    {category:'storage_http_status',authoritative:false,spiffs_available:'true',http_ready:'false'}];
  values.forEach((diagnostic,index)=>reducer.observe({sequence:index+1,hostMonotonicMs:0,diagnostic}));
  // Act / Assert
  const result=reducer.finish({hostMonotonicMs:0}); assert.equal(result.unattributedPanicReceiptRecords,1);
  assert.equal(result.unattributedAllocationReceiptRecords,2); assert.equal(result.storageReadyRecords,0); assert.equal(result.storageUnavailableRecords,1);
});

test('malformed required records reject immediately and poison completeness rather than disappearing', () => {
  // Arrange / Act / Assert
  for(const diagnostic of [{...boot(1),private:'fixture'}, {...boot(1),authoritative:'false'}, {...identity(),firmware_commit:'bad'},
    {category:'storage_http_status',authoritative:false,spiffs_available:true,http_ready:false}]) {
    const reducer=createResetOriginObservation(policy);
    assert.throws(()=>reducer.observe({sequence:1,hostMonotonicMs:0,diagnostic}));
    const result=reducer.finish({hostMonotonicMs:120000}); assert(result.issues.includes('invalid_record')); assert.equal(result.coverageComplete,false);
  }
});

test('record and boot-segment limits fail visibly', () => {
  // Arrange
  const records=createResetOriginObservation(policy),segments=createResetOriginObservation(policy);
  for(let sequence=1;sequence<=4096;sequence++) records.observe({sequence,hostMonotonicMs:0,diagnostic:identity()});
  for(let sequence=1;sequence<=8;sequence++) segments.observe({sequence,hostMonotonicMs:sequence,diagnostic:boot(0,sequence)});
  // Act / Assert
  assert.throws(()=>records.observe({sequence:4097,hostMonotonicMs:0,diagnostic:identity()}),/reset_origin_record_bound/u);
  assert.throws(()=>segments.observe({sequence:9,hostMonotonicMs:9,diagnostic:boot(0,9)}),/reset_origin_boot_segment_bound/u);
});

const realCapture=process.env.RESET_ORIGIN_RETAINED_CAPTURE,gateRoot=process.env.RESET_ORIGIN_GATE_ROOT;
test('real recovery-1 capture parsed by the actual Gate parser has advancing one-boot history, not repeated panics',
  {skip:!realCapture||!gateRoot},()=> {
    // Arrange: raw historical input stays in child memory; only closed Gate metadata crosses stdout.
    const code=`const {maybeWorkerSerialDiagnostic}=await import(process.argv.at(-2)); const text=await Bun.file(process.argv.at(-1)).text(); const allowed=new Set(['boot','startup','runtime_identity','panic','allocation_failure','allocation_context','storage_http_status']); process.stdout.write(JSON.stringify(text.split(/\\r?\\n/).map(maybeWorkerSerialDiagnostic).filter(value=>value&&allowed.has(value.category))));`;
    const parsed=spawnSync('bun',['-e',code,resolve(gateRoot,'web/worker-serial-diagnostics.ts'),resolve(realCapture)],{encoding:'utf8',timeout:10000,maxBuffer:1048576});
    assert.equal(parsed.status,0,'Gate parser must complete'); assert.equal(parsed.stderr,'');
    const values=JSON.parse(parsed.stdout),reducer=createResetOriginObservation({...policy,
      firmwareCommit:'daa69bb03274968e78d131e9b0bf13ff12f64ff0',appElfSha256:'49643f417a27b1842865c92109facf253efc8b3837c3906074ccc20a41ce3545'});
    // Act: no historical per-record host timestamps exist, so this deliberately supplies no host coverage.
    values.forEach((diagnostic,index)=>reducer.observe({sequence:index+1,hostMonotonicMs:0,diagnostic}));
    const result=reducer.finish({hostMonotonicMs:0});
    // Assert
    assert.equal(result.initialResetReason,'panic'); assert.equal(result.initialBootOrdinal,2); assert.equal(result.observedTransitionCount,0);
    assert.equal(result.segments[0].boot.records,11); assert.equal(result.bootAdvances,10);
    assert.equal(result.healthyStartupRecords,47); assert.equal(result.healthyStartupAdvances,46);
    assert.equal(result.identityConflicts,0); assert.equal(result.priorResetAttribution,'unknown'); assert.equal(result.coverageComplete,false);
  });

test('backward finish time cannot fabricate a positive observation span', () => {
  // Arrange
  const reducer=createResetOriginObservation({...policy,startedAtHostMonotonicMs:1000});
  reducer.observe({sequence:1,hostMonotonicMs:1000,diagnostic:boot(1000)});
  // Act / Assert
  const result=reducer.finish({hostMonotonicMs:999}); assert.equal(result.hostSpanMs,0);
  assert.equal(result.coverageComplete,false); assert(result.issues.includes('host_clock_regression'));
});
