# Raw Core 0.4 compiler

`compiler.raw` is the readable Raw Core 0.4 compiler. It validates Raw Core and
emits deterministic Core-0 source. The retained Core-0 compiler performs
AArch64 ELF lowering:

```text
Raw Core source -> raw0 compiler -> validated Core-0 source -> core0 compiler -> AArch64 ELF
```

The readable compiler has rebuilt its own source through two byte-identical
native generations. `compiler.core` remains the recovery bridge that first
constructed it.

The verified readable compiler is retained as `../dist/raw0.elf`. Direct
AArch64 emission is still pending, so `../dist/core0.elf` remains the frozen
backend in the active two-stage pipeline.

The bootstrap-host target supports `host.read`, `host.write`, and `host.exit`.
They are target effects rather than canonical Raw Core primitives.
