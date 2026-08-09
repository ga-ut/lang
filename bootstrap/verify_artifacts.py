#!/usr/bin/env python3
"""Host-side audit helper; not part of the M0 runtime or build path."""

from __future__ import annotations

import hashlib
import struct
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
EXPECTED_M0 = "272c00e94ba538e08ef1f253cbbc8d279c457816aee810ad3269e9c912206c3c"


def inspect(path: Path, expected_hash: str) -> bytes:
    data = path.read_bytes()
    assert len(data) == 4096
    assert data[:4] == b"\x7fELF"
    assert data[4:7] == bytes([2, 1, 1])
    assert struct.unpack_from("<H", data, 18)[0] == 183
    phoff = struct.unpack_from("<Q", data, 32)[0]
    assert struct.unpack_from("<H", data, 56)[0] == 1
    segment_type, flags = struct.unpack_from("<II", data, phoff)
    assert (segment_type, flags) == (1, 5)
    assert hashlib.sha256(data).hexdigest() == expected_hash
    return data


def main() -> None:
    inspect(DIST / "m0.elf", EXPECTED_M0)
    inspect(
        DIST / "exit42.elf",
        "fbbda142d085fe85ab2aad0ffe98c3bfb61a77086fce7e1ca31ffaeac69b8ea1",
    )
    assert (DIST / "M0_VERIFIED").is_file()
    print("verified: M0 and exit42 are static AArch64 ELF files")
    print("verified: one read+execute PT_LOAD segment, no interpreter")
    print(f"M0 sha256: {EXPECTED_M0}")


if __name__ == "__main__":
    main()
