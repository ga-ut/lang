# Raw Core 0.4 compiler

`compiler.raw` is the readable Raw Core 0.4 compiler. It validates Raw Core and
directly emits a deterministic static AArch64 Linux ELF:

```text
Raw Core source -> raw0 compiler -> AArch64 ELF
```

The direct compiler has rebuilt its own source through two byte-identical
native generations without invoking the Core-0 backend. `compiler.core` and
`../dist/core0.elf` remain recovery seeds that constructed the first readable
and first direct generations.

The verified direct compiler is retained as `../dist/raw0.elf`. It writes the
ELF header and AArch64 instruction words itself and needs no assembler, linker,
libc, or dynamic loader.

The bootstrap-host target supports `host.read`, `host.write`, and `host.exit`.
They are target effects rather than canonical Raw Core primitives.
