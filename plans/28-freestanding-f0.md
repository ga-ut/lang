# Gaut freestanding F0

## Goal

Make the self-hosted Gaut compiler emit one deterministic AArch64 image that
QEMU `virt` boots directly without Linux, libc, a dynamic loader, a foreign
runtime, an assembler, or a linker.

## Source and output contract

- A freestanding program starts with `target qemu_virt;`.
- The declaration selects a checked board layout and entry adapter. It is not
  a runtime primitive and does not add an alias for any memory behavior.
- `dist/gaut.elf < os/boot.gaut > dist/gaut-os.img` emits a page-aligned raw
  image instead of an ELF.
- The compiler supplies only the initial stack, Gaut evaluation and return
  storage, the call to `main`, and a wait loop after `main` returns.
- `os/boot.gaut` owns the PL011 driver and uses the existing ordered `load32`
  and `store32` primitives for MMIO.

## Acceptance transcript

```text
gaut-os: boot
```

The transcript must be observed once on QEMU `virt` serial output. A missing,
duplicated, or different line, an unexpected exception, or a timeout is a
failure.

## Gates

1. Add the optional target declaration without changing existing hosted Gaut
   source or primitive IDs.
2. Emit the freestanding entry adapter and raw image through one direct
   AArch64 backend path.
3. Build `os/boot.gaut` and require deterministic image bytes.
4. Rebuild `gaut/compiler.gaut` through two byte-identical hosted generations
   in the isolated offline AArch64 VM.
5. Boot the image twice in a clean networkless QEMU `virt` process and match
   the exact transcript.
6. Retain source, final compiler, final image, checksums, and a concise record;
   remove compiler generations, VM runtime, QEMU state, and logs.

## Boundary

- F0 proves that Gaut owns reset-to-program execution and UART MMIO. It is not
  yet the interactive Linux-free development loop.
- F1 will load the Gaut compiler in Gaut OS, accept source bytes, compile into
  memory, and run the generated program.
- Page allocation, exceptions, timer interrupts, scheduling, storage,
  graphics, networking, browser, and AI remain after F0.
- No remote push without explicit authorization.

## Status

- Gate 1: complete
- Gate 2: complete
- Gate 3: complete
- Gate 4: complete
- Gate 5: complete
- Gate 6: complete
