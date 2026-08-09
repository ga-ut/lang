# Gaut freestanding F1

## Goal

Boot the self-hosted Gaut compiler directly on QEMU `virt`, read one Gaut
source packet already present in RAM, compile it into RAM, and transfer control
to the generated program without Linux or a foreign runtime.

## Source and execution contract

- The retained compiler source remains one readable `gaut/compiler.gaut`.
- The freestanding compiler build prepends `target qemu_virt;` in disposable
  build input; it does not retain a forked compiler implementation.
- The launch harness places an eight-byte little-endian source length followed
  by the exact source bytes at physical address `0x47000000`.
- On `qemu_virt`, `host.read` consumes that single checked RAM packet and
  `host.write` sends bytes through the PL011 UART.
- `platform.run(address)` is the sole target effect for transferring control
  to a generated image. The bootstrap-host adapter consumes the address and
  returns; the `qemu_virt` adapter branches to it.
- The first F1 loop is deliberately one-shot: one source packet, one compile,
  one execution per clean boot.

## Acceptance transcript

The child program's serial text must be exactly:

```text
gaut-os: compiled
```

The child line must occur once. A compile diagnostic, unexpected exception,
second child line, binary compiler output, or timeout is a failure.

## Gates

1. Specify the RAM packet, target-effect, and one-shot execution contracts.
2. Implement `qemu_virt` RAM input, PL011 output, failure halt, and generated
   image transfer in the single direct AArch64 backend.
3. Keep hosted compiler behavior stable and reach two byte-identical hosted
   self-host generations in the offline AArch64 VM.
4. Build a freestanding compiler from the retained compiler source without
   retaining a second source tree.
5. Boot the compiler with a Gaut child source packet in clean networkless QEMU,
   observe the exact final transcript, and repeat once.
6. Retain source, final compiler, freestanding compiler image, example,
   checksums, and concise verification evidence; remove generated packets,
   compiler generations, emulator state, logs, and temporary dependencies.

## Boundary after F1

- F1 truthfully proves that Gaut OS can compile and execute one Gaut program.
- It does not yet provide an editor, command loop, named files, persistent
  storage, incremental compilation, process isolation, or recovery after a
  faulty child program.
- F2 will add a serial command monitor and an explicit persistent workspace so
  multiple edit/build/run cycles can occur in one environment.
- No remote push without explicit authorization.

## Status

- Gate 1: complete
- Gate 2: complete
- Gate 3: complete
- Gate 4: complete
- Gate 5: complete
- Gate 6: complete
