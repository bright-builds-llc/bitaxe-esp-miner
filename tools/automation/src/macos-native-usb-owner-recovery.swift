import AppKit
import Foundation

private struct PromptIntent: Decodable {
    let schema_version: String
    let operation: String
    let fixture_status: String?
}

private struct PromptResult: Encodable {
    let schema_version = "bitaxe-native-usb-owner-recovery-checkpoint-v1"
    let action = "manual_boot_reset"
    let status: String
}

private func writeResult(_ result: PromptResult, to url: URL) {
    do {
        let data = try JSONEncoder().encode(result)
        try data.write(to: url, options: [.atomic])
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    } catch {
        exit(74)
    }
}

let arguments = CommandLine.arguments
guard arguments.count == 5, arguments[1] == "--intent", arguments[3] == "--result" else {
    exit(64)
}
private let intentURL = URL(fileURLWithPath: arguments[2])
private let resultURL = URL(fileURLWithPath: arguments[4])
private let intent: PromptIntent
do {
    intent = try JSONDecoder().decode(PromptIntent.self, from: Data(contentsOf: intentURL))
} catch {
    writeResult(PromptResult(status: "input_invalid"), to: resultURL)
    exit(0)
}
guard intent.schema_version == "bitaxe-native-usb-owner-recovery-prompt-v1" else {
    writeResult(PromptResult(status: "input_invalid"), to: resultURL)
    exit(0)
}
if intent.operation == "fixture" {
    let status = intent.fixture_status == "cancelled" ? "cancelled" : "accepted"
    writeResult(PromptResult(status: status), to: resultURL)
    exit(0)
}
guard intent.operation == "prompt" else {
    writeResult(PromptResult(status: "input_invalid"), to: resultURL)
    exit(0)
}

NSApplication.shared.setActivationPolicy(.accessory)
let alert = NSAlert()
alert.messageText = "Put the Bitaxe into ROM download mode"
alert.informativeText = "Hold BOOT, press and release RESET, then release BOOT. Click Ready only after completing that sequence."
alert.addButton(withTitle: "Ready")
alert.addButton(withTitle: "Cancel")
NSApplication.shared.activate(ignoringOtherApps: true)
let response = alert.runModal()
writeResult(
    PromptResult(status: response == .alertFirstButtonReturn ? "accepted" : "cancelled"),
    to: resultURL
)
