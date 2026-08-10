# Gaut OS contract

## Current machine

- architecture: AArch64
- reference board: QEMU `virt`
- entry privilege: EL1 from QEMU direct-kernel boot
- RAM: 128 MiB beginning at `0x40000000`
- console: PL011 UART at `0x09000000`
- build input: checked request records over the PL011 serial channel
- network device: absent
- persistent disk: absent

QEMU is a replaceable hardware model, not part of Gaut semantics.

## Current compile-and-run flow

`dist/gaut-os.img` is the readable Gaut compiler built with external profile
`2`. `os/run.sh` sends consecutive profile `3` child requests through PL011
serial input and terminates the bounded session with sixteen zero bytes. Each
request contains its profile ID, eight-byte little-endian source length, and
exact source bytes, with no inter-record padding.

The resident compiler:

1. emits `gaut-os: ready`;
2. receives one complete serial request into its fixed source region;
3. reports the accepted request number;
4. parses Gaut and emits a raw AArch64 child image into its output region;
5. uses `platform.run(address)` to call the child entry;
6. restores its own runtime and repeats from the ready marker; and
7. on the zero terminator, requests
   machine shutdown through PSCI `SYSTEM_OFF`.

The child acceptance programs are `os/examples/child1.gaut` and
`os/examples/child2.gaut`. A complete boot must emit exactly:

```text
gaut-os: ready
gaut-os: received 1
gaut-os: child 1
gaut-os: ready
gaut-os: received 2
gaut-os: child 2
gaut-os: ready
```

The guest path contains no Linux, libc, dynamic loader, Python, C, Rust, LLVM,
assembler, linker, or foreign runtime.

The QEMU process must exit by itself with status zero after the ready marker.
Manual interruption, a surviving emulator process, or measurable idle CPU use
after completion fails this contract.

## Resident memory contract

- the profile `2` supervisor owns `0x47f00000` through `0x47ffffff`;
- a profile `3` child owns `0x47e00000` through `0x47efffff`;
- the supervisor output image remains in the supervisor region while executing;
- `platform.run` preserves the supervisor stack, local base, evaluation stack,
  return stack, and request cursor across the child call; and
- one child finishes before the next image replaces the output buffer.

The two regions prevent ordinary Gaut locals and fixed memory in a child from
overwriting compiler state. There is still no MMU, fault containment,
persistent workspace, or malicious-child isolation.

## Low-power wait contract

Profile `2` configures the PL011 receive and receive-timeout interrupts through
the QEMU `virt` GICv2 interface. `host.read` checks for a byte, clears a stale
interrupt, checks again, and executes `WFI` only while the receive FIFO remains
empty. Serial arrival wakes the CPU and the same canonical read continues.

A delayed-input acceptance run must remain at `gaut-os: ready` with no
measurable QEMU CPU use, then accept a request, run its child, return to ready,
and shut down through the existing zero terminator.

## Current limitation and next acceptance contract

A malformed request, compile failure, or child fault still does not recover to
the resident loop. The next implementation adds recoverable supervisor errors
and child exception reporting while preserving the low-power wait and memory
contracts. No editor, file system, scheduler, MMU, or optimizer is added before
rejection, fault reporting, and a following successful request are proven in
one boot.

## Later kernel boundary

After the resident loop, Gaut OS adds explicit exception vectors, EL0 task
isolation, an MMU-backed page allocator, timer interrupts, scheduling,
persistent storage, rendering, networking, and higher-level services in that
order as concrete programs demand them.
