#!/usr/bin/env python3
"""Audit constructor for Core-0.

This file directly encodes AArch64 and ELF.  It never invokes a C compiler,
assembler, linker, LLVM, or external library.  It is intentionally not the
final active compiler: compiler.core must replace it at the self-hosting gate.
"""

from __future__ import annotations

import argparse
import re
import struct
from pathlib import Path


LOAD = 0x400000
HEADER_SIZE = 120
PAGE = 4096


class CompileError(Exception):
    pass


class Code:
    def __init__(self) -> None:
        self.words: list[int] = []
        self.origins: list[int] = []
        self.current_line = 0

    def emit(self, word: int) -> int:
        at = len(self.words)
        self.words.append(word & 0xFFFFFFFF)
        self.origins.append(self.current_line)
        return at

    def patch_b(self, at: int, target: int, link: bool = False) -> None:
        delta = target - at
        if not -(1 << 25) <= delta < (1 << 25):
            raise CompileError("branch is out of range")
        self.words[at] = (0x94000000 if link else 0x14000000) | (delta & 0x03FFFFFF)

    def patch_cbz(self, at: int, target: int, register: int = 0) -> None:
        delta = target - at
        if not -(1 << 18) <= delta < (1 << 18):
            raise CompileError("conditional branch is out of range")
        self.words[at] = 0xB4000000 | ((delta & 0x7FFFF) << 5) | register

    def mov_u64(self, rd: int, value: int) -> None:
        value &= 0xFFFFFFFFFFFFFFFF
        self.emit(0xD2800000 | ((value & 0xFFFF) << 5) | rd)
        for shift in (16, 32, 48):
            part = (value >> shift) & 0xFFFF
            if part:
                self.emit(0xF2800000 | ((shift // 16) << 21) | (part << 5) | rd)

    def push(self, register: int = 0) -> None:
        # str xN, [x20]; add x20, x20, #8
        self.emit(0xF9000000 | (20 << 5) | register)
        self.emit(0x91002294)

    def pop(self, register: int = 0) -> None:
        # sub x20, x20, #8; ldr xN, [x20]
        self.emit(0xD1002294)
        self.emit(0xF9400000 | (20 << 5) | register)

    def bytes(self) -> bytes:
        return b"".join(struct.pack("<I", word) for word in self.words)


def tokenize(source: str) -> list[tuple[str, int]]:
    result: list[tuple[str, int]] = []
    for line_no, line in enumerate(source.splitlines(), 1):
        line = line.split("#", 1)[0]
        result.extend((token, line_no) for token in line.split())
    return result


def binary_op(code: Code, instruction: int) -> None:
    code.pop(1)
    code.pop(0)
    code.emit(instruction)
    code.push(0)


def compare_op(code: Code, condition: int) -> None:
    code.pop(1)
    code.pop(0)
    code.emit(0xEB01001F)  # cmp x0, x1
    # CSET is CSINC with both inputs XZR and the requested condition inverted.
    code.emit(0x9A800400 | (31 << 16) | ((condition ^ 1) << 12) | (31 << 5))
    code.push(0)


def compile_source(source: str, origin_map: list[int] | None = None) -> bytes:
    tokens = tokenize(source)
    code = Code()

    # One explicit 1 MiB arena. x19 is its base, x20 the value-stack cursor,
    # and x21 a separate return-address stack. A normal AArch64 BL overwrites
    # x30, so every Core function preserves it here before making nested calls.
    code.emit(0xD14403FF)  # sub sp, sp, #0x100, lsl #12
    code.emit(0x910003F3)  # mov x19, sp
    code.emit(0x914007F4)  # add x20, sp, #1, lsl #12
    code.emit(0x912003F5)  # add x21, sp, #0x800
    entry_call = code.emit(0x94000000)
    code.pop(0)
    code.mov_u64(8, 93)
    code.emit(0xD4000001)  # svc #0

    functions: dict[str, int] = {}
    main_at: int | None = None
    blocks: list[tuple[str, int]] = []
    in_function = False
    i = 0

    simple_binary = {
        "+": 0x8B010000,       # add x0, x0, x1
        "-": 0xCB010000,       # sub x0, x0, x1
        "*": 0x9B017C00,       # mul x0, x0, x1
        "/": 0x9AC10800,       # udiv x0, x0, x1
        "&": 0x8A010000,
        "|": 0xAA010000,
        "^": 0xCA010000,
        "<<": 0x9AC12000,      # lslv x0, x0, x1
        ">>": 0x9AC12400,      # lsrv x0, x0, x1
    }
    conditions = {"=": 0x0, "!=": 0x1, ">=": 0x2, "<": 0x3, ">": 0x8, "<=": 0x9}

    while i < len(tokens):
        token, line = tokens[i]
        i += 1
        code.current_line = line

        if token == "fn":
            if in_function or i >= len(tokens):
                raise CompileError(f"line {line}: invalid fn")
            name, name_line = tokens[i]
            i += 1
            if name in functions:
                raise CompileError(f"line {name_line}: duplicate function {name}")
            functions[name] = len(code.words)
            if name == "main":
                main_at = len(code.words)
            code.emit(0xF90002BE)  # str x30, [x21]
            code.emit(0x910022B5)  # add x21, x21, #8
            in_function = True
            continue

        if token == "endfn":
            if not in_function or blocks:
                raise CompileError(f"line {line}: endfn with an open control block")
            code.emit(0xD10022B5)  # sub x21, x21, #8
            code.emit(0xF94002BE)  # ldr x30, [x21]
            code.emit(0xD65F03C0)
            in_function = False
            continue

        if not in_function:
            raise CompileError(f"line {line}: executable token outside a function: {token}")

        if token.startswith("$"):
            try:
                value = int(token[1:], 16)
            except ValueError as error:
                raise CompileError(f"line {line}: invalid hexadecimal literal {token}") from error
            code.mov_u64(0, value)
            code.push(0)
        elif token in simple_binary:
            binary_op(code, simple_binary[token])
        elif token in conditions:
            compare_op(code, conditions[token])
        elif token == "dup":
            code.pop(0)
            code.push(0)
            code.push(0)
        elif token == "drop":
            code.pop(0)
        elif token == "swap":
            code.pop(1)
            code.pop(0)
            code.push(1)
            code.push(0)
        elif token == "over":
            code.pop(1)
            code.pop(0)
            code.push(0)
            code.push(1)
            code.push(0)
        elif token == "mem":
            # User arena starts at +64 KiB. The first page holds locals,
            # return addresses, and the evaluation stack; the regions must not
            # alias because merely pushing a pointer would overwrite user data.
            code.emit(0x91404260)  # add x0, x19, #0x10, lsl #12
            code.push(0)
        elif token == "load8":
            code.pop(0)
            code.emit(0x39400000)
            code.push(0)
        elif token == "load32":
            code.pop(0)
            code.emit(0xB9400000)
            code.push(0)
        elif token == "load64":
            code.pop(0)
            code.emit(0xF9400000)
            code.push(0)
        elif token in ("store8", "store32", "store64"):
            code.pop(1)  # address
            code.pop(0)  # value
            code.emit({"store8": 0x39000020, "store32": 0xB9000020, "store64": 0xF9000020}[token])
        elif re.fullmatch(r"v[0-9a-fA-F]{1,2}@", token):
            slot = int(token[1:-1], 16)
            if slot >= 128:
                raise CompileError(f"line {line}: local slot is out of range")
            code.emit(0xF9400000 | (slot << 10) | (19 << 5))
            code.push(0)
        elif re.fullmatch(r"v[0-9a-fA-F]{1,2}!", token):
            slot = int(token[1:-1], 16)
            if slot >= 128:
                raise CompileError(f"line {line}: local slot is out of range")
            code.pop(0)
            code.emit(0xF9000000 | (slot << 10) | (19 << 5))
        elif token == "if":
            code.pop(0)
            blocks.append(("if", code.emit(0xB4000000)))
        elif token == "else":
            if not blocks or blocks[-1][0] != "if":
                raise CompileError(f"line {line}: else without if")
            _, conditional = blocks.pop()
            jump = code.emit(0x14000000)
            code.patch_cbz(conditional, len(code.words))
            blocks.append(("else", jump))
        elif token == "endif":
            if not blocks or blocks[-1][0] not in ("if", "else"):
                raise CompileError(f"line {line}: endif without if")
            kind, branch = blocks.pop()
            if kind == "if":
                code.patch_cbz(branch, len(code.words))
            else:
                code.patch_b(branch, len(code.words))
        elif token == "while":
            blocks.append(("while", len(code.words)))
        elif token == "do":
            if not blocks or blocks[-1][0] != "while":
                raise CompileError(f"line {line}: do without while")
            code.pop(0)
            blocks.append(("do", code.emit(0xB4000000)))
        elif token == "endwhile":
            if len(blocks) < 2 or blocks[-1][0] != "do" or blocks[-2][0] != "while":
                raise CompileError(f"line {line}: endwhile without while/do")
            _, exit_branch = blocks.pop()
            _, loop_start = blocks.pop()
            back = code.emit(0x14000000)
            code.patch_b(back, loop_start)
            code.patch_cbz(exit_branch, len(code.words))
        elif token == "call":
            if i >= len(tokens):
                raise CompileError(f"line {line}: call requires a function name")
            name, name_line = tokens[i]
            i += 1
            if name not in functions:
                raise CompileError(f"line {name_line}: calls must refer to an earlier function: {name}")
            at = code.emit(0x94000000)
            code.patch_b(at, functions[name], link=True)
        elif token in ("host.read", "host.write"):
            code.pop(2)
            code.pop(1)
            code.pop(0)
            code.mov_u64(8, 63 if token == "host.read" else 64)
            code.emit(0xD4000001)
            code.push(0)
        elif token == "host.exit":
            code.pop(0)
            code.mov_u64(8, 93)
            code.emit(0xD4000001)
        else:
            raise CompileError(f"line {line}: unknown token {token}")

    if in_function:
        raise CompileError("end of file: missing endfn")
    if blocks:
        raise CompileError("end of file: open control block")
    if main_at is None:
        raise CompileError("program has no main function")
    code.patch_b(entry_call, main_at, link=True)
    if origin_map is not None:
        origin_map.extend(code.origins)

    body = code.bytes()
    file_size = (HEADER_SIZE + len(body) + PAGE - 1) & -PAGE
    ident = b"\x7fELF" + bytes([2, 1, 1, 0, 0]) + bytes(7)
    header = ident + struct.pack(
        "<HHIQQQIHHHHHH",
        2, 183, 1, LOAD + HEADER_SIZE, 64, 0, 0, 64, 56, 1, 0, 0, 0,
    )
    program_header = struct.pack(
        "<IIQQQQQQ", 1, 5, 0, LOAD, LOAD, file_size, file_size, PAGE,
    )
    return (header + program_header + body).ljust(file_size, b"\0")


def main() -> int:
    parser = argparse.ArgumentParser(description="direct Core-0 -> AArch64 ELF audit constructor")
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        binary = compile_source(args.source.read_text())
    except CompileError as error:
        parser.exit(1, f"core0: {error}\n")
    args.output.write_bytes(binary)
    args.output.chmod(0o755)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
