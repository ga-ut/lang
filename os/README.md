# Gaut OS

`boot.gaut` is the F0 machine-boundary program for the checked AArch64 QEMU
`virt` board. It selects the freestanding target explicitly, polls the PL011
transmit-full flag, writes the acceptance line through ordered 32-bit MMIO,
returns from `main`, and enters the compiler-generated wait loop.

The decimal board constants are:

- `150994944`: PL011 base address `0x09000000`
- `24`: PL011 flag-register offset `0x18`
- `32`: transmit FIFO full bit `1 << 5`

They remain in this board-specific source and are not Gaut language semantics.

Expected serial transcript:

```text
gaut-os: boot
```
