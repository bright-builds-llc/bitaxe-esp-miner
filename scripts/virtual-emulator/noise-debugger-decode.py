"""Decode bounded private debugger snapshots; absence never establishes safety."""
import json
import os
from pathlib import Path
import re
import struct
import sys

ABI = {'tcb_bytes': 340, 'stack_offset': 48, 'end_offset': 72, 'top_offset': 0,
       'name_offset': 52, 'panic_bytes': 36, 'frame_offset': 28,
       'exception_bytes': 112, 'journal_bytes': 768, 'count_bytes': 4, 'current_bytes': 8,
       'reached_bytes': 4, 'released_bytes': 4}
CUTOFFS = [101, 102, 103, 105, 106, 107, 109, 110]


def maybe_bytes(root, name, size):
    candidate = root / name
    if not candidate.exists():
        return None
    if candidate.is_symlink() or candidate.stat().st_mode & 0o777 != 0o600:
        raise ValueError('debugger_private_file')
    data = candidate.read_bytes()
    if len(data) != size:
        raise ValueError('debugger_snapshot_size')
    return data


def checkpoints(raw, count):
    result = []
    if count > 16:
        raise ValueError('debugger_checkpoint_count')
    previous = 100
    for index in range(count):
        words = struct.unpack_from('<12I', raw, index * 48)
        magic, phase, stage, integrity, core, base, span, pointer, free, largest, water, inside = words
        if (magic != 0x564e5031 or not 101 <= phase <= 114 or phase <= previous
                or stage not in (1, 2) or core not in (0, 1) or span != 16384
                or inside not in (0, 1) or integrity not in (0, 1)):
            raise ValueError('debugger_checkpoint_format')
        if stage == 2 and largest > free:
            raise ValueError('debugger_heap_metrics')
        actual_inside = base <= pointer < base + span
        if bool(inside) != actual_inside:
            raise ValueError('debugger_checkpoint_pointer')
        result.append({'phase': phase, 'stage': stage,
                       'heap_integrity': bool(integrity) if stage == 2 else None,
                       'heap_observation_available': stage == 2,
                       'internal_free_bytes': free if stage == 2 else None,
                       'internal_largest_bytes': largest if stage == 2 else None,
                       'stack_low_water_bytes': water, 'configured_stack_span_bytes': span,
                       'stack_pointer_inside_configured_span': actual_inside})
        previous = phase
    if any(raw[count * 48:]):
        raise ValueError('debugger_checkpoint_unpublished')
    return result


def task_facts(raw, captured_sp):
    base, end, top = [struct.unpack_from('<I', raw, offset)[0] for offset in (48, 72, 0)]
    name = raw[52:68].split(b'\0')[0]
    if not 0x3fc88000 <= base <= end < 0x3fd00000 or end - base > 65536:
        return {'task_label': 'unknown', 'bounds_available': False, 'reason': 'invalid_tcb_bounds'}
    label = name.decode('ascii', errors='replace')
    return {'task_label': label if label in ('main', 'IDLE0', 'IDLE1') else 'other',
            'bounds_available': True, 'bound_distance_bytes': end - base,
            'saved_tcb_top_inside_bounds': base <= top <= end,
            'captured_sp_inside_bounds': base <= captured_sp <= end if captured_sp is not None else None,
            'captured_sp_below_base_bytes': max(0, base - captured_sp) if captured_sp is not None else None}


def decode(root, stdout, cutoff):
    if cutoff not in CUTOFFS or len(stdout.encode()) > 2 * 1024 * 1024:
        raise ValueError('debugger_arguments')
    preflight = json.loads((root / 'gdb-preflight.json').read_text())
    if preflight.get('abi') != ABI or preflight.get('gdb_version') != '16.3_20250913':
        raise ValueError('debugger_abi_binding')
    snapshots = []
    for index in range(2):
        values = {}
        for field in ('KIND', 'PC', 'SP', 'ARG'):
            matches = re.findall(r'^SNAPSHOT_' + str(index) + '_' + field + r' ([0-9]+)$', stdout, re.M)
            if len(matches) == 1:
                values[field] = int(matches[0])
            elif len(matches) > 1:
                raise ValueError('debugger_snapshot_duplicate')
        raw = maybe_bytes(root, f'snapshot{index}-journal.raw', 768)
        counter = maybe_bytes(root, f'snapshot{index}-count.raw', 4)
        if not values and raw is None and counter is None:
            continue
        kind = {1: 'panic', 2: 'assert', 3: 'prefix', 4: 'released'}.get(values.get('KIND'), 'unknown')
        records = checkpoints(raw, struct.unpack('<I', counter)[0]) if raw is not None and counter is not None else None
        original_sp = None
        cause = None
        frame = maybe_bytes(root, f'snapshot{index}-exception-frame.raw', 112)
        panic = maybe_bytes(root, f'snapshot{index}-panic-info.raw', 36)
        if kind == 'panic' and frame is not None and panic is not None and struct.unpack_from('<I', panic, 4)[0] == 4:
            original_sp = struct.unpack_from('<I', frame, 16)[0]
            code = struct.unpack_from('<I', frame, 80)[0]
            cause = {15: 'LoadStorePIFAddrError', 28: 'LoadProhibited', 29: 'StoreProhibited'}.get(code, 'other_exception')
        tasks = []
        for slot in range(2):
            tcb = maybe_bytes(root, f'snapshot{index}-tcb{slot}.raw', 340)
            if tcb is not None:
                tasks.append(task_facts(tcb, original_sp if original_sp is not None else values.get('SP')))
        scope_violation = any(record['phase'] > cutoff and record['phase'] != 114 for record in records or [])
        reached_raw = maybe_bytes(root, f'snapshot{index}-prefix-reached.raw', 4)
        released_raw = maybe_bytes(root, f'snapshot{index}-prefix-released.raw', 4)
        reached = struct.unpack('<I', reached_raw)[0] if reached_raw is not None else None
        released = struct.unpack('<I', released_raw)[0] if released_raw is not None else None
        if reached is not None and reached not in [0, *CUTOFFS] or released is not None and released not in (0, 1):
            raise ValueError('debugger_prefix_flags')
        snapshots.append({'index': index, 'kind': kind, 'registers_available': len(values) == 4,
                          'records': records, 'checkpoint_memory_available': records is not None,
                          'task_bounds': tasks if tasks else None, 'task_bounds_available': bool(tasks),
                          'original_exception_frame_available': original_sp is not None,
                          'exception_category': cause, 'scope_violation': scope_violation,
                          'prefix_argument_matches': values.get('ARG') == cutoff if kind in ('prefix', 'released') else None,
                          'prefix_reached_phase': reached, 'prefix_released_flag': released,
                          'prefix_flags_available': reached is not None and released is not None,
                          'debugger_paused_time': True})
    return {'schema': 'bitaxe-noise-live-snapshots-v1', 'cutoff_phase': cutoff, 'snapshots': snapshots,
            'snapshot_count': len(snapshots), 'snapshot_memory_available': bool(snapshots),
            'unknown_reason': None if snapshots else 'no_snapshot_captured',
            'full_noise_qualified': False, 'hardware_qualified': False}


if __name__ == '__main__':
    os.umask(0o077)
    root, log, cutoff, output = sys.argv[1:]
    result = decode(Path(root), Path(log).read_text(), int(cutoff))
    with open(output, 'x', encoding='utf-8') as target:
        json.dump(result, target, sort_keys=True)
