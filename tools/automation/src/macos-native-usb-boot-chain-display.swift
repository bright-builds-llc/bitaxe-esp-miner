import AppKit
import Foundation

private struct Intent: Decodable { let operation: String; let fixture_category: String? }
private struct Result: Encodable {
    let schema_version = "bitaxe-native-usb-boot-chain-display-v1"
    let status: String
    let category: String
}
private let categories = ["active_ui", "boot_or_error_text", "blank_or_dark", "frozen_or_static", "unknown"]

private func write(_ value: Result, to url: URL) {
    do {
        try JSONEncoder().encode(value).write(to: url, options: [.atomic])
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    } catch { exit(74) }
}

let args = CommandLine.arguments
guard args.count == 5, args[1] == "--intent", args[3] == "--result" else { exit(64) }
let resultURL = URL(fileURLWithPath: args[4])
guard let intent = try? JSONDecoder().decode(Intent.self, from: Data(contentsOf: URL(fileURLWithPath: args[2]))) else {
    write(Result(status: "input_invalid", category: "unknown"), to: resultURL); exit(0)
}
if intent.operation == "fixture" {
    let category = categories.contains(intent.fixture_category ?? "") ? intent.fixture_category! : "unknown"
    write(Result(status: "accepted", category: category), to: resultURL); exit(0)
}
guard intent.operation == "prompt" else { write(Result(status: "input_invalid", category: "unknown"), to: resultURL); exit(0) }

NSApplication.shared.setActivationPolicy(.accessory)
let alert = NSAlert()
alert.messageText = "Current Bitaxe display state"
alert.informativeText = "Choose the closest description before resetting the board."
let popup = NSPopUpButton(frame: NSRect(x: 0, y: 0, width: 260, height: 26))
popup.addItems(withTitles: ["Active UI", "Boot or error text", "Blank or dark", "Frozen or static", "Unknown"])
alert.accessoryView = popup
alert.addButton(withTitle: "Record")
alert.addButton(withTitle: "Cancel")
NSApplication.shared.activate(ignoringOtherApps: true)
let response = alert.runModal()
let category = categories[popup.indexOfSelectedItem]
write(Result(status: response == .alertFirstButtonReturn ? "accepted" : "cancelled", category: category), to: resultURL)
