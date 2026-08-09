# ARM64 Isolated Lab

This lab boots a tiny ARM64 Linux environment with Apple's built-in
Virtualization framework. It installs no VM application or package manager
formula and exposes no network, shared directory, clipboard, USB, or graphics
device to the guest.

현재 기본 구성은 약 45MB이며 영구 디스크가 없습니다. VM을 종료하면 게스트에서
변경한 내용은 사라지고, 실행하지 않을 때 CPU와 메모리를 사용하지 않습니다.

## Lifecycle

```sh
./setup.sh
./run.sh
```

Inside the guest, use `poweroff -f` to stop it. Memory and CPU are allocated
only while `run.sh` is active. The initial profile has no writable persistent
disk, so guest changes disappear at shutdown.

부팅 중 `Mounting boot media failed` 다음에 나타나는 recovery shell은 이 구성에서는
정상입니다. 설치 디스크 없이 initramfs 셸만 사용하는 의도된 상태입니다.

To enable persistence only when needed:

```sh
./create-workspace.sh
./run.sh
```

The workspace has a 256 MB logical capacity but is sparse, so it initially
uses only a few megabytes of physical host storage. In the guest recovery
shell, mount it with:

```sh
mkdir -p /work
mount -t vfat /dev/vda1 /work
```

## Host footprint

`setup.sh` verifies an official Alpine ARM64 ISO, extracts only its kernel and
initramfs, deletes the temporary ISO, and builds a small local launcher.
Everything stays inside this directory. Delete this directory to remove the
lab completely.
