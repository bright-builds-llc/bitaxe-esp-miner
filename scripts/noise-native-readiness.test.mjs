import assert from 'node:assert/strict';
import test from 'node:test';
import { auditNoiseEntry, requireNoiseConfiguration, NOISE_ENTRY_SYMBOL } from './noise-native-readiness.mjs';
const source = '// Existing borrowed job, no thread allocation';
const transportSource = 'const WORKER_STACK_BYTES: usize = 12 * 1024;\n.stack_size(WORKER_STACK_BYTES)';
const fixture = (bytes = 1024, extra = '') => `42001000 <${NOISE_ENTRY_SYMBOL}>:\n42001000: 000000 entry a1, ${bytes}\n${extra}42001003: 000000 retw.n\n`;
const configuration = `CONFIG_SPIRAM_MALLOC_RESERVE_INTERNAL=98304
CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384
CONFIG_ESP_MAIN_TASK_AFFINITY=0x0
CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5
CONFIG_SPIRAM_TRY_ALLOCATE_WIFI_LWIP=y
CONFIG_ESP_WIFI_STATIC_RX_BUFFER_NUM=6
CONFIG_ESP_WIFI_STATIC_TX_BUFFER_NUM=6
CONFIG_ESP_WIFI_TX_BUFFER_TYPE=0
CONFIG_ESP_WIFI_DYNAMIC_RX_BUFFER_NUM=32
CONFIG_ESP_WIFI_AMPDU_RX_ENABLED=y
CONFIG_ESP_WIFI_RX_BA_WIN=12
`;

test('selected native frame at the entry budget is admissible without a whole-worker claim', () => {
  // Arrange / Act
  const result = auditNoiseEntry(fixture(8192), source, transportSource);
  // Assert
  assert.equal(result.entryBytes, 8192); assert.equal(result.stackBytes, 12288);
  assert.equal(result.completeCallgraphBound, false);
});
for (const bytes of [0, 31, 33, 8208, 24576]) {
  test(`entry frame ${bytes} cannot be represented as a qualified small entry`, () => {
    // Arrange / Act / Assert
    assert.throws(() => auditNoiseEntry(fixture(bytes), source, transportSource), /noise_native_entry_budget/u);
  });
}
test('duplicate entry symbol is not silently selected', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditNoiseEntry(fixture() + fixture(), source, transportSource), /noise_native_entry_missing_or_multiple/u);
});
test('dynamic stack adjustment prevents a fixed-frame claim', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditNoiseEntry(fixture(1024, '42001002: 000000 addi a1, a1, -16\n'), source, transportSource), /noise_native_dynamic_stack/u);
});
test('missing worker entry never reuses another owner frame', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditNoiseEntry(fixture().replace(NOISE_ENTRY_SYMBOL, 'bitaxe_production_owner_entry'), source, transportSource), /noise_native_entry_missing_or_multiple/u);
});
test('a changed stack declaration needs review instead of inheriting the old bound', () => {
  // Arrange / Act / Assert
  assert.throws(() => auditNoiseEntry(fixture(), source, transportSource.replace('12 * 1024', '24 * 1024')), /noise_native_stack_declaration/u);
});
test('preserved configuration admits the existing owner and buffer budgets', () => {
  // Arrange / Act / Assert
  assert.doesNotThrow(() => requireNoiseConfiguration(configuration));
});
for (const [name, text] of [
  ['lower internal reserve', configuration.replace('98304', '65536')],
  ['changed main affinity', configuration.replace('0x0', '0x1')],
  ['duplicate stack assignment', configuration + 'CONFIG_ESP_MAIN_TASK_STACK_SIZE=16384\n'],
  ['missing Wi-Fi buffer contract', configuration.replace('CONFIG_ESP_WIFI_RX_BA_WIN=12\n', '')],
]) {
  test(`${name} is rejected`, () => {
    // Arrange / Act / Assert
    assert.throws(() => requireNoiseConfiguration(text), /noise_native_configuration/u);
  });
}

const loopName = 'bitaxe_firmware::production_mining_session::transport::run_worker';
const dispatchName = 'bitaxe_firmware::production_mining_session::transport::borrow::NoiseBorrowWorker::run_job';
const prepareName = 'rustsecp256k1_v0_9_2_ellswift_encode';
const authName = 'noise_sv2::initiator::Initiator::step_2_with_now';
const verifyName = 'rustsecp256k1_v0_9_2_schnorrsig_verify';
const boundSource = '.dispatch(owner_entry, job_completed)';
const boundTransport = transportSource + '\nborrow.run_job()';
const boundBorrow = '(callbacks.run)();\n(callbacks.complete)();';
function selectedFixture({ auth = 4096, verify = 4096, includePrepare = true } = {}) {
  const rows = [
    [loopName, 128, [1]], [dispatchName, 64, []], [NOISE_ENTRY_SYMBOL, 64, includePrepare ? [3, 4] : [4]],
    [prepareName, 4096, []], [authName, auth, [5]], [verifyName, verify, []],
    ['std::sys::backtrace::__rust_begin_short_backtrace', 32, [0]],
    ['core::ops::function::FnOnce::call_once{{vtable.shim}}', 112, [6]],
    ['std::sys::pal::unix::thread::Thread::new::thread_start', 32, []], ['pthread_task_func', 32, []],
  ];
  const address = i => (0x42001000 + i * 0x100).toString(16);
  return rows.map(([name, bytes, targets], i) => `${address(i)} <${name}>:\n${address(i)}: 000000 entry a1, ${bytes}\n`
    + targets.map((target, k) => `${(parseInt(address(i), 16) + 3 * (k + 1)).toString(16)}: 000000 call8 ${address(target)} <${rows[target][0]}>\n`).join('')
    + `${(parseInt(address(i), 16) + 3 * (targets.length + 1)).toString(16)}: 000000 retw.n\n`).join('\n');
}

test('borrowed worker audit takes the maximum resolved crypto chain, not sequential-phase sum', async () => {
  // Arrange
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  // Act
  const result = auditBorrowedNoisePaths(selectedFixture(), boundSource, boundTransport, boundBorrow);
  // Assert
  assert.equal(result.selectedPathBytes, 8656); assert.equal(result.addedStackBytes, 0);
  assert.equal(result.completeCallgraphBound, false);
  assert.equal(result.callbackBinding, 'source_bound_fn_pointer_registration');
});

test('a known crypto chain exceeding the existing transport stack cannot qualify', async () => {
  // Arrange
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  // Act / Assert
  assert.throws(() => auditBorrowedNoisePaths(selectedFixture({ auth: 8192 }), boundSource, boundTransport, boundBorrow), /noise_native_selected_path_budget/u);
});
test('the selected chain must leave the prospective platform margin', async () => {
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  const result = auditBorrowedNoisePaths(selectedFixture({ auth: 7216 }), boundSource, boundTransport, boundBorrow);
  assert.equal(result.remainingStackBytes, 512);
  assert.throws(() => auditBorrowedNoisePaths(selectedFixture({ auth: 7232 }), boundSource, boundTransport, boundBorrow), /noise_native_selected_path_budget/u);
});
test('a missing measured thread caller cannot be replaced by a zero frame', async () => {
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  const fixture = selectedFixture().replace('core::ops::function::FnOnce::call_once{{vtable.shim}}', 'unrelated');
  assert.throws(() => auditBorrowedNoisePaths(fixture, boundSource, boundTransport, boundBorrow), /noise_native_thread_caller/u);
});

test('missing resolved preparation path is unproved rather than a zero-cost operation', async () => {
  // Arrange
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  // Act / Assert
  assert.throws(() => auditBorrowedNoisePaths(selectedFixture({ includePrepare: false }), boundSource, boundTransport, boundBorrow), /noise_native_crypto_path_unproven/u);
});

test('callback registration drift invalidates the source-bound part of the selected path', async () => {
  // Arrange
  const { auditBorrowedNoisePaths } = await import('./noise-native-readiness.mjs');
  // Act / Assert
  assert.throws(() => auditBorrowedNoisePaths(selectedFixture(), '.dispatch(other, job_completed)', boundTransport, boundBorrow), /noise_native_callback_binding/u);
});
