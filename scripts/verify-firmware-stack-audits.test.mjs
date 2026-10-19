import test from 'node:test';
import assert from 'node:assert/strict';
import { stackAuditRows } from './verify-firmware-stack-audits.mjs';

test('an audit that cannot find its owner is a blocked row, never a skipped one', () => {
  // Arrange: valid stack contracts over an ELF with no functions.
  const sources = { usbOwner: '.name("bwg-serial-writer".into())\n        .stack_size(8192)\n', preparedLink: '',
    httpServer: '', bapAdapter: '' };
  // Act
  const rows = stackAuditRows('', sources);
  // Assert
  assert.deepEqual(rows.find((row) => row.audit === 'writer'),
    { audit: 'writer', result: 'blocked', error: 'control_stack_root_missing' });
});
