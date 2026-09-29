"""Private captured fault provenance. Program bytes supply ABI/identity only, never receipts."""
import hashlib
import json
import os
from pathlib import Path
import struct
from cutoff import native_elf, verify as verify_cutoff

PANIC = 'BITAXE_PANIC_FRAME_RECORD'
ALLOCATION = 'BITAXE_ALLOCATION_HISTORY'
SOURCE = 'BITAXE_FAULT_COMPILED_SOURCE'


def symbol(program, name, size):
    values = [item for section in program.iter_sections() if section['sh_type'] == 'SHT_SYMTAB'
              for item in section.iter_symbols() if item.name == name]
    if len(values) != 1:
        raise ValueError('provenance_symbol')
    value = values[0]
    if value['st_info']['type'] != 'STT_OBJECT' or value['st_shndx'] == 'SHN_UNDEF' or value['st_size'] != size or value['st_value'] % 4:
        raise ValueError('provenance_symbol')
    return value


def captured(core, address, size):
    segments = [segment for segment in core.iter_segments() if segment['p_type'] == 'PT_LOAD'
                and segment['p_vaddr'] < address + size and segment['p_vaddr'] + segment['p_memsz'] > address]
    if len(segments) != 1:
        raise ValueError('provenance_mapping')
    segment = segments[0]
    offset = address - segment['p_vaddr']
    if offset < 0 or offset + size > segment['p_filesz']:
        raise ValueError('provenance_truncated')
    data = segment.data()[offset:offset + size]
    if len(data) != size:
        raise ValueError('provenance_truncated')
    return data


def words(data):
    return list(struct.unpack('<' + str(len(data) // 4) + 'I', data))


def compiled_words(program, name, size):
    value = symbol(program, name, size)
    section = program.get_section(value['st_shndx'])
    offset = value['st_value'] - section['sh_addr']
    if section['sh_type'] != 'SHT_PROGBITS' or offset < 0 or offset + size > section['sh_size']:
        raise ValueError('provenance_compiled_binding')
    return words(section.data()[offset:offset + size])


def fnv(values):
    result = 0x811c9dc5
    for byte in struct.pack('<' + str(len(values)) + 'I', *values):
        result = ((result ^ byte) * 0x01000193) & 0xffffffff
    return result


def rolling(values):
    result = 0x6d5a56a9
    for value in values:
        result = (((result << 5) | (result >> 27)) ^ value) & 0xffffffff
    return result


def executable(program, address):
    return any(segment['p_type'] == 'PT_LOAD' and segment['p_flags'] & 1 and
               segment['p_vaddr'] <= address < segment['p_vaddr'] + segment['p_filesz']
               for segment in program.iter_segments())


def validate_panic(record, source, abi):
    if len(record) != 48 or record[:3] != [0x50465231, 1, 48] or record[46] != fnv(record[1:46]) or record[47] != (record[46] ^ 0xffffffff):
        raise ValueError('provenance_panic_integrity')
    if record[4:6] != source or record[6:8] == [0, 0] or record[8] > 1 or any(value not in (0, 1) for value in record[44:46]):
        raise ValueError('provenance_panic_identity')
    if record[3] & ~63 or record[3] & 4 == 0 or record[3] & 32:
        raise ValueError('provenance_panic_flags')
    if record[40:43] != [65536, 36, 112] or any(record[i] not in (0, 1) for i in [16, 29, 31, 43]):
        raise ValueError('provenance_panic_abi')
    if record[30] & ~15 or record[34] != 0 or record[32] < 1 or not 1 <= record[33] <= 64 or record[3] & 8 == 0:
        raise ValueError('provenance_task_decision')
    if abi[:23] != [0x50464131, 1, 48, 36, 112, 12, 0, 4, 8, 12, 24, 28, 32, 0, 4, 8, 12, 16, 80, 84, 0, 4, 8] or abi[27] & ~3 or abi[28:] != [65536, 112, 0, 0x50465231]:
        raise ValueError('provenance_compiled_abi')
    if record[35:43] != [abi[27], *abi[23:27], abi[28], abi[3], abi[4]] or record[43] != int(record[15] == record[25]):
        raise ValueError('provenance_compiled_abi')
    start, end = record[25:27]
    rtc = lambda value: bool(abi[27] & 1) and abi[25] <= value < abi[26]
    start_sane = (abi[23] + 16 <= start <= abi[24] - 16 and start % 16 == 0) or rtc(start)
    end_sane = abi[23] <= end < abi[24] or rtc(end)
    external = lambda value: bool(abi[27] & 2) and 0x3c000000 <= value < 0x3e000000
    if (not external(start) and record[44] != int(start_sane)) or (not external(end) and record[45] != int(end_sane)):
        raise ValueError('provenance_sdk_predicates')
    rejected = int(not record[44]) | (int(not record[45]) << 1) | (int(start >= end) << 2) | (int((end - start) & 0xffffffff > abi[28]) << 3)
    if record[30] != rejected or record[31] != int(rejected != 0) or record[29] != 1:
        raise ValueError('provenance_sdk_predicates')
    return record


def validate_allocations(data, source, boot):
    if len(data) != 384 or data[:7] != [0x42414c48, 1, 16, 20, 184, 2, 8] or data[12] != 0xa110ca7e or data[15] != rolling(data[:11]):
        raise ValueError('provenance_allocation_header')
    if data[7:9] != source or data[9:11] != boot or data[13] != 0 or data[14] != 0:
        raise ValueError('provenance_allocation_identity')
    records = []
    for core in range(2):
        offset = 16 + core * 184
        seq, flags, overwritten, reserved = data[offset:offset + 4]
        if flags & ~1 or reserved > 1 or flags & 1 and seq != 0xffffffff or overwritten != max(0, seq - 8):
            raise ValueError('provenance_allocation_count')
        if 0 < seq <= 8 and data[offset + 4:offset + 24] != data[offset + 24:offset + 44]:
            raise ValueError('provenance_allocation_first_join')
        for index in range(9):
            record = data[offset + 4 + index * 20:offset + 24 + index * 20]
            if seq == 0 or index > 0 and seq < index:
                if any(record):
                    raise ValueError('provenance_allocation_uncommitted')
                continue
            expected = 1 if index == 0 else seq - ((seq - index) % 8)
            if record[0] != expected or record[2] != expected or record[1] != 0x414c4c31 or record[3] != core or record[19] != rolling(record[1:19]):
                raise ValueError('provenance_allocation_integrity')
            if record[4:6] != source or record[18] & ~15 or record[15] > 9 or (record[6:8] != boot and not (record[6:8] == [0, 0] and record[18] & 2)):
                raise ValueError('provenance_allocation_identity')
            if bool(record[18] & 2) != (record[6:8] == [0, 0]) or bool(record[18] & 4) != bool(record[18] & 1 and record[16] == 0xffffffff):
                raise ValueError('provenance_allocation_flags')
            if record[18] & 8 and record[18] & 1 or (not record[18] & 1 and any(record[14:18])) or (record[18] & 1 and (record[11] == 0 or record[14] == 0)):
                raise ValueError('provenance_allocation_owner')
            records.append(record)
    return records


def note_fields(core):
    notes = [note for segment in core.iter_segments() if segment['p_type'] == 'PT_NOTE' for note in segment.iter_notes()]
    registers, tasks, extra = [], [], []
    for note in notes:
        kind = note['n_type']
        data = note['n_desc']
        if not isinstance(data, bytes):
            # pyelftools decodes standard NT_PRSTATUS to raw bytes on Xtensa; reject other architectures/representations.
            if kind in (1, 'NT_PRSTATUS'):
                raise ValueError('provenance_register_note')
            continue
        if kind in (1, 'NT_PRSTATUS') and note['n_name'] == 'CORE':
            if len(data) != 588:
                raise ValueError('provenance_register_note')
            registers.append((struct.unpack_from('<I', data, 24)[0], struct.unpack_from('<I', data, 72)[0]))
        elif kind == 678:
            if len(data) != 36:
                raise ValueError('provenance_task_note')
            tasks.append(words(data[:20]))
        elif kind == 677:
            if len(data) < 4:
                raise ValueError('provenance_extra_note')
            extra.append(struct.unpack_from('<I', data)[0])
    return registers, tasks, extra


def private_json(path, value):
    with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), 'w', encoding='utf-8') as stream:
        json.dump(value, stream, sort_keys=True)


def verify(program_path, core_path, output, expected_elf):
    if hashlib.sha256(Path(program_path).read_bytes()).hexdigest() != expected_elf:
        raise ValueError('provenance_elf_identity')
    with open(program_path, 'rb') as p, open(core_path, 'rb') as c:
        program, core = native_elf(p), native_elf(c)
        if program['e_type'] != 'ET_EXEC' or core['e_type'] != 'ET_CORE':
            raise ValueError('provenance_elf_type')
        raw = {'schema': 'bitaxe-private-fault-provenance-raw/1', 'elf_sha256': expected_elf, 'verified': False, 'cause_proven': False}
        mapping_errors = []
        for key, name, size in [('panic_words', PANIC, 192), ('allocation_words', ALLOCATION, 1536)]:
            try:
                raw[key] = words(captured(core, symbol(program, name, size)['st_value'], size))
            except ValueError:
                mapping_errors.append(key)
        raw['unavailable_regions'] = mapping_errors
        private_json(Path(output) / 'provenance-raw.private.json', raw)
        if mapping_errors:
            raise ValueError('provenance_capture_regions')
        source = compiled_words(program, SOURCE, 8)
        abi = compiled_words(program, 'BITAXE_FAULT_PROVENANCE_ABI', 128)
        panic = validate_panic(raw['panic_words'], source, abi)
        allocation_words = raw['allocation_words']
        allocations = validate_allocations(allocation_words, source, panic[6:8])
        registers, tasks, extra = note_fields(core)
        matched = [row for row in registers if row[0] == panic[24]]
        task = [row for row in tasks if row[2] == panic[24]]
        if len(matched) != 1 or len(task) > 1 or extra != [panic[24]]:
            raise ValueError('provenance_crashed_task_join')
        fake = matched[0][1] == 0x20000000
        if fake != bool(panic[31]) or panic[28] <= panic[27] or panic[28] - panic[27] > 65536:
            raise ValueError('provenance_sdk_decision_join')
        captured_stack = captured(core, panic[27], panic[28] - panic[27])
        if len(captured_stack) < 112 or struct.unpack_from('<I', captured_stack, 4)[0] != matched[0][1]:
            raise ValueError('provenance_sdk_frame_join')
        if not fake and panic[27] == panic[15] and matched[0][1] != panic[18]:
            raise ValueError('provenance_original_frame_join')
        if task and (task[0][3] != panic[27] or task[0][4] != panic[28] - panic[27]):
            raise ValueError('provenance_sdk_decision_join')
        meaningful = (panic[3] & 23 == 23 and panic[15] % 4 == 0 and panic[21] % 16 == 0 and
                      panic[36] <= panic[15] <= panic[37] - 112 and panic[36] <= panic[21] < panic[37] and executable(program, panic[18]))
        if not meaningful:
            raise ValueError('provenance_original_frame_unavailable')
        cutoff = verify_cutoff(program_path, core_path)
        private = {'schema': 'bitaxe-private-fault-provenance/1', 'elf_sha256': expected_elf,
                   'panic_words': panic, 'abi_words': abi, 'allocation_words': allocation_words, 'allocation_records': allocations, 'sdk_fake_crashed_frame': fake, 'cause_proven': False}
        private_json(Path(output) / 'provenance.private.json', private)
        return {'fault_provenance_verified': True, 'original_frame_meaningful': meaningful, 'sdk_fake_crashed_frame': fake,
                'allocation_records_verified': len({(row[3], row[2]) for row in allocations}),
                'allocation_early_unbound_records': len({(row[3], row[2]) for row in allocations if row[18] & 2}),
                'allocation_history_overwritten': any(allocation_words[18 + core * 184] > 0 for core in range(2)),
                'allocation_registry_full': any(allocation_words[19 + core * 184] != 0 for core in range(2)), 'self_test_provenance_qualified': cutoff['self_test_marked'],
                'provenance_verifier_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
