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
request contains its profile ID, remaining payload length, command ID, and
command payload. An immediate build payload contains the source-unit count and
unpadded records of module-name length, source length, name bytes, and source
bytes. The retained sixteen-byte serial header therefore still tells the
low-power receiver exactly how many additional bytes to read.

The resident compiler:

1. emits `gaut-os: ready`;
2. receives one complete serial request into its fixed source region;
3. reports the received request number;
4. parses Gaut and emits a raw AArch64 child image into its output region;
5. calls the Gaut module function `platform.run(address)`, whose QEMU adapter
   wraps the lower `call_image(address)` operation;
6. restores its own runtime and repeats from the ready marker; and
7. on the zero terminator, requests
   machine shutdown through PSCI `SYSTEM_OFF`.

For a complete but invalid Gaut source request, `fail` writes the same canonical
21-byte diagnostic used by the hosted compiler and calls `host.exit(2)`. The
profile `2` adapter abandons the current call stack, branches to its preserved
entry anchor, rebuilds the compiler runtime registers, and emits `ready` again.
No second parser or diagnostic path exists for resident compilation.

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

`os/build.sh` sends a profile `1` or `2` compiler request to the resident Gaut
compiler and retains the artifact framed between its two ready markers. This
lets Gaut OS rebuild the split compiler through a byte-identical fixed point
using QEMU without booting a Linux guest.

The QEMU process must exit by itself with status zero after the ready marker.
Manual interruption, a surviving emulator process, or measurable idle CPU use
after completion fails this contract.

## Resident memory contract

- the profile `2` supervisor owns `0x47f00000` through `0x47ffffff`;
- a profile `3` child owns `0x47e00000` through `0x47efffff`;
- the supervisor output image remains in the supervisor region while executing;
- `call_image` preserves the supervisor stack, local base, evaluation stack,
  return stack, and request cursor across the child call, and returns the
  child's `main()` word through the Gaut `platform.run` module function; and
- one child finishes before the next image replaces the output buffer.

The two regions prevent ordinary Gaut locals and fixed memory in a child from
overwriting compiler state. There is still no MMU, fault containment,
persistent source storage, or malicious-child isolation.

## Named source storage contract

The supervisor owns eight fixed 32768-byte source slots inside its existing
1 MiB runtime arena. A slot stores one name of at most 64 bytes and one source
of at most 32680 bytes. `store` fills the first empty slot or replaces the slot
with the same name; `run` reconstructs one ordinary profile `3` build request
and uses the same parser and emitter as immediate execution.

The table is not cleared when compile rejection returns to the resident entry,
so another stored source remains runnable after an invalid stored source is
rejected. The table is volatile and disappears when QEMU shuts down. There is
no allocator, disk format, automatic dependency search, or second compiler
path.

## Low-power wait contract

Profile `2` configures the PL011 receive and receive-timeout interrupts through
the QEMU `virt` GICv2 interface. `host.read` checks for a byte, clears a stale
interrupt, checks again, and executes `WFI` only while the receive FIFO remains
empty. Serial arrival wakes the CPU and the same canonical read continues.

A delayed-input acceptance run must remain at `gaut-os: ready` with no
measurable QEMU CPU use, then accept a request, run its child, return to ready,
and shut down through the existing zero terminator.

## Current limitation and next acceptance contract

Named sources do not survive shutdown, and one stored name currently contains
one source unit. A malformed length that cannot be safely framed and a child
hardware fault also do not yet recover.

The next implementation adds a Gaut-native test result protocol. It observes
the word already returned through `platform.run`, treats zero as pass and a
nonzero word as failure, and emits one deterministic supervisor result record.
It adds no `assert` keyword, second compiler path, editor, scheduler, MMU, or
optimizer.

## Later kernel boundary

After the native test result protocol, Gaut OS adds persistent source storage,
explicit exception vectors, EL0 task isolation, an MMU-backed page allocator,
timer interrupts, scheduling, rendering, networking, and higher-level services
in that order as concrete programs demand them.
