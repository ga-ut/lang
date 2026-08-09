#!/usr/bin/env python3
"""Construct the first M0 hex0 compiler without an assembler or linker."""

from __future__ import annotations

import hashlib
import os
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
LOAD_ADDRESS = 0x400000
ELF_HEADER_SIZE = 64
PROGRAM_HEADER_SIZE = 56
CODE_OFFSET = ELF_HEADER_SIZE + PROGRAM_HEADER_SIZE


class A64:
    def __init__(self) -> None:
        self.words: list[int] = []
        self.labels: dict[str, int] = {}
        self.fixups: list[tuple[int, str, str, int]] = []

    def emit(self, word: int) -> None:
        self.words.append(word & 0xFFFFFFFF)

    def label(self, name: str) -> None:
        if name in self.labels:
            raise ValueError(f"duplicate label: {name}")
        self.labels[name] = len(self.words)

    def movz(self, rd: int, imm16: int, *, bits: int = 64) -> None:
        base = 0xD2800000 if bits == 64 else 0x52800000
        self.emit(base | ((imm16 & 0xFFFF) << 5) | rd)

    def add_imm(self, rd: int, rn: int, imm12: int, *, bits: int = 64) -> None:
        base = 0x91000000 if bits == 64 else 0x11000000
        self.emit(base | ((imm12 & 0xFFF) << 10) | (rn << 5) | rd)

    def sub_imm(self, rd: int, rn: int, imm12: int, *, bits: int = 64) -> None:
        base = 0xD1000000 if bits == 64 else 0x51000000
        self.emit(base | ((imm12 & 0xFFF) << 10) | (rn << 5) | rd)

    def cmp_imm(self, rn: int, imm12: int, *, bits: int = 64) -> None:
        base = 0xF100001F if bits == 64 else 0x7100001F
        self.emit(base | ((imm12 & 0xFFF) << 10) | (rn << 5))

    def add_reg(self, rd: int, rn: int, rm: int, *, bits: int = 32) -> None:
        base = 0x8B000000 if bits == 64 else 0x0B000000
        self.emit(base | (rm << 16) | (rn << 5) | rd)

    def mov_reg(self, rd: int, rm: int, *, bits: int = 32) -> None:
        base = 0xAA0003E0 if bits == 64 else 0x2A0003E0
        self.emit(base | (rm << 16) | rd)

    def orr_reg(self, rd: int, rn: int, rm: int, *, bits: int = 32) -> None:
        base = 0xAA000000 if bits == 64 else 0x2A000000
        self.emit(base | (rm << 16) | (rn << 5) | rd)

    def ldrb(self, rt: int, rn: int) -> None:
        self.emit(0x39400000 | (rn << 5) | rt)

    def strb(self, rt: int, rn: int) -> None:
        self.emit(0x39000000 | (rn << 5) | rt)

    def svc(self) -> None:
        self.emit(0xD4000001)

    def b(self, label: str) -> None:
        self.fixups.append((len(self.words), label, "b", 0))
        self.emit(0x14000000)

    def b_cond(self, condition: int, label: str) -> None:
        self.fixups.append((len(self.words), label, "b.cond", condition))
        self.emit(0x54000000 | condition)

    def cbz(self, rt: int, label: str, *, nonzero: bool = False) -> None:
        base = 0xB5000000 if nonzero else 0xB4000000
        self.fixups.append((len(self.words), label, "cbz", base))
        self.emit(base | rt)

    def finish(self) -> bytes:
        for at, name, kind, extra in self.fixups:
            if name not in self.labels:
                raise ValueError(f"missing label: {name}")
            delta = self.labels[name] - at
            if kind == "b":
                if not -(1 << 25) <= delta < (1 << 25):
                    raise ValueError("branch out of range")
                self.words[at] = 0x14000000 | (delta & 0x03FFFFFF)
            elif kind == "b.cond":
                if not -(1 << 18) <= delta < (1 << 18):
                    raise ValueError("conditional branch out of range")
                self.words[at] = 0x54000000 | ((delta & 0x7FFFF) << 5) | extra
            elif kind == "cbz":
                if not -(1 << 18) <= delta < (1 << 18):
                    raise ValueError("compare branch out of range")
                rt = self.words[at] & 0x1F
                self.words[at] = extra | ((delta & 0x7FFFF) << 5) | rt
            else:
                raise AssertionError(kind)
        return b"".join(struct.pack("<I", word) for word in self.words)


def build_code() -> bytes:
    a = A64()
    sp = 31

    # x19: whether one high nibble is pending
    # w20: pending high nibble
    a.sub_imm(sp, sp, 16)
    a.movz(19, 0)

    a.label("read")
    a.movz(0, 0)            # stdin
    a.add_imm(1, sp, 0)     # one-byte buffer
    a.movz(2, 1)
    a.movz(8, 63)           # Linux ARM64 read
    a.svc()
    a.cmp_imm(0, 0)
    a.b_cond(0xD, "eof")    # LE: EOF or read error
    a.ldrb(3, sp)

    # ASCII 0-9
    a.cmp_imm(3, ord("0"), bits=32)
    a.b_cond(0x3, "upper_check")  # LO
    a.cmp_imm(3, ord("9"), bits=32)
    a.b_cond(0x9, "digit")        # LS

    # ASCII A-F
    a.label("upper_check")
    a.cmp_imm(3, ord("A"), bits=32)
    a.b_cond(0x3, "lower_check")
    a.cmp_imm(3, ord("F"), bits=32)
    a.b_cond(0x9, "upper")

    # ASCII a-f; every other byte is a separator
    a.label("lower_check")
    a.cmp_imm(3, ord("a"), bits=32)
    a.b_cond(0x3, "read")
    a.cmp_imm(3, ord("f"), bits=32)
    a.b_cond(0x8, "read")          # HI
    a.sub_imm(4, 3, ord("a"), bits=32)
    a.add_imm(4, 4, 10, bits=32)
    a.b("nibble")

    a.label("upper")
    a.sub_imm(4, 3, ord("A"), bits=32)
    a.add_imm(4, 4, 10, bits=32)
    a.b("nibble")

    a.label("digit")
    a.sub_imm(4, 3, ord("0"), bits=32)

    a.label("nibble")
    a.cbz(19, "save_high")

    # w5 = (w20 << 4) | w4, expressed with ADDs to keep seed encoding tiny.
    a.add_reg(5, 20, 20)
    a.add_reg(5, 5, 5)
    a.add_reg(5, 5, 5)
    a.add_reg(5, 5, 5)
    a.orr_reg(5, 5, 4)
    a.strb(5, sp)
    a.movz(19, 0)

    a.movz(0, 1)            # stdout
    a.add_imm(1, sp, 0)
    a.movz(2, 1)
    a.movz(8, 64)           # Linux ARM64 write
    a.svc()
    a.b("read")

    a.label("save_high")
    a.mov_reg(20, 4)
    a.movz(19, 1)
    a.b("read")

    a.label("eof")
    a.cbz(19, "success")
    a.movz(0, 2)            # odd number of nibbles
    a.b("exit")

    a.label("success")
    a.movz(0, 0)

    a.label("exit")
    a.movz(8, 93)           # Linux ARM64 exit
    a.svc()

    return a.finish()


def build_elf(code: bytes) -> bytes:
    entry = LOAD_ADDRESS + CODE_OFFSET
    total_size = CODE_OFFSET + len(code)

    ident = b"\x7fELF" + bytes([2, 1, 1, 0, 0]) + bytes(7)
    elf_header = ident + struct.pack(
        "<HHIQQQIHHHHHH",
        2,                    # ET_EXEC
        183,                  # EM_AARCH64
        1,
        entry,
        ELF_HEADER_SIZE,
        0,
        0,
        ELF_HEADER_SIZE,
        PROGRAM_HEADER_SIZE,
        1,
        0,
        0,
        0,
    )
    program_header = struct.pack(
        "<IIQQQQQQ",
        1,                    # PT_LOAD
        5,                    # PF_R | PF_X
        0,
        LOAD_ADDRESS,
        LOAD_ADDRESS,
        total_size,
        total_size,
        0x1000,
    )
    assert len(elf_header) == ELF_HEADER_SIZE
    assert len(program_header) == PROGRAM_HEADER_SIZE
    return elf_header + program_header + code


def format_self_source(binary: bytes) -> str:
    rows = []
    for offset in range(0, len(binary), 16):
        chunk = binary[offset : offset + 16]
        rows.append(" ".join(f"{byte:02x}" for byte in chunk))
    return "\n".join(rows) + "\n"


def main() -> None:
    DIST.mkdir(parents=True, exist_ok=True)
    binary = build_elf(build_code())
    seed_path = DIST / "seed0.elf"
    source_path = DIST / "hex0.hex"
    seed_path.write_bytes(binary)
    source_path.write_text(format_self_source(binary), encoding="ascii")
    os.chmod(seed_path, 0o755)

    digest = hashlib.sha256(binary).hexdigest()
    (DIST / "SHA256SUMS").write_text(
        f"{digest}  seed0.elf\n{hashlib.sha256(source_path.read_bytes()).hexdigest()}  hex0.hex\n",
        encoding="ascii",
    )
    print(f"built {seed_path} ({len(binary)} bytes)")
    print(f"self source {source_path} ({source_path.stat().st_size} bytes)")
    print(f"seed sha256 {digest}")


if __name__ == "__main__":
    main()
