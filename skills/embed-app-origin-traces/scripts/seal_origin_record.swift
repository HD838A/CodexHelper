#!/usr/bin/env swift

import CryptoKit
import Foundation

private let generatorVersion = "1.0.0"

private struct OriginRecord: Codable, Equatable {
    let schemaVersion: Int
    let repository: String
    let bundleIdentifier: String
    let commit: String
    let appName: String
    let version: String
    let build: String
    let originID: String
    let issuedAt: String
}

private struct GeneratedPayload: Codable, Equatable {
    let schemaVersion: Int
    let algorithm: String
    let sealedBoxBase64: String
    let keyFragmentsBase64: [String]
    let recordSHA256: String
}

private struct IntegrationPoint: Codable, Equatable {
    let path: String
    let anchor: String
    let role: String
    let fileSHA256: String
}

private struct EvidenceManifest: Codable, Equatable {
    let schemaVersion: Int
    let generatorVersion: String
    let record: OriginRecord
    let triggerTokens: [String]
    let payload: GeneratedPayload
    var integrationPoints: [IntegrationPoint]
    var artifactSHA256: String?
}

private enum ToolError: Error, CustomStringConvertible {
    case usage(String)
    case invalid(String)
    case io(String)
    case verification(String)

    var description: String {
        switch self {
        case let .usage(message), let .invalid(message), let .io(message), let .verification(message):
            return message
        }
    }
}

private struct Arguments {
    private var values: [String: String] = [:]
    private var flags: Set<String> = []

    init(_ raw: ArraySlice<String>) throws {
        var index = raw.startIndex
        while index < raw.endIndex {
            let key = raw[index]
            guard key.hasPrefix("--") else {
                throw ToolError.usage("unexpected argument: \(key)")
            }
            let next = raw.index(after: index)
            if key == "--force" {
                flags.insert(key)
                index = next
                continue
            }
            guard next < raw.endIndex else {
                throw ToolError.usage("missing value for \(key)")
            }
            let value = raw[next]
            guard !value.hasPrefix("--") else {
                throw ToolError.usage("missing value for \(key)")
            }
            guard values[key] == nil else {
                throw ToolError.usage("duplicate argument: \(key)")
            }
            values[key] = value
            index = raw.index(after: next)
        }
    }

    func required(_ key: String) throws -> String {
        guard let value = values[key], !value.isEmpty else {
            throw ToolError.usage("missing required argument: \(key)")
        }
        return value
    }

    func optional(_ key: String, default defaultValue: String = "") -> String {
        values[key] ?? defaultValue
    }

    func contains(_ flag: String) -> Bool {
        flags.contains(flag)
    }
}

private func usage() -> String {
    """
    usage:
      seal_origin_record.swift generate \\
        --repository <https-url> --bundle-id <identifier> --commit <full-hash> \\
        --app-name <name> --version <version> --build <build> \\
        --trigger <comma-separated-tokens> --payload-output <path> \\
        --manifest-output <path> [--origin-id <uuid>] \\
        [--fragment-count <2-8>] [--force]

      seal_origin_record.swift verify --payload <path> --manifest <path>
    """
}

private func makeEncoder() -> JSONEncoder {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]
    return encoder
}

private func randomData(count: Int) -> Data {
    let key = SymmetricKey(size: .bits256)
    return key.withUnsafeBytes { Data($0.prefix(count)) }
}

private func sha256Hex(_ data: Data) -> String {
    SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
}

private func splitKey(_ key: Data, count: Int) throws -> [Data] {
    guard key.count == 32 else {
        throw ToolError.invalid("AES key must contain exactly 32 bytes")
    }
    guard (2...8).contains(count) else {
        throw ToolError.invalid("fragment count must be between 2 and 8")
    }

    var fragments: [Data] = []
    var accumulator = [UInt8](repeating: 0, count: key.count)
    for _ in 0..<(count - 1) {
        let fragment = randomData(count: key.count)
        for (index, byte) in fragment.enumerated() {
            accumulator[index] ^= byte
        }
        fragments.append(fragment)
    }

    let keyBytes = [UInt8](key)
    let finalBytes = zip(keyBytes, accumulator).map(^)
    fragments.append(Data(finalBytes))
    return fragments
}

private func reconstructKey(_ fragments: [Data]) throws -> Data {
    guard fragments.count >= 2, fragments.allSatisfy({ $0.count == 32 }) else {
        throw ToolError.verification("every key fragment must contain exactly 32 bytes")
    }
    var key = [UInt8](repeating: 0, count: 32)
    for fragment in fragments {
        for (index, byte) in fragment.enumerated() {
            key[index] ^= byte
        }
    }
    return Data(key)
}

private func parseTrigger(_ value: String) throws -> [String] {
    let tokens = value.split(separator: ",", omittingEmptySubsequences: false).map {
        $0.trimmingCharacters(in: .whitespacesAndNewlines)
    }
    guard (3...12).contains(tokens.count), tokens.allSatisfy({ !$0.isEmpty && $0.count <= 64 }) else {
        throw ToolError.invalid("trigger must contain 3 to 12 non-empty comma-separated tokens")
    }
    return tokens
}

private func validateRepository(_ value: String) throws {
    guard let url = URL(string: value),
          url.scheme == "https",
          url.host != nil,
          url.user == nil,
          url.password == nil
    else {
        throw ToolError.invalid("repository must be an absolute HTTPS URL without credentials")
    }
}

private func validateCommit(_ value: String) throws {
    let pattern = #"^[0-9a-fA-F]{40,64}$"#
    guard value.range(of: pattern, options: .regularExpression) != nil else {
        throw ToolError.invalid("commit must be a full 40 to 64 character hexadecimal hash")
    }
}

private func validatedOutputURL(_ path: String, force: Bool) throws -> URL {
    let url = URL(fileURLWithPath: path).standardizedFileURL
    let parent = url.deletingLastPathComponent()
    var isDirectory: ObjCBool = false
    guard FileManager.default.fileExists(atPath: parent.path, isDirectory: &isDirectory), isDirectory.boolValue else {
        throw ToolError.io("output directory does not exist: \(parent.path)")
    }
    if !force, FileManager.default.fileExists(atPath: url.path) {
        throw ToolError.io("refusing to overwrite existing output: \(url.path); pass --force to replace it")
    }
    return url
}

private func writePrivate(_ data: Data, to url: URL) throws {
    do {
        try data.write(to: url, options: .atomic)
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    } catch {
        throw ToolError.io("failed to write \(url.path): \(error)")
    }
}

private func readJSON<T: Decodable>(_ type: T.Type, from path: String) throws -> T {
    do {
        let data = try Data(contentsOf: URL(fileURLWithPath: path))
        return try JSONDecoder().decode(type, from: data)
    } catch {
        throw ToolError.io("failed to read \(path): \(error)")
    }
}

private func generate(_ rawArguments: ArraySlice<String>) throws {
    let arguments = try Arguments(rawArguments)
    let repository = try arguments.required("--repository")
    let bundleIdentifier = try arguments.required("--bundle-id")
    let commit = try arguments.required("--commit")
    let appName = try arguments.required("--app-name")
    let version = try arguments.required("--version")
    let build = try arguments.required("--build")
    let triggerTokens = try parseTrigger(try arguments.required("--trigger"))
    let force = arguments.contains("--force")
    let payloadURL = try validatedOutputURL(try arguments.required("--payload-output"), force: force)
    let manifestURL = try validatedOutputURL(try arguments.required("--manifest-output"), force: force)
    guard payloadURL != manifestURL else {
        throw ToolError.invalid("payload and manifest outputs must be different files")
    }

    try validateRepository(repository)
    try validateCommit(commit)
    guard !bundleIdentifier.isEmpty, bundleIdentifier.count <= 255, !bundleIdentifier.contains(where: { $0.isWhitespace }) else {
        throw ToolError.invalid("bundle identifier is empty or invalid")
    }
    guard !appName.isEmpty, appName.count <= 128 else {
        throw ToolError.invalid("app name must contain 1 to 128 characters")
    }
    guard version.count <= 64, build.count <= 64 else {
        throw ToolError.invalid("version and build must be 64 characters or fewer")
    }

    let originID: String
    let requestedOriginID = arguments.optional("--origin-id")
    if requestedOriginID.isEmpty {
        originID = UUID().uuidString.lowercased()
    } else if UUID(uuidString: requestedOriginID) != nil {
        originID = requestedOriginID.lowercased()
    } else {
        throw ToolError.invalid("origin ID must be a UUID")
    }

    let fragmentCountString = arguments.optional("--fragment-count", default: "3")
    guard let fragmentCount = Int(fragmentCountString), (2...8).contains(fragmentCount) else {
        throw ToolError.invalid("fragment count must be an integer between 2 and 8")
    }

    let record = OriginRecord(
        schemaVersion: 1,
        repository: repository,
        bundleIdentifier: bundleIdentifier,
        commit: commit.lowercased(),
        appName: appName,
        version: version,
        build: build,
        originID: originID,
        issuedAt: ISO8601DateFormatter().string(from: Date())
    )
    let encoder = makeEncoder()
    let recordData = try encoder.encode(record)
    let keyData = randomData(count: 32)
    let key = SymmetricKey(data: keyData)
    let sealedBox = try AES.GCM.seal(recordData, using: key)
    guard let combined = sealedBox.combined else {
        throw ToolError.invalid("CryptoKit did not return a combined AES-GCM sealed box")
    }
    let fragments = try splitKey(keyData, count: fragmentCount)
    let payload = GeneratedPayload(
        schemaVersion: 1,
        algorithm: "AES-256-GCM/XOR-SHARES",
        sealedBoxBase64: combined.base64EncodedString(),
        keyFragmentsBase64: fragments.map { $0.base64EncodedString() },
        recordSHA256: sha256Hex(recordData)
    )
    let manifest = EvidenceManifest(
        schemaVersion: 1,
        generatorVersion: generatorVersion,
        record: record,
        triggerTokens: triggerTokens,
        payload: payload,
        integrationPoints: [],
        artifactSHA256: nil
    )

    try writePrivate(try encoder.encode(payload), to: payloadURL)
    try writePrivate(try encoder.encode(manifest), to: manifestURL)
    print("generated origin payload: \(payloadURL.path)")
    print("generated private manifest: \(manifestURL.path)")
    print("origin ID: \(originID)")
}

private func verify(_ rawArguments: ArraySlice<String>) throws {
    let arguments = try Arguments(rawArguments)
    let payloadPath = try arguments.required("--payload")
    let manifestPath = try arguments.required("--manifest")
    let payload = try readJSON(GeneratedPayload.self, from: payloadPath)
    let manifest = try readJSON(EvidenceManifest.self, from: manifestPath)

    guard payload == manifest.payload else {
        throw ToolError.verification("payload does not match the private manifest")
    }
    guard payload.schemaVersion == 1, manifest.schemaVersion == 1 else {
        throw ToolError.verification("unsupported schema version")
    }
    guard payload.algorithm == "AES-256-GCM/XOR-SHARES" else {
        throw ToolError.verification("unsupported payload algorithm")
    }
    try validateRepository(manifest.record.repository)
    try validateCommit(manifest.record.commit)
    guard (3...12).contains(manifest.triggerTokens.count) else {
        throw ToolError.verification("private manifest contains an invalid trigger")
    }

    guard let combined = Data(base64Encoded: payload.sealedBoxBase64) else {
        throw ToolError.verification("sealed box is not valid Base64")
    }
    let fragments = try payload.keyFragmentsBase64.map { value -> Data in
        guard let data = Data(base64Encoded: value) else {
            throw ToolError.verification("key fragment is not valid Base64")
        }
        return data
    }
    let keyData = try reconstructKey(fragments)
    let sealedBox = try AES.GCM.SealedBox(combined: combined)
    let opened = try AES.GCM.open(sealedBox, using: SymmetricKey(data: keyData))
    guard sha256Hex(opened) == payload.recordSHA256 else {
        throw ToolError.verification("decrypted record hash does not match")
    }
    let decoded = try JSONDecoder().decode(OriginRecord.self, from: opened)
    guard decoded == manifest.record else {
        throw ToolError.verification("decrypted record does not match the private manifest")
    }

    print("origin payload verification passed")
    print("origin ID: \(decoded.originID)")
}

do {
    let arguments = CommandLine.arguments.dropFirst()
    guard let command = arguments.first else {
        throw ToolError.usage(usage())
    }
    switch command {
    case "generate":
        try generate(arguments.dropFirst())
    case "verify":
        try verify(arguments.dropFirst())
    case "help", "--help", "-h":
        print(usage())
    default:
        throw ToolError.usage("unknown command: \(command)\n\n\(usage())")
    }
} catch {
    FileHandle.standardError.write(Data("error: \(error)\n".utf8))
    exit(1)
}
