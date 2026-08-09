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

`examples/compiled.gaut` is the F1 child program. `run-f1.sh` frames the
selected source in a disposable RAM packet, boots `dist/gaut-f1.img` with no
network device, and lets the freestanding Gaut compiler compile and run it.
No Python, C, Rust, LLVM, assembler, linker, Linux guest, or persistent disk is
in that path. QEMU remains the replaceable hardware emulator on the host.

```sh
./os/run-f1.sh os/examples/compiled.gaut
```

The current F1 machine is one-shot. Press Control-C after the child finishes;
the launcher then removes its temporary packet. Repeated editing, named files,
and persistent storage begin in F2.
