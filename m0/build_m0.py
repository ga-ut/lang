#!/usr/bin/env python3
"""One-time direct AArch64 constructor for the self-hosting M0 assembler."""

from __future__ import annotations

import hashlib
import os
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
LOAD = 0x400000
HEADER_SIZE = 120
FILE_SIZE = 4096
CODE_CAPACITY = FILE_SIZE - HEADER_SIZE


class Code:
    def __init__(self) -> None:
        self.words: list[int] = []
        self.labels: dict[str, int] = {}
        self.fixups: list[tuple[int, str, str, int]] = []

    def emit(self, word: int) -> None:
        self.words.append(word & 0xFFFFFFFF)

    def label(self, name: str) -> None:
        self.labels[name] = len(self.words)

    def movz(self, rd: int, imm: int, bits: int = 64, shift: int = 0) -> None:
        base = 0xD2800000 if bits == 64 else 0x52800000
        self.emit(base | ((shift // 16) << 21) | ((imm & 0xFFFF) << 5) | rd)

    def movk(self, rd: int, imm: int, bits: int = 64, shift: int = 16) -> None:
        base = 0xF2800000 if bits == 64 else 0x72800000
        self.emit(base | ((shift // 16) << 21) | ((imm & 0xFFFF) << 5) | rd)

    def mov_reg(self, rd: int, rm: int, bits: int = 64) -> None:
        self.emit((0xAA0003E0 if bits == 64 else 0x2A0003E0) | (rm << 16) | rd)

    def add_imm(self, rd: int, rn: int, imm: int, shift12: bool = False) -> None:
        self.emit(0x91000000 | (int(shift12) << 22) | ((imm & 0xFFF) << 10) | (rn << 5) | rd)

    def sub_imm(self, rd: int, rn: int, imm: int, shift12: bool = False) -> None:
        self.emit(0xD1000000 | (int(shift12) << 22) | ((imm & 0xFFF) << 10) | (rn << 5) | rd)

    def add_reg(self, rd: int, rn: int, rm: int, bits: int = 64) -> None:
        self.emit((0x8B000000 if bits == 64 else 0x0B000000) | (rm << 16) | (rn << 5) | rd)

    def sub_reg(self, rd: int, rn: int, rm: int) -> None:
        self.emit(0xCB000000 | (rm << 16) | (rn << 5) | rd)

    def orr_reg(self, rd: int, rn: int, rm: int, bits: int = 32) -> None:
        self.emit((0xAA000000 if bits == 64 else 0x2A000000) | (rm << 16) | (rn << 5) | rd)

    def cmp_imm(self, rn: int, imm: int, bits: int = 32) -> None:
        self.emit((0xF100001F if bits == 64 else 0x7100001F) | ((imm & 0xFFF) << 10) | (rn << 5))

    def cmp_reg(self, rn: int, rm: int, bits: int = 64) -> None:
        self.emit((0xEB00001F if bits == 64 else 0x6B00001F) | (rm << 16) | (rn << 5))

    def ldrb(self, rt: int, rn: int) -> None:
        self.emit(0x39400000 | (rn << 5) | rt)

    def strb(self, rt: int, rn: int) -> None:
        self.emit(0x39000000 | (rn << 5) | rt)

    def strw(self, rt: int, rn: int) -> None:
        self.emit(0xB9000000 | (rn << 5) | rt)

    def svc(self) -> None:
        self.emit(0xD4000001)

    def ret(self) -> None:
        self.emit(0xD65F03C0)

    def b(self, label: str) -> None:
        self.fixups.append((len(self.words), label, "b", 0))
        self.emit(0x14000000)

    def bl(self, label: str) -> None:
        self.fixups.append((len(self.words), label, "bl", 0))
        self.emit(0x94000000)

    def bc(self, condition: int, label: str) -> None:
        self.fixups.append((len(self.words), label, "bc", condition))
        self.emit(0x54000000 | condition)

    def adr(self, rd: int, label: str) -> None:
        self.fixups.append((len(self.words), label, "adr", rd))
        self.emit(0x10000000 | rd)

    def finish(self) -> bytes:
        for at, target, kind, extra in self.fixups:
            delta_words = self.labels[target] - at
            if kind in ("b", "bl"):
                base = 0x14000000 if kind == "b" else 0x94000000
                self.words[at] = base | (delta_words & 0x03FFFFFF)
            elif kind == "bc":
                self.words[at] = 0x54000000 | ((delta_words & 0x7FFFF) << 5) | extra
            elif kind == "adr":
                delta = delta_words * 4
                self.words[at] = 0x10000000 | ((delta & 3) << 29) | (((delta >> 2) & 0x7FFFF) << 5) | extra
        return b"".join(struct.pack("<I", word) for word in self.words)


def elf_header() -> bytes:
    ident = b"\x7fELF" + bytes([2, 1, 1, 0, 0]) + bytes(7)
    header = ident + struct.pack(
        "<HHIQQQIHHHHHH",
        2, 183, 1, LOAD + HEADER_SIZE, 64, 0, 0, 64, 56, 1, 0, 0, 0,
    )
    ph = struct.pack("<IIQQQQQQ", 1, 5, 0, LOAD, LOAD, FILE_SIZE, FILE_SIZE, 0x1000)
    return header + ph


def build_program() -> tuple[bytes, int]:
    c = Code()
    sp = 31

    # 192 KiB: input, generated code, and guaranteed zero-fill scratch pages.
    c.sub_imm(sp, sp, 48, shift12=True)
    c.add_imm(19, sp, 0)                 # input base (SP cannot use MOV's XZR encoding)
    c.add_imm(22, sp, 16, shift12=True) # generated code base
    c.mov_reg(23, 22)                    # generated code cursor

    # read(stdin, input, 65535)
    c.movz(0, 0)
    c.mov_reg(1, 19)
    c.movz(2, 0xFFFF)
    c.movz(8, 63)
    c.svc()
    c.add_reg(20, 19, 0)                # input end
    c.mov_reg(21, 19)                    # input cursor

    c.label("main")
    c.cmp_reg(21, 20)
    c.bc(0x2, "finish")                 # HS
    c.ldrb(28, 21)
    c.add_imm(21, 21, 1)
    c.cmp_imm(28, 32)
    c.bc(0x9, "main")                   # LS: whitespace

    # Keep the first character, then skip the rest of the command name.
    c.label("skip_command")
    c.cmp_reg(21, 20)
    c.bc(0x2, "dispatch")
    c.ldrb(1, 21)
    c.cmp_imm(1, 32)
    c.bc(0x9, "dispatch")
    c.add_imm(21, 21, 1)
    c.b("skip_command")

    c.label("dispatch")
    for char, label in (("w", "word"), ("b", "byte"), ("m", "movz"), ("s", "svc"), ("j", "jump"), ("z", "zero")):
        c.cmp_imm(28, ord(char))
        c.bc(0x0, label)                 # EQ
    c.b("main")

    c.label("word")
    c.bl("parse_hex")
    c.strw(0, 23)
    c.add_imm(23, 23, 4)
    c.b("main")

    c.label("byte")
    c.bl("parse_hex")
    c.strb(0, 23)
    c.add_imm(23, 23, 1)
    c.b("main")

    c.label("movz")
    c.bl("parse_hex")
    c.mov_reg(27, 0)                    # destination register
    c.bl("parse_hex")                   # immediate
    for _ in range(5):
        c.add_reg(0, 0, 0, bits=32)     # imm << 5
    c.orr_reg(0, 0, 27, bits=32)
    c.movz(1, 0)
    c.movk(1, 0xD280, bits=32, shift=16)
    c.orr_reg(0, 0, 1, bits=32)
    c.strw(0, 23)
    c.add_imm(23, 23, 4)
    c.b("main")

    c.label("svc")
    c.movz(0, 1)
    c.movk(0, 0xD400, bits=32, shift=16)
    c.strw(0, 23)
    c.add_imm(23, 23, 4)
    c.b("main")

    c.label("jump")
    c.bl("parse_hex")
    c.movz(1, 0)
    c.movk(1, 0x1400, bits=32, shift=16)
    c.orr_reg(0, 0, 1, bits=32)
    c.strw(0, 23)
    c.add_imm(23, 23, 4)
    c.b("main")

    c.label("zero")
    c.bl("parse_hex")
    c.cb_zero_loop = len(c.words)
    c.label("zero_loop")
    c.cmp_imm(0, 0, bits=64)
    c.bc(0x0, "main")
    c.strb(31, 23)
    c.add_imm(23, 23, 1)
    c.sub_imm(0, 0, 1)
    c.b("zero_loop")

    # Parse one hexadecimal argument, returning its value in x0.
    c.label("parse_hex")
    c.movz(0, 0)
    c.label("hex_skip")
    c.cmp_reg(21, 20)
    c.bc(0x2, "hex_done")
    c.ldrb(1, 21)
    c.add_imm(21, 21, 1)
    c.cmp_imm(1, 32)
    c.bc(0x9, "hex_skip")

    c.label("hex_char")
    c.cmp_imm(1, ord("0"))
    c.bc(0x3, "hex_done")              # LO
    c.cmp_imm(1, ord("9"))
    c.bc(0x9, "hex_digit")             # LS
    c.cmp_imm(1, ord("A"))
    c.bc(0x3, "hex_lower")
    c.cmp_imm(1, ord("F"))
    c.bc(0x9, "hex_upper")
    c.label("hex_lower")
    c.cmp_imm(1, ord("a"))
    c.bc(0x3, "hex_done")
    c.cmp_imm(1, ord("f"))
    c.bc(0x8, "hex_done")              # HI
    c.sub_imm(1, 1, ord("a"))
    c.add_imm(1, 1, 10)
    c.b("hex_accumulate")
    c.label("hex_upper")
    c.sub_imm(1, 1, ord("A"))
    c.add_imm(1, 1, 10)
    c.b("hex_accumulate")
    c.label("hex_digit")
    c.sub_imm(1, 1, ord("0"))
    c.label("hex_accumulate")
    for _ in range(4):
        c.add_reg(0, 0, 0)
    c.add_reg(0, 0, 1)
    c.cmp_reg(21, 20)
    c.bc(0x2, "hex_done")
    c.ldrb(1, 21)
    c.add_imm(21, 21, 1)
    c.b("hex_char")
    c.label("hex_done")
    c.ret()

    c.label("finish")
    # Pad generated code to a fixed 3976-byte segment.
    c.add_imm(24, 22, CODE_CAPACITY)
    c.label("pad")
    c.cmp_reg(23, 24)
    c.bc(0x2, "write_result")
    c.strb(31, 23)
    c.add_imm(23, 23, 1)
    c.b("pad")

    c.label("write_result")
    c.movz(0, 1)
    c.adr(1, "header_template")
    c.movz(2, HEADER_SIZE)
    c.movz(8, 64)
    c.svc()
    c.movz(0, 1)
    c.mov_reg(1, 22)
    c.movz(2, CODE_CAPACITY)
    c.movz(8, 64)
    c.svc()
    c.movz(0, 0)
    c.movz(8, 93)
    c.svc()

    c.label("header_template")
    code = c.finish()
    template_offset = len(code)
    code += elf_header()
    if len(code) > CODE_CAPACITY:
        raise ValueError(f"M0 code exceeds fixed capacity: {len(code)}")
    code += bytes(CODE_CAPACITY - len(code))
    return elf_header() + code, template_offset


def self_source(binary: bytes) -> str:
    code = binary[HEADER_SIZE:]
    last = len(code)
    while last > 0 and code[last - 1] == 0:
        last -= 1
    rounded = (last + 3) & ~3
    rows = [f"word {struct.unpack_from('<I', code, offset)[0]:08x}" for offset in range(0, rounded, 4)]
    if rounded < len(code):
        rows.append(f"zero {len(code) - rounded:x}")
    return "\n".join(rows) + "\n"


def main() -> None:
    DIST.mkdir(parents=True, exist_ok=True)
    binary, _ = build_program()
    source = self_source(binary)
    (DIST / "m0-bootstrap.hex").write_text("\n".join(f"{b:02x}" for b in binary) + "\n", encoding="ascii")
    (DIST / "m0.m0").write_text(source, encoding="ascii")
    (DIST / "m0-expected.elf").write_bytes(binary)
    os.chmod(DIST / "m0-expected.elf", 0o755)
    print(f"m0 binary: {len(binary)} bytes")
    print(f"m0 source: {len(source)} bytes")
    print(f"m0 sha256: {hashlib.sha256(binary).hexdigest()}")


if __name__ == "__main__":
    main()
