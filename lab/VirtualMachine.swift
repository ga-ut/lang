import Foundation
import Virtualization

final class Delegate: NSObject, VZVirtualMachineDelegate {
    func virtualMachine(_ virtualMachine: VZVirtualMachine, didStopWithError error: Error) {
        FileHandle.standardError.write(Data("\nVM stopped with error: \(error)\n".utf8))
        exit(1)
    }

    func guestDidStop(_ virtualMachine: VZVirtualMachine) {
        FileHandle.standardError.write(Data("\nVM stopped.\n".utf8))
        exit(0)
    }
}

let runtimeDirectory = URL(fileURLWithPath: CommandLine.arguments[0])
    .deletingLastPathComponent()
    .standardizedFileURL
let kernelURL = runtimeDirectory.appendingPathComponent("Image")
let initrdURL = runtimeDirectory.appendingPathComponent("initramfs-virt")

guard FileManager.default.fileExists(atPath: kernelURL.path),
      FileManager.default.fileExists(atPath: initrdURL.path) else {
    FileHandle.standardError.write(Data("Run ./setup.sh first.\n".utf8))
    exit(2)
}

let bootLoader = VZLinuxBootLoader(kernelURL: kernelURL)
bootLoader.initialRamdiskURL = initrdURL
bootLoader.commandLine = [
    "console=hvc0",
    "modules=virtio_pci,virtio_console,virtio_blk,fat,vfat,nls_cp437",
    "nomodeset",
    "quiet"
].joined(separator: " ")

let console = VZVirtioConsoleDeviceSerialPortConfiguration()
console.attachment = VZFileHandleSerialPortAttachment(
    fileHandleForReading: .standardInput,
    fileHandleForWriting: .standardOutput
)

let configuration = VZVirtualMachineConfiguration()
configuration.bootLoader = bootLoader
configuration.cpuCount = 2
configuration.memorySize = 2 * 1024 * 1024 * 1024
configuration.serialPorts = [console]
configuration.entropyDevices = [VZVirtioEntropyDeviceConfiguration()]
configuration.memoryBalloonDevices = [VZVirtioTraditionalMemoryBalloonDeviceConfiguration()]
configuration.networkDevices = []

let workspaceURL = runtimeDirectory.appendingPathComponent("workspace.img")
if FileManager.default.fileExists(atPath: workspaceURL.path) {
    do {
        let attachment = try VZDiskImageStorageDeviceAttachment(
            url: workspaceURL,
            readOnly: false,
            cachingMode: .cached,
            synchronizationMode: .full
        )
        let workspace = VZVirtioBlockDeviceConfiguration(attachment: attachment)
        workspace.blockDeviceIdentifier = "OurWorkspace"
        configuration.storageDevices = [workspace]
    } catch {
        FileHandle.standardError.write(Data("Unable to attach workspace: \(error)\n".utf8))
        exit(3)
    }
} else {
    configuration.storageDevices = []
}

do {
    try configuration.validate()
} catch {
    FileHandle.standardError.write(Data("Invalid VM configuration: \(error)\n".utf8))
    exit(3)
}

let delegate = Delegate()
let machine = VZVirtualMachine(configuration: configuration)
machine.delegate = delegate

machine.start { result in
    if case .failure(let error) = result {
        FileHandle.standardError.write(Data("Unable to start VM: \(error)\n".utf8))
        exit(4)
    }
}

dispatchMain()
