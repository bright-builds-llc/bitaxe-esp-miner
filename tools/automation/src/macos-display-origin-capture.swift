import AppKit
import Foundation

private struct CaptureIntent: Decodable {
    let schema_version: String
    let operation: String
    let generation: Int
    let fixture_status: String?
    let fixture_ipv4: String?
}

private struct CaptureResult: Encodable {
    let schema_version = "bitaxe-native-usb-display-origin-capture-v1"
    let generation: Int
    let status: String
    let ipv4: String
}

private func writeResult(_ result: CaptureResult, to url: URL) {
    do {
        let data = try JSONEncoder().encode(result)
        try data.write(to: url, options: [.atomic])
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    } catch {
        exit(74)
    }
}

private func privateIPv4(_ value: String) -> Bool {
    let parts = value.split(separator: ".", omittingEmptySubsequences: false)
    guard parts.count == 4 else { return false }
    let numbers = parts.compactMap { part -> Int? in
        guard !part.isEmpty, String(Int(part) ?? -1) == part else { return nil }
        return Int(part)
    }
    guard numbers.count == 4, numbers.allSatisfy({ (0...255).contains($0) }) else { return false }
    let a = numbers[0]
    let b = numbers[1]
    let d = numbers[3]
    return d > 0 && d < 255
        && (a == 10 || (a == 172 && (16...31).contains(b)) || (a == 192 && b == 168))
}

let arguments = CommandLine.arguments
guard arguments.count == 5, arguments[1] == "--intent", arguments[3] == "--result" else {
    exit(64)
}
private let intentURL = URL(fileURLWithPath: arguments[2])
private let resultURL = URL(fileURLWithPath: arguments[4])
private let intent: CaptureIntent
do {
    intent = try JSONDecoder().decode(CaptureIntent.self, from: Data(contentsOf: intentURL))
} catch {
    writeResult(CaptureResult(generation: 0, status: "input_invalid", ipv4: ""), to: resultURL)
    exit(0)
}
guard intent.schema_version == "bitaxe-native-usb-display-origin-prompt-v1",
      intent.generation == 1 || intent.generation == 2 else {
    writeResult(CaptureResult(generation: intent.generation, status: "input_invalid", ipv4: ""), to: resultURL)
    exit(0)
}
if intent.operation == "fixture" {
    let value = intent.fixture_ipv4 ?? ""
    let status = intent.fixture_status ?? "accepted"
    writeResult(
        CaptureResult(
            generation: intent.generation,
            status: status == "accepted" && privateIPv4(value) ? "accepted" : status,
            ipv4: status == "accepted" && privateIPv4(value) ? value : ""
        ),
        to: resultURL
    )
    exit(0)
}
guard intent.operation == "prompt" else {
    writeResult(CaptureResult(generation: intent.generation, status: "input_invalid", ipv4: ""), to: resultURL)
    exit(0)
}

NSApplication.shared.setActivationPolicy(.accessory)
while true {
    let alert = NSAlert()
    alert.messageText = "Bitaxe recovery address"
    alert.informativeText = "Enter the private IPv4 address currently shown on the Bitaxe display."
    alert.addButton(withTitle: "Continue")
    alert.addButton(withTitle: "Cancel")
    let field = NSTextField(frame: NSRect(x: 0, y: 0, width: 260, height: 24))
    field.placeholderString = "192.168.1.100"
    alert.accessoryView = field
    NSApplication.shared.activate(ignoringOtherApps: true)
    let response = alert.runModal()
    if response != .alertFirstButtonReturn {
        writeResult(CaptureResult(generation: intent.generation, status: "cancelled", ipv4: ""), to: resultURL)
        exit(0)
    }
    let value = field.stringValue.trimmingCharacters(in: .whitespacesAndNewlines)
    if privateIPv4(value) {
        writeResult(CaptureResult(generation: intent.generation, status: "accepted", ipv4: value), to: resultURL)
        exit(0)
    }
    let invalid = NSAlert()
    invalid.messageText = "Invalid private IPv4 address"
    invalid.informativeText = "Use the address shown on the display, without http://, a port, or a path."
    invalid.runModal()
}
