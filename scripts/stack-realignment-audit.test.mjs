import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { auditStackRealignment, parseFunctions, realignmentCallers } from './stack-realignment-audit.mjs';

// Synthetic objdump: one realigning std constructor, a std wrapper, a startup caller, a per-request caller.
const DISASSEMBLY = `
42001000 <std::sync::mpmc::sync_channel>:
42001000:\t034136        \tentry\ta1, 0x1a0
42001003:\tf83c          \tmovi.n\ta8, 63
42001005:\t108180        \tand\ta8, a1, a8
42001008:\t118a          \tadd.n\ta1, a1, a8
4200100a:\tf01d          \tretw.n

42002000 <std::sync::mpsc::sync_channel<bool>>:
42002000:\t006136        \tentry\ta1, 48
42002003:\t5ac581        \tl32r\ta8, 42000f00 <lit+0x10> (42001000 <std::sync::mpmc::sync_channel>)
42002006:\t0008e0        \tcallx8\ta8
42002009:\tf01d          \tretw.n

42003000 <app::writer::send_control>:
42003000:\t006136        \tentry\ta1, 48
42003003:\t000005        \tcall8\t42002000 <std::sync::mpsc::sync_channel<bool>>
42003006:\tf01d          \tretw.n

42004000 <app::startup::prepare>:
42004000:\t006136        \tentry\ta1, 48
42004003:\t5ac581        \tl32r\ta8, 42000f04 <lit+0x14> (42002000 <std::sync::mpsc::sync_channel<bool>>)
42004006:\t0008e0        \tcallx8\ta8
42004009:\tf01d          \tretw.n

42005000 <app::later_math>:
42005000:\t006136        \tentry\ta1, 48
42005003:\t000000        \tnop
42005006:\t000000        \tnop
42005009:\t000000        \tnop
4200500c:\t000000        \tnop
4200500f:\t000000        \tnop
42005012:\t000000        \tnop
42005015:\t000000        \tnop
42005018:\t118a          \tadd.n\ta1, a1, a8
4200501a:\tf01d          \tretw.n

42006000 <_frxt_dispatch>:
42006000:\t2128          \tl32i.n\ta2, a1, 8
42006002:\t118a          \tadd.n\ta1, a1, a8
`;

test('a prologue realignment of a1 marks the function; later words and call0 code do not', () => {
  // Arrange / Act
  const flagged = [...parseFunctions(DISASSEMBLY).values()].filter(fn => fn.realigns).map(fn => fn.name);
  // Assert
  assert.deepEqual(flagged, ['std::sync::mpmc::sync_channel']);
});

test('callers are found through call8 and l32r literals, walking through std wrappers', () => {
  // Arrange / Act
  const { callers } = realignmentCallers(parseFunctions(DISASSEMBLY));
  // Assert
  assert.deepEqual(callers, ['app::startup::prepare', 'app::writer::send_control']);
});

test('a per-request caller outside the startup allowlist blocks the audit', () => {
  // Arrange
  const allowlist = [{ symbol: 'app::startup::prepare', reason: 'startup' }];
  // Act
  const result = auditStackRealignment(DISASSEMBLY, allowlist);
  // Assert
  assert.equal(result.result, 'blocked');
  assert.deepEqual(result.unexpected_callers, ['app::writer::send_control']);
});

test('the audit passes when every caller is allowlisted', () => {
  // Arrange
  const allowlist = ['app::startup::prepare', 'app::writer::send_control'].map(symbol => ({ symbol, reason: 'x' }));
  // Act / Assert
  assert.equal(auditStackRealignment(DISASSEMBLY, allowlist).result, 'no_runtime_realignment_callers');
});

test('the committed allowlist names only startup callers with reasons', () => {
  // Arrange
  const allowlist = JSON.parse(readFileSync(new URL('./stack-realignment-allowlist.json', import.meta.url), 'utf8'));
  // Act / Assert
  assert.equal(allowlist.schema, 'stack-realignment-allowlist-v1');
  for (const entry of allowlist.callers) assert.match(entry.reason, /^startup: /u);
  assert.ok(!allowlist.callers.some(entry => /send_control|acquire|request_safety|queue_safety|bwg::cooling/u.test(entry.symbol)));
});
