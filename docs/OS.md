# Core OS O1 contract

O1 is a small operating-system nucleus, not a Linux replacement and not a
browser demo. Its job is to prove that Core can own the machine boundary.

## Initial machine

- architecture: AArch64
- reference board: QEMU `virt`
- privilege at entry: EL1 when QEMU's direct-kernel boot provides it; the boot
  shim normalizes other supported entry states before Core kernel code runs
- console: PL011 UART
- interrupt controller: the QEMU `virt` GIC version selected by the launch
  manifest
- clock: ARM generic timer
- memory layout: supplied by one checked board description, never guessed by
  ordinary kernel code

QEMU is a replaceable hardware model. QEMU behavior is not language semantics.

## What must be Core source

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
core-os: entry
core-os: uart ok
core-os: memory ok
core-os: exceptions ok
core-os: timer ok
core-os: task ok
core-os: O1 PASS
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
