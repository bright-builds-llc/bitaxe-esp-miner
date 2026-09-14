export const MAIN_STACK_BUDGET_BYTES = 16384;
const STAGES = [
  /^bitaxe_firmware::main$/,
  /^bitaxe_firmware::http_api::cadence_owner::PreparedHttpRuntime::run$/,
  /^bitaxe_firmware::http_api::websocket::live_telemetry_cadence_loop$/,
  /^bitaxe_worker_control::cadence::run_iteration$/,
  /^<bitaxe_firmware::http_api::websocket::TelemetryIteration as bitaxe_worker_control::cadence::CadenceLoopIo>::live$/,
  /^bitaxe_firmware::runtime_snapshot::publish_projected_live_telemetry_payload_profiled$/,
  /^bitaxe_api::runtime_projection::project_api_views$/,
  /^bitaxe_api::wire::SystemInfoWire::from_snapshot$/,
  /^bitaxe_api::mining::mining_state_from_runtime$/,
  /^<alloc::vec::.*SpecFromIterNested.*>::from_iter$/,
];
const COMPLETION = /^bitaxe_firmware::operator_snapshot_publication::OperatorSnapshotPublisher::publish_profiled::\{\{closure\}\}$/;
const check = (condition, code) => { if (!condition) throw Error(code); };

function functions(text) {
  const result = new Map(); let current;
  for (const line of text.split('\n')) {
    const header = /^([0-9a-f]+) <(.+)>:\s*$/.exec(line);
    if (header) {
      const address = parseInt(header[1], 16);
      check(!result.has(address), 'telemetry_duplicate_function');
      current = { address, symbol: header[2], instructions: [] }; result.set(address, current); continue;
    }
    const instruction = /^\s*([0-9a-f]+):\s+[0-9a-f]+\s+([a-z][a-z0-9_.]*)\s*(.*)$/.exec(line);
    if (current && instruction) current.instructions.push({ address: parseInt(instruction[1], 16), op: instruction[2], args: instruction[3] });
  }
  return result;
}
function destination(instruction) {
  if (/^(?:s8i|s16i|s32i|ssi|ssx|s16i|s32i\.n|b\w*|j|jx|call\w*|ret\w*|nop\w*|loop\w*|memw|isync|rsync|esync|dsync|waiti|ill|break\w*)$/.test(instruction.op)) return null;
  const register = /^a(\d+)(?:,|$)/.exec(instruction.args);
  return register ? Number(register[1]) : null;
}
const branchTarget = instruction => {
  const match = /(?:^|,\s*)([0-9a-f]+)\s+</.exec(instruction.args);
  return match ? parseInt(match[1], 16) : null;
};

/** Conservative constant-register dataflow proves literal-loaded call targets across branches. */
function calls(fn) {
  const instructions = fn.instructions, positions = new Map(instructions.map((value, index) => [value.address, index]));
  const states = new Map(), queue = [], edges = new Map(), loopBack = new Map();
  if (!instructions.length) return [];
  states.set(0, Array(16).fill(null)); queue.push(0);
  for (const [index, instruction] of instructions.entries()) {
    if (/^loop(?:nez|gtz)?$/.test(instruction.op)) {
      const end = positions.get(branchTarget(instruction));
      check(end !== undefined && end > index + 1, 'telemetry_loop_unknown');
      loopBack.set(end - 1, index + 1);
    }
  }
  let visits = 0;
  while (queue.length) {
    check(++visits < 1000000, 'telemetry_dataflow_bound');
    const index = queue.shift(), instruction = instructions[index], incoming = states.get(index), outgoing = [...incoming];
    const dest = destination(instruction);
    if (dest !== null) outgoing[dest] = null;
    if (instruction.op === 'l32r') {
      const literal = /^a(\d+),.*\(([0-9a-f]+) <.+>\)\s*$/.exec(instruction.args);
      if (literal) outgoing[Number(literal[1])] = parseInt(literal[2], 16);
    } else if (/^mov(?:\.n)?$/.test(instruction.op)) {
      const move = /^a(\d+),\s*a(\d+)$/.exec(instruction.args);
      if (move) outgoing[Number(move[1])] = incoming[Number(move[2])];
    }
    if (/^callx?(?:0|4|8|12)$/.test(instruction.op)) {
      const register = /^a(\d+)$/.exec(instruction.args);
      const target = instruction.op.startsWith('callx') ? (register ? incoming[Number(register[1])] : null) : branchTarget(instruction);
      if (target !== null) edges.set(instruction.address, { target, call_address: instruction.address });
      else edges.delete(instruction.address);
      // Windowed call8 preserves caller a2-a7; other call conventions are not assumed.
      for (let r = instruction.op.endsWith('8') ? 8 : 0; r < 16; r++) if (r !== 1) outgoing[r] = null;
    }
    const successors = [];
    if (!/^(?:ret\w*(?:\.n)?|jx|ill|break\w*)$/.test(instruction.op)) {
      if (instruction.op !== 'j' && index + 1 < instructions.length) successors.push(index + 1);
      if (/^(?:j|b\w*|loop(?:nez|gtz)?)$/.test(instruction.op)) {
        const target = positions.get(branchTarget(instruction));
        if (target !== undefined) successors.push(target);
      }
      if (loopBack.has(index)) successors.push(loopBack.get(index));
    }
    for (const next of successors) {
      const prior = states.get(next), merged = prior ? prior.map((value, r) => value === outgoing[r] ? value : null) : [...outgoing];
      if (!prior || merged.some((value, r) => value !== prior[r])) { states.set(next, merged); queue.push(next); }
    }
  }
  return [...edges.values()];
}
function frame(fn) {
  const entries = fn.instructions.filter(value => value.op === 'entry');
  check(entries.length === 1 && entries[0] === fn.instructions[0], 'telemetry_frame_missing_or_multiple');
  const match = /^a1,\s*(0x[0-9a-f]+|\d+)$/.exec(entries[0].args);
  const bytes = match ? Number(match[1]) : NaN;
  check(Number.isSafeInteger(bytes) && bytes >= 32 && bytes % 16 === 0, 'telemetry_frame_unknown');
  check(!fn.instructions.some(value => value.op !== 'entry' && destination(value) === 1), 'telemetry_dynamic_stack_unknown');
  return bytes;
}

/** Audits only the named projection path; passing is not a whole-task stack bound. */
export function auditTelemetryStack(disassembly, sdkconfig) {
  const assignments = sdkconfig.split(/\r?\n/).filter(line => line.startsWith('CONFIG_ESP_MAIN_TASK_STACK_SIZE='));
  check(assignments.length === 1 && assignments[0] === `CONFIG_ESP_MAIN_TASK_STACK_SIZE=${MAIN_STACK_BUDGET_BYTES}`, 'telemetry_stack_contract');
  const symbols = functions(disassembly), callCache = new Map();
  const outgoing = fn => { if (!callCache.has(fn.address)) callCache.set(fn.address, calls(fn)); return callCache.get(fn.address); };
  const roots = [...symbols.values()].filter(fn => STAGES[0].test(fn.symbol));
  check(roots.length === 1, 'telemetry_main_missing_or_multiple');
  let paths = [{ nodes: [roots[0]], edges: [] }];
  for (let stage = 1; stage < STAGES.length; stage++) {
    const next = [];
    for (const path of paths) {
      const caller = path.nodes.at(-1);
      for (const edge of outgoing(caller)) {
        const target = symbols.get(edge.target); if (!target) continue;
        if (STAGES[stage].test(target.symbol)) next.push({ nodes: [...path.nodes, target], edges: [...path.edges, edge] });
        if (stage === 6 && COMPLETION.test(target.symbol)) {
          for (const inner of outgoing(target)) {
            const projected = symbols.get(inner.target);
            if (projected && STAGES[stage].test(projected.symbol)) next.push({ nodes: [...path.nodes, target, projected], edges: [...path.edges, edge, inner] });
          }
        }
      }
    }
    check(next.length > 0 && next.length <= 128, `telemetry_path_unknown_stage_${stage}`); paths = next;
  }
  const candidates = paths.map(path => ({ ...path, frames: path.nodes.map(frame) }));
  const worst = candidates.reduce((best, path) => path.frames.reduce((a,b)=>a+b,0) > best.frames.reduce((a,b)=>a+b,0) ? path : best);
  const total = worst.frames.reduce((a,b)=>a+b,0);
  return { schema: 'telemetry-stack-audit-v1', result: total <= MAIN_STACK_BUDGET_BYTES ? 'targeted_path_fits' : 'budget_exceeded',
    main_stack_budget_bytes: MAIN_STACK_BUDGET_BYTES, targeted_path_bytes: total, matched_paths: candidates.length,
    complete_callgraph_bound: false, hardware_safety_verified: false,
    nodes: worst.nodes.map((fn,index)=>({ symbol: fn.symbol, address: fn.address.toString(16), entry_bytes: worst.frames[index] })),
    edges: worst.edges.map(edge=>({ call_address: edge.call_address.toString(16), target: edge.target.toString(16) })) };
}
