import CoreWLAN
import Foundation

private struct AssociationIntent: Decodable {
    let schema_version: String
    let operation: String
    let interface_name: String
    let configuration_candidate: String
    let fixture_status: String?
}

private struct AssociationResult: Encodable {
    let schema_version = "bitaxe-macos-provisioning-association-result-v1"
    let status: String
    let error_domain: String?
    let error_code: Int?
}

private func writeResult(_ result: AssociationResult, to url: URL) {
    do {
        let data = try JSONEncoder().encode(result)
        try data.write(to: url, options: [.atomic])
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    } catch {
        exit(74)
    }
}

private func result(_ status: String, error: NSError? = nil) -> AssociationResult {
    AssociationResult(
        status: status,
        error_domain: error?.domain,
        error_code: error?.code
    )
}

private func candidateIsValid(_ value: String) -> Bool {
    value.range(of: #"^Bitaxe_[0-9A-F]{4}$"#, options: .regularExpression) != nil
}

let arguments = CommandLine.arguments
guard arguments.count == 5, arguments[1] == "--intent", arguments[3] == "--result" else {
    exit(64)
}

private let intentURL = URL(fileURLWithPath: arguments[2])
private let resultURL = URL(fileURLWithPath: arguments[4])
private let intent: AssociationIntent
do {
    intent = try JSONDecoder().decode(AssociationIntent.self, from: Data(contentsOf: intentURL))
} catch {
    writeResult(result("input_invalid", error: error as NSError), to: resultURL)
    exit(0)
}

guard intent.schema_version == "bitaxe-macos-provisioning-association-intent-v1",
      candidateIsValid(intent.configuration_candidate),
      !intent.interface_name.isEmpty
else {
    writeResult(result("input_invalid"), to: resultURL)
    exit(0)
}

if intent.operation == "fixture" {
    let allowed = [
        "ready", "input_invalid", "interface_unavailable", "directed_scan_failed",
        "candidate_absent", "candidate_ambiguous", "association_rejected",
        "association_not_running",
    ]
    let fixtureStatus = intent.fixture_status ?? "ready"
    writeResult(
        result(allowed.contains(fixtureStatus) ? fixtureStatus : "input_invalid"),
        to: resultURL
    )
    exit(0)
}

guard intent.operation == "associate" else {
    writeResult(result("input_invalid"), to: resultURL)
    exit(0)
}

guard let interface = CWWiFiClient.shared().interface(withName: intent.interface_name) else {
    writeResult(result("interface_unavailable"), to: resultURL)
    exit(0)
}

let networks: Set<CWNetwork>
do {
    guard let candidateData = intent.configuration_candidate.data(using: .utf8) else {
        writeResult(result("input_invalid"), to: resultURL)
        exit(0)
    }
    networks = try interface.scanForNetworks(withSSID: candidateData, includeHidden: true)
} catch {
    writeResult(result("directed_scan_failed", error: error as NSError), to: resultURL)
    exit(0)
}

guard networks.count == 1, let network = networks.first else {
    writeResult(result(networks.isEmpty ? "candidate_absent" : "candidate_ambiguous"), to: resultURL)
    exit(0)
}

do {
    try interface.associate(to: network, password: nil)
} catch {
    writeResult(result("association_rejected", error: error as NSError), to: resultURL)
    exit(0)
}

guard interface.interfaceMode() != .none else {
    writeResult(result("association_not_running"), to: resultURL)
    exit(0)
}

writeResult(result("ready"), to: resultURL)
