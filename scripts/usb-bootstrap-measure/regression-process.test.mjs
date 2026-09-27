import { nodeRuntimeEnvironment } from "../str005-noise-serial/node-runtime.mjs";
import test from "node:test";
import assert from "node:assert/strict";
import { runRegression } from "./regression-process.mjs";
const options = { cwd: process.cwd(), env: { PATH: process.env.PATH, LANG: "C", LC_ALL: "C", ...nodeRuntimeEnvironment() }, timeout: 5000, maxBuffer: 65536 };
test("fast successful program retains actual output and exit after owner cleanup", async () => {
  const result = await runRegression(process.execPath, ["-e", "process.stdout.write('ok');process.stderr.write('err')"], options);
  assert.equal(result.status, 0); assert.equal(result.signal, null); assert.equal(result.error, undefined);
  assert.equal(result.stdout.toString(), "ok"); assert.equal(result.stderr.toString(), "err");
});
test("ignored TERM cannot make the regression timeout unbounded", async () => {
  const start = performance.now();
  const result = await runRegression(process.execPath, ["-e", "process.on('SIGTERM',()=>{});setInterval(()=>{},1000)"], { ...options, timeout: 3000 });
  assert.equal(result.error.code, "bootstrap_reader_runner_timeout"); assert.ok(performance.now() - start < 3500);
});
test("combined stream overflow is bounded and cleanup completes", async () => {
  const result = await runRegression(process.execPath, ["-e", "process.stdout.write('regression-started:'+ 'x'.repeat(80));process.stderr.write('y'.repeat(80));setInterval(()=>{},1000)"], { ...options, maxBuffer: 100 });
  assert.equal(result.error.code, "bootstrap_reader_runner_output"); assert.equal(result.stdout.length + result.stderr.length, 100);
  assert.ok(result.stdout.toString().startsWith("regression-started:"));
});
test("a program leaving a child process cannot pass and the child is reaped from the owned group", async () => {
  const script = "const{spawn}=require('node:child_process');spawn(process.execPath,['-e',\"process.on('SIGTERM',()=>{});setInterval(()=>{},1000)\"],{stdio:'ignore'}).unref();";
  const result = await runRegression(process.execPath, ["-e", script], options);
  assert.equal(result.status, 0); assert.equal(result.error.code, "bootstrap_reader_runner_descendants");
});
test("zero output capacity is rejected before launch", async () => {
  await assert.rejects(runRegression(process.execPath, ["-e", "throw Error('must not execute')"], { ...options, maxBuffer: 0 }), /bootstrap_reader_runner_bounds/u);
});
test("observed detached descendant is cleaned after its immediate parent exits", async () => {
  const script = "const{spawn}=require('node:child_process');const child=spawn(process.execPath,['-e',\"process.on('SIGTERM',()=>{});setInterval(()=>{},1000)\"],{detached:true,stdio:'ignore'});process.stdout.write(String(child.pid));child.unref();setTimeout(()=>{},300);";
  const result = await runRegression(process.execPath, ["-e", script], options);
  assert.equal(result.error.code, "bootstrap_reader_runner_descendants");
  const pid = Number(result.stdout.toString()); assert.ok(Number.isSafeInteger(pid) && pid > 0);
  assert.throws(() => process.kill(pid, 0), { code: "ESRCH" });
});
test("program spawn failure stays failed and releases the observed wrapper", async () => {
  const result = await runRegression("/definitely-missing-regression-program", [], options);
  assert.equal(result.status, null); assert.equal(result.error.code, "bootstrap_reader_runner_spawn");
});
