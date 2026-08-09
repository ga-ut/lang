# Gaut OS contracts

## F0 — direct boot

F0 proves the smallest machine boundary before O1:

- `os/boot.gaut` explicitly selects `target qemu_virt;`;
- `dist/gaut.elf` emits a deterministic raw AArch64 image;
- QEMU `virt` boots it directly without Linux or a foreign runtime;
- the PL011 driver is Gaut source using ordered memory primitives; and
- serial output is exactly `gaut-os: boot` followed by one newline.

The compiler may synthesize only the initial stack, evaluation and return
storage, the call to `main`, and the post-return wait loop. F0 does not include
an allocator, exception vectors, interrupts, scheduling, storage, or an
interactive development environment.

## F1 — one-shot compile and run

F1 boots the same self-hosted compiler as a freestanding Gaut program. The
launcher places an eight-byte little-endian source length and the exact Gaut
source bytes at physical address `0x47000000`. The compiler reads that packet,
emits a raw `qemu_virt` image into its fixed output memory, and transfers
control to its entry word. The hosted adapter returns from `platform.run` and
then writes the artifact; the freestanding adapter does not return, so binary
artifact bytes do not leak onto the serial console.

The retained acceptance child is `os/examples/compiled.gaut`. Its final serial
line is exactly:

```text
gaut-os: compiled
```

This proves one Linux-free `source -> compile -> execute` cycle. It does not
provide interactive editing, a command loop, files, persistent storage,
isolation from a faulty child, or a second compilation in the same boot.

## O1 — operating-system nucleus

O1 is a small operating-system nucleus, not a Linux replacement and not a
browser demo. Its job is to prove that Gaut can own the wider machine boundary.

## Initial machine

- architecture: AArch64
- reference board: QEMU `virt`
- privilege at entry: EL1 when QEMU's direct-kernel boot provides it; the boot
  shim normalizes other supported entry states before Gaut kernel code runs
- console: PL011 UART
- interrupt controller: the QEMU `virt` GIC version selected by the launch
  manifest
- clock: ARM generic timer
- memory layout: supplied by one checked board description, never guessed by
  ordinary kernel code

QEMU is a replaceable hardware model. QEMU behavior is not language semantics.

## What must be Gaut source

- kernel entry state machine after the compiler-generated reset shim
- UART driver
- physical page allocator
- exception handlers and reports
- timer setup and interrupt handling
- cooperative scheduler and its first task

The compiler may synthesize binary layout, the minimal reset shim, exception
vector alignment, and target intrinsics. No handwritten C, Rust, assembly, or
foreign runtime is linked into the kernel.

## Required boot transcript

```text
gaut-os: entry
gaut-os: uart ok
gaut-os: memory ok
gaut-os: exceptions ok
gaut-os: timer ok
gaut-os: task ok
gaut-os: O1 PASS
```

The test harness must also provoke one controlled synchronous exception and
observe its class before continuing. A timeout, missing line, duplicate boot,
or unexpected exception is failure.

## Isolation and footprint

- The compiler VM and OS emulator use no network by default.
- Build and run storage is created on demand.
- Generated disks, compiler generations, traces, and emulator caches are kept
  under the project output directory or a disposable temporary directory.
- After verification, retain source, the final compiler, the final kernel, a
  checksum manifest, and concise logs. Remove intermediate generations,
  temporary disks, crash dumps, and caches.

## O1 completion test

1. Build the compiler from the retained self-hosted compiler.
2. Build the kernel using that newly built compiler.
3. Boot it in a clean, networkless QEMU process.
4. Match the required serial transcript and controlled-exception evidence.
5. Repeat from clean generated state and compare compiler and kernel hashes.
