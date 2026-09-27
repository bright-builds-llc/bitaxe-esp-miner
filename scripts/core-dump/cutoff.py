"""Read the native cutoff receipt from captured PT_LOAD bytes, never program BSS."""
import hashlib
from pathlib import Path
import struct

from elftools.elf.elffile import ELFFile

SYMBOL = 'BITAXE_PANIC_CUTOFF_RECEIPT'
WORDS = 7
MAGIC = 0x50434F32
SELF_TEST = 0x53544631
OUTPUT_MASK = (1 << 10) | (1 << 1)
ENABLE_MASK = 1 << 10
REVOKED = 4
FLAGS = 7


def native_elf(stream):
    elf = ELFFile(stream)
    if elf.elfclass != 32 or not elf.little_endian or elf['e_machine'] != 'EM_XTENSA':
        raise ValueError('cutoff_elf_format')
    return elf


def symbol_address(program):
    symbols = [symbol for section in program.iter_sections()
               if section['sh_type'] == 'SHT_SYMTAB'
               for symbol in section.iter_symbols() if symbol.name == SYMBOL]
    if len(symbols) != 1:
        raise ValueError('cutoff_symbol')
    symbol = symbols[0]
    address = int(symbol['st_value'])
    if (symbol['st_info']['type'] != 'STT_OBJECT' or symbol['st_shndx'] == 'SHN_UNDEF'
            or symbol['st_size'] != WORDS * 4 or address % 4
            or not 0x3FC88000 <= address < 0x3FD00000 - WORDS * 4):
        raise ValueError('cutoff_symbol')
    return address


def captured_receipt(core, address):
    end = address + WORDS * 4
    # Reject any ambiguous or partly missing mapping. In particular, p_memsz
    # cannot authorize synthesizing missing receipt bytes as zero-filled BSS.
    segments = [segment for segment in core.iter_segments()
                if segment['p_type'] == 'PT_LOAD'
                and int(segment['p_vaddr']) < end
                and int(segment['p_vaddr']) + int(segment['p_memsz']) > address]
    if len(segments) != 1:
        raise ValueError('cutoff_mapping')
    segment = segments[0]
    offset = address - int(segment['p_vaddr'])
    if offset < 0 or offset + WORDS * 4 > int(segment['p_filesz']):
        raise ValueError('cutoff_truncated')
    data = segment.data()[offset:offset + WORDS * 4]
    if len(data) != WORDS * 4:
        raise ValueError('cutoff_truncated')
    return struct.unpack('<7I', data)


def validate(words):
    magic, configured, enabled, output, generation, state, marker = words
    if magic != MAGIC:
        raise ValueError('cutoff_magic')
    if configured != OUTPUT_MASK or enabled & OUTPUT_MASK != OUTPUT_MASK or output & OUTPUT_MASK != ENABLE_MASK:
        raise ValueError('cutoff_unsafe_outputs')
    if generation == 0 or state & FLAGS != REVOKED or state >> 3 != generation:
        raise ValueError('cutoff_unrevoked')
    if marker not in (0, SELF_TEST):
        raise ValueError('cutoff_self_test_marker')
    return {'native_cutoff_verified': True, 'captured_memory_verified': True,
            'asic_outputs_disabled': True, 'generation_revoked': True,
            'self_test_marked': marker == SELF_TEST,
            'cutoff_verifier_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}


def verify(program_path, core_path):
    with open(program_path, 'rb') as program_file, open(core_path, 'rb') as core_file:
        program = native_elf(program_file)
        core = native_elf(core_file)
        if program['e_type'] != 'ET_EXEC' or core['e_type'] != 'ET_CORE':
            raise ValueError('cutoff_elf_type')
        return validate(captured_receipt(core, symbol_address(program)))
