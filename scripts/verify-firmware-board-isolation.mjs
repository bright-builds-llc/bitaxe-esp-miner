#!/usr/bin/env node
// Gate: each board's firmware ELF carries only its own board facts (ADR-0041). The Ultra 205
// ELF names the 205 and holds no Gamma 601 safe-boot code; the 601 ELF names the 601, carries
// its safe boot and links none of the 205's sensor, fan, ASIC-initialization or safety owners.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

// Each literal was chosen as present in one board's ELF and absent from the other's, so none is
// vacuous.
// The 601 still links shared API and settings code that names the 205's parts; what it must not
// link are the owners that drive them, whose thread names and start lines are below.
export const BOARD_LITERALS = {
  '205': {
    required: ['firmware_board=205', 'operator-sensors', 'fan-controller'],
    // The record's literal, not the module name: debug info may name unlinked helpers.
    forbidden: ['firmware_board=601', 'usb_gamma601_safe_boot'],
  },
  '601': {
    required: ['firmware_board=601', 'usb_gamma601_safe_boot', 'mining_owner=absent'],
    forbidden: [
      'firmware_board=205',
      // The 205's sensor and fan owner threads, ASIC initialization and safety supervisor.
      'operator-sensors',
      'fan-controller',
      'operator_sensor_runtime=started',
      'core_voltage_adc=available',
      'asic_init',
      'safety_supervisor',
    ],
  },
};

/** Checks one ELF's bytes against its board's literals. */
export function checkBoard(board, bytes) {
  const literals = BOARD_LITERALS[board];
  if (!literals) throw Error('firmware_board_isolation_unknown_board');
  if (bytes.length === 0) throw Error('firmware_board_isolation_empty_elf');
  const missing = literals.required.filter((literal) => !bytes.includes(literal));
  const present = literals.forbidden.filter((literal) => bytes.includes(literal));
  return { board, isolated: missing.length === 0 && present.length === 0, missing, present };
}

export function main(args) {
  if (args.length !== 4 || args[0] !== '205' || args[2] !== '601') {
    throw Error('firmware_board_isolation_arguments');
  }
  const results = [checkBoard('205', readFileSync(args[1])), checkBoard('601', readFileSync(args[3]))];
  return { result: results.every((result) => result.isolated) ? 'boards_isolated' : 'board_literal_leak', results };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = main(process.argv.slice(2));
  console.log(JSON.stringify(result));
  if (result.result !== 'boards_isolated') process.exitCode = 1;
}
