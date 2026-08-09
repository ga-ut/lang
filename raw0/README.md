# Raw Core 0.4 bridge

`compiler.core` is the first readable-source bridge written in the frozen,
self-hosted Core-0 recovery language. It validates Raw Core 0.4 and emits
deterministic Core-0 source. The retained Core-0 compiler performs AArch64 ELF
lowering:

```text
Raw Core source -> raw0 bridge -> validated Core-0 source -> core0 compiler -> AArch64 ELF
```

This is an intermediate bootstrap gate, not the final readable self-hosted
compiler. The bridge must next be rewritten in Raw Core, rebuilt through two
native generations, and compared byte for byte.

The verified native bridge is retained as `../dist/raw0.elf` so the readable
language can be used before that final gate.

The bootstrap-host target supports `host.read`, `host.write`, and `host.exit`.
They are target effects rather than canonical Raw Core primitives.
