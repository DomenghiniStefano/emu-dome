"""Builds emudome-test.nds from test.s. Needs only Python 3 (no ARM toolchain).

A tiny ARM (ARMv4/v5, ARM state) assembler for the subset test.s uses, then an NDS header:
ARM9 and ARM7 programs, no file system, no banner, no logo (the header's logo area and logo CRC
stay zero: a homebrew game is booted directly and not checked). The header CRC is filled in, so
the library reads the file as a valid NDS game.

    python3 build.py
"""

import pathlib
import re
import struct

HERE = pathlib.Path(__file__).parent
OUT = HERE / "emudome-test.nds"

CONDS = {"eq": 0, "ne": 1, "cs": 2, "cc": 3, "mi": 4, "pl": 5, "vs": 6, "vc": 7,
         "hi": 8, "ls": 9, "ge": 10, "lt": 11, "gt": 12, "le": 13, "": 14}
DP = {"and": 0, "eor": 1, "sub": 2, "rsb": 3, "add": 4, "tst": 8, "cmp": 10,
      "orr": 12, "mov": 13, "bic": 14, "mvn": 15}
MEM = ("ldrh", "strh", "ldrb", "strb", "ldr", "str")
SHIFTS = {"lsl": 0, "lsr": 1, "asr": 2, "ror": 3}
BASES = sorted(list(DP) + list(MEM) + ["bl", "b"], key=len, reverse=True)


def reg(text):
    text = text.strip().lower()
    named = {"sp": 13, "lr": 14, "pc": 15}
    if text in named:
        return named[text]
    if not re.fullmatch(r"r(1[0-5]|[0-9])", text):
        raise ValueError(f"not a register: {text}")
    return int(text[1:])


def number(text):
    return int(text.strip().lstrip("#"), 0)


def rotated(value):
    """The 12-bit immediate operand for `value` (8 bits rotated right by an even amount)."""
    value &= 0xFFFFFFFF
    for rot in range(16):
        v = ((value << (2 * rot)) | (value >> (32 - 2 * rot))) & 0xFFFFFFFF if rot else value
        if v < 256:
            return (rot << 8) | v
    raise ValueError(f"immediate not encodable: {value:#x}")


def split_mnemonic(word):
    for base in BASES:
        if word.startswith(base) and word[len(base):] in CONDS:
            return base, CONDS[word[len(base):]]
    raise ValueError(f"unknown instruction: {word}")


def operand2(parts):
    """`#imm`, `Rm`, or `Rm, lsl #n`: (I bit, 12 bits)."""
    if parts[0].startswith("#"):
        return 1, rotated(number(parts[0]))
    rm = reg(parts[0])
    if len(parts) == 1:
        return 0, rm
    kind, amount = parts[1].split()
    return 0, ((number(amount) & 31) << 7) | (SHIFTS[kind.lower()] << 5) | rm


class Program:
    def __init__(self, name, base):
        self.name, self.base = name, base
        self.lines = []  # (mnemonic, operands text, source line)
        self.labels = {}
        self.pool = []  # literal values, placed after the code

    def size(self):
        return 4 * (len(self.lines) + len(self.pool))

    def add(self, line):
        if line.endswith(":"):
            self.labels[line[:-1]] = 4 * len(self.lines)
            return
        word, _, rest = line.partition(" ")
        self.lines.append((word.lower(), rest.strip(), line))
        if "=" in rest:
            value = number(rest.split("=", 1)[1])
            if value not in self.pool:
                self.pool.append(value)

    def assemble(self):
        code_end = 4 * len(self.lines)
        out = bytearray()
        for index, (word, rest, line) in enumerate(self.lines):
            at = 4 * index
            try:
                out += struct.pack("<I", self.encode(word, rest, at, code_end))
            except (ValueError, KeyError) as e:
                raise SystemExit(f"{self.name}: {line}: {e}")
        for value in self.pool:
            out += struct.pack("<I", value & 0xFFFFFFFF)
        return bytes(out)

    def encode(self, word, rest, at, code_end):
        base, cond = split_mnemonic(word)
        c = cond << 28
        if base in ("b", "bl"):
            offset = (self.labels[rest] - (at + 8)) >> 2
            return c | 0x0A000000 | ((base == "bl") << 24) | (offset & 0xFFFFFF)
        if base in MEM:
            rd_text, addr = rest.split(",", 1)
            rd, addr = reg(rd_text), addr.strip()
            load = base.startswith("ldr")
            if addr.startswith("="):
                if base != "ldr":
                    raise ValueError("only ldr takes =constant")
                literal = code_end + 4 * self.pool.index(number(addr[1:]))
                rn, offset = 15, literal - (at + 8)
            else:
                inner = addr.strip("[]").split(",")
                rn = reg(inner[0])
                offset = number(inner[1]) if len(inner) > 1 else 0
            up = offset >= 0
            offset = abs(offset)
            if base in ("ldrh", "strh"):
                if offset > 0xFF:
                    raise ValueError("halfword offset over 255")
                return (c | 0x014000B0 | (up << 23) | (load << 20) | (rn << 16) | (rd << 12)
                        | ((offset >> 4) << 8) | (offset & 0xF))
            if offset > 0xFFF:
                raise ValueError("offset over 4095")
            byte = base.endswith("b")
            return (c | 0x05000000 | (up << 23) | (byte << 22) | (load << 20) | (rn << 16)
                    | (rd << 12) | offset)
        parts = [p.strip() for p in rest.split(",")]
        op = DP[base]
        if base in ("mov", "mvn"):
            rd, rn, op2 = reg(parts[0]), 0, parts[1:]
        elif base in ("tst", "cmp"):
            rd, rn, op2 = 0, reg(parts[0]), parts[1:]
        else:
            rd, rn, op2 = reg(parts[0]), reg(parts[1]), parts[2:]
        imm, bits = operand2(op2)
        s = base in ("tst", "cmp")
        return c | (imm << 25) | (op << 21) | (s << 20) | (rn << 16) | (rd << 12) | bits


def parse(source):
    programs, current = {}, None
    for raw in source.splitlines():
        line = raw.split("@", 1)[0].strip()
        if not line:
            continue
        if line.startswith(".cpu"):
            _, name, base = line.split()
            current = programs[name] = Program(name, int(base, 0))
            continue
        current.add(line)
    return programs


def crc16(data):
    """CRC-16/MODBUS, as the NDS header uses it."""
    crc = 0xFFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ 0xA001 if crc & 1 else crc >> 1
    return crc


def align(n, to):
    return (n + to - 1) // to * to


def build():
    programs = parse((HERE / "test.s").read_text())
    arm9, arm7 = programs["arm9"].assemble(), programs["arm7"].assemble()
    # ARM9 after the 4 KiB header area (melonDS reads 0x1000 bytes of header): an ARM9 offset
    # under 0x4000 marks a homebrew game, which melonDS boots without the checks a retail card
    # gets (secure area, logo CRC), so the header needs nothing taken from a commercial card.
    arm9_at = 0x1000
    arm7_at = align(arm9_at + len(arm9), 0x200)
    used = arm7_at + len(arm7)
    rom = bytearray(align(used, 0x200))
    rom[arm9_at:arm9_at + len(arm9)] = arm9
    rom[arm7_at:arm7_at + len(arm7)] = arm7

    h = bytearray(0x200)
    h[0x00:0x0C] = b"EMUDOMETEST\0"
    h[0x0C:0x10] = b"EDTS"  # game code
    h[0x10:0x12] = b"00"  # maker code
    h[0x14] = 0  # capacity: 128 KiB
    struct.pack_into("<4I", h, 0x20, arm9_at, programs["arm9"].base, programs["arm9"].base,
                     len(arm9))
    struct.pack_into("<4I", h, 0x30, arm7_at, programs["arm7"].base, programs["arm7"].base,
                     len(arm7))
    struct.pack_into("<2I", h, 0x60, 0x00586000, 0x001808F8)  # cartridge bus timings
    struct.pack_into("<2I", h, 0x80, used, 0x4000)  # used size, header size
    struct.pack_into("<H", h, 0x15E, crc16(h[:0x15E]))
    rom[:0x200] = h
    OUT.write_bytes(rom)
    print(OUT, len(rom), "bytes; ARM9", len(arm9), "bytes, ARM7", len(arm7), "bytes")


if __name__ == "__main__":
    build()
