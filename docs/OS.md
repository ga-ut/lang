# Gaut OS contract

## Current machine

- architecture: AArch64
- reference board: QEMU `virt`
- entry privilege: EL1 from QEMU direct-kernel boot
- RAM: 128 MiB beginning at `0x40000000`
- console: PL011 UART at `0x09000000`
- build input: checked request records over the PL011 serial channel
- network device: absent
- persistent workspace: optional 64 MiB backing file on the second CFI flash
  bank beginning at `0x04000000`
- general disk and filesystem: absent

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

1. clears its bounded RAM catalog and scans valid persistent records into it;
2. emits `gaut-os: ready`;
3. receives one complete serial request into its fixed source region;
4. reports the received request number;
5. parses Gaut and emits a raw AArch64 child image into its output region;
6. calls the Gaut module function `platform.run(address)`, whose QEMU adapter
   wraps the lower `call_image(address)` operation;
7. restores its own runtime and repeats from the ready marker; and
8. on the zero terminator, requests
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
The adapter accepts only a nonempty, at-most-262144-byte payload aligned to the
compiler's 4096-byte artifact frame. A Gaut diagnostic between ready markers is
therefore a failed build and never replaces the requested output file; the host
does not reinterpret the diagnostic or decide source validity.

A command `4` request compiles the same ordinary profile `3` Gaut source and
observes the word returned through `platform.run`. Gaut OS itself writes
`gaut-os: test passed` for zero and `gaut-os: test failed` for every nonzero
word. `test` and `assert` are not Gaut keywords, no test runtime is linked into
the child, and the host does not decide the result.

A command `5` request has the same source-unit payload as command `1` and is
valid only for profile `2`. The current supervisor retains the generated
compiler instead of writing it to serial, copies it into the generation arena,
and transfers control to it. The host supplies the same checked request once
more. The generated compiler builds the following generation into its ordinary
output buffer, calls `verification.images_differ`, and reports zero or nonzero
through the command `4` result protocol. The host neither extracts the two
images nor compares or hashes them to decide success.

A command `6` request contains only the normal profile, length, and command
words. Gaut OS walks every occupied named-source catalog entry in deterministic
physical order, loads one payload at a time into a reusable scratch region, and
reconstructs the same unpadded command `1` record accepted by `compile_request`.
Profile `3` executes and judges the stored workspace through
the command `4` result protocol. Profile `2` retains and judges compiler
generations through the command `5` fixed-point path. The second profile `2`
workspace request is 24 bytes and reuses the resident source inventory instead
of receiving the source records again.

The QEMU process must exit by itself with status zero after the ready marker.
Manual interruption, a surviving emulator process, or measurable idle CPU use
after completion fails this contract.

## Resident memory contract

- the profile `2` supervisor owns `0x47f00000` through `0x47ffffff`;
- a profile `3` child owns `0x47e00000` through `0x47efffff`;
- a retained profile `2` generation owns `0x47d00000` through `0x47dfffff`;
- the supervisor output image remains in the supervisor region while executing;
- `call_image` preserves the supervisor stack, local base, evaluation stack,
  return stack, and request cursor across the child call, and returns the
  child's `main()` word through the Gaut `platform.run` module function; and
- one child finishes before the next image replaces the output buffer.

The fixed-point handoff copies at most 262144 image bytes to `0x47d00000`.
The retained byte size and one stage word live immediately after that maximum
image region at `0x47d40000` and `0x47d40008`. The generated compiler executes
from the generation arena, reuses the fixed profile `2` supervisor arena, and
builds its following generation at `0x47f30000`. After comparing both sizes and
every byte it remains the resident supervisor. The stage word stays set for the
bounded boot, so another fixed-point request compares against the active
retained generation instead of overwriting its code.

The QEMU `platform` module owns the generation base and maximum image size as
ordinary zero-argument Gaut functions. `fixed_point` calls each once before any
copy or comparison loop, stores the returned words in locals, and derives the
metadata addresses from them. This keeps the machine layout behind the platform
boundary without global storage, new syntax, or per-byte function calls.

These arenas prevent ordinary Gaut locals and fixed memory in a child from
overwriting compiler state. There is still no MMU, fault containment, or
malicious-child isolation.

## Named source storage contract

The supervisor owns an 8192-byte RAM catalog with capacity for 64 fixed
112-byte entries. An occupied entry retains a name of at most 64 bytes, source
length, physical record index, format version, and checksum. The corresponding
source payload does not remain resident: one reusable 32768-byte region holds a
record only while it is validated, stored, selected, or appended to a build.
This replaces the previous ten simultaneously resident payloads without adding
dynamic allocation.

`store` fills the first empty catalog entry or replaces the entry with the same
name and synchronizes only that physical record before reporting success. `run`
loads the selected record and reconstructs one ordinary profile `3` build
request. A workspace request streams all occupied records into the existing
131072-byte checked request region and uses the same parser and emitter as an
immediate build. Because records are unpadded, their two request length words
are written as little-endian bytes and may begin at unaligned addresses.
The scratch transfer contract places the physical record index in the word
immediately following its 32768-byte payload, preserving the existing
one-argument platform load/save surface.

Compile rejection restarts the resident entry and reconstructs the catalog by
scanning flash, so a valid stored source remains runnable after an invalid one
is rejected. Each catalog location owns one 256 KiB flash sector. The first
4096 bytes contain a versioned header with the physical index, exact
32768-byte payload size, and FNV-1a checksum; the remaining payload begins at
offset 4096. Gaut erases the sector, writes the eight payload blocks, and
commits the header block last. A missing, partial, wrongly indexed,
out-of-bounds, or checksum-mismatched record is absent from the next catalog.
Physical sector order is workspace order.

`os/boot.sh` creates or attaches the exact 64 MiB backing file selected by
`GAUT_WORKSPACE_FLASH` and maps it as QEMU `pflash1`. The host does not parse,
repair, select, or judge source records. There is no allocator, general disk
format, filesystem, automatic dependency search, or second compiler path.

## Low-power wait contract

Profile `2` configures the PL011 receive and receive-timeout interrupts through
the QEMU `virt` GICv2 interface. `host.read` checks for a byte, clears a stale
interrupt, checks again, and executes `WFI` only while the receive FIFO remains
empty. Serial arrival wakes the CPU and the same canonical read continues.

A delayed-input acceptance run must remain at `gaut-os: ready` with no
measurable QEMU CPU use, then accept a request, run its child, return to ready,
and shut down through the existing zero terminator.

## Current limitations

A malformed serial length that cannot be safely framed and a child hardware
fault do not yet recover. Persistent records are deliberately capped at 64,
each source remains capped at 32680 bytes, and the selected workspace must fit
the existing 131072-byte request region. This catalog is not a filesystem. The
current CFI adapter assumes the documented QEMU `virt` flash address, bank
width, sector size, and buffered-write contract.

## Later kernel boundary

After the current compiler optimization gate, Gaut OS adds explicit exception
vectors, EL0 task isolation, an MMU-backed page allocator, timer interrupts,
scheduling, rendering, networking, and higher-level services in that order as
concrete programs demand them.
