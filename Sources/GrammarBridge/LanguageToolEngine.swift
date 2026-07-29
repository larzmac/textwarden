//
//  LanguageToolEngine.swift
//  TextWarden
//
//  HTTP client for a locally running LanguageTool server (https://languagetool.org/dev).
//  Speaks the /v2/check API and converts results into TextWarden's GrammarErrorModel,
//  including the UTF-16 (Java/LT) → Unicode-scalar (Harper/TextWarden) offset conversion.
//
//  The server is expected at http://localhost:8081 (brew services start languagetool).
//

import Foundation

/// Engine selection persisted in UserDefaults (read by EngineRouter and settings UI)
enum GrammarEngineMode: String, CaseIterable {
    case auto
    case languageTool
    case harper

    static let defaultsKey = "grammarEngineMode"

    var displayName: String {
        switch self {
        case .auto: "Auto (LanguageTool, Harper fallback)"
        case .languageTool: "LanguageTool"
        case .harper: "Harper"
        }
    }

    static var current: GrammarEngineMode {
        GrammarEngineMode(rawValue: UserDefaults.standard.string(forKey: defaultsKey) ?? "") ?? .auto
    }
}

/// Client for a local LanguageTool HTTP server.
/// Thread-safe: designed to be called from the analysis background queue.
final class LanguageToolEngine: @unchecked Sendable {
    static let serverURLDefaultsKey = "languageToolServerURL"
    static let defaultServerURL = "http://localhost:8081"

    /// Texts above this size are declined (return nil) so the router falls back to Harper,
    /// which handles large documents locally in milliseconds. LT's API caps near this anyway.
    static let maxPayloadCharacters = 20_000

    /// After a connection failure, skip LanguageTool for this long so a downed
    /// server degrades to Harper instantly instead of paying a timeout per keystroke.
    private let failureCooldown: TimeInterval = 30

    private let session: URLSession
    private let lock = NSLock()
    private var lastFailureAt: Date?

    init() {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 4
        config.timeoutIntervalForResource = 6
        session = URLSession(configuration: config)
    }

    private var serverURL: String {
        let stored = UserDefaults.standard.string(forKey: Self.serverURLDefaultsKey)
        let trimmed = stored?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return trimmed.isEmpty ? Self.defaultServerURL : trimmed
    }

    /// True when a recent request failed and the cooldown hasn't elapsed.
    var isInFailureCooldown: Bool {
        lock.lock()
        defer { lock.unlock() }
        guard let last = lastFailureAt else { return false }
        return Date().timeIntervalSince(last) < failureCooldown
    }

    private func recordFailure() {
        lock.lock()
        lastFailureAt = Date()
        lock.unlock()
    }

    private func recordSuccess() {
        lock.lock()
        lastFailureAt = nil
        lock.unlock()
    }

    /// TextWarden dialect display name → LanguageTool language code
    static func languageCode(forDialect dialect: String) -> String {
        switch dialect {
        case "British": "en-GB"
        case "Canadian": "en-CA"
        case "Australian": "en-AU"
        default: "en-US"
        }
    }

    /// Check text against the LanguageTool server.
    /// - Returns: nil on any transport/decode failure (caller falls back to Harper).
    func check(_ text: String, dialect: String) -> GrammarAnalysisResult? {
        guard !text.isEmpty else {
            return GrammarAnalysisResult(errors: [], wordCount: 0, analysisTimeMs: 0)
        }
        guard text.count <= Self.maxPayloadCharacters else {
            Logger.info("LanguageTool: text too large (\(text.count) chars > \(Self.maxPayloadCharacters)) - deferring to Harper", category: Logger.analysis)
            return nil
        }
        guard let url = URL(string: serverURL + "/v2/check") else { return nil }

        let started = DispatchTime.now()

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded; charset=utf-8", forHTTPHeaderField: "Content-Type")
        request.httpBody = Self.formEncode([
            ("text", text),
            ("language", Self.languageCode(forDialect: dialect)),
        ])

        var responseData: Data?
        var succeeded = false
        let semaphore = DispatchSemaphore(value: 0)
        let task = session.dataTask(with: request) { data, response, error in
            defer { semaphore.signal() }
            guard error == nil,
                  let http = response as? HTTPURLResponse, http.statusCode == 200,
                  let data else { return }
            responseData = data
            succeeded = true
        }
        task.resume()
        semaphore.wait()

        guard succeeded, let data = responseData,
              let decoded = try? JSONDecoder().decode(LTResponse.self, from: data)
        else {
            recordFailure()
            Logger.warning("LanguageTool request failed (server: \(serverURL)) - entering cooldown", category: Logger.analysis)
            return nil
        }
        recordSuccess()

        let errors = decoded.matches.compactMap { Self.errorModel(from: $0, in: text) }
        let elapsedMs = UInt64((DispatchTime.now().uptimeNanoseconds - started.uptimeNanoseconds) / 1_000_000)
        let wordCount = text.split(whereSeparator: \.isWhitespace).count

        return GrammarAnalysisResult(errors: errors, wordCount: wordCount, analysisTimeMs: elapsedMs)
    }

    // MARK: - Match conversion

    /// Convert one LanguageTool match into a GrammarErrorModel.
    /// LT offsets are UTF-16 code units; GrammarErrorModel.start/end are Unicode-scalar indices.
    static func errorModel(from match: LTMatch, in text: String) -> GrammarErrorModel? {
        guard match.length > 0,
              let startIndex = TextIndexConverter.stringIndex(forUTF16Offset: match.offset, in: text),
              let endIndex = TextIndexConverter.stringIndex(forUTF16Offset: match.offset + match.length, in: text),
              startIndex < endIndex
        else { return nil }

        let start = TextIndexConverter.stringIndexToScalarIndex(startIndex, in: text)
        let end = TextIndexConverter.stringIndexToScalarIndex(endIndex, in: text)
        let (category, severity) = categorize(match)

        return GrammarErrorModel(
            start: start,
            end: end,
            message: match.shortMessage?.isEmpty == false ? match.shortMessage! : match.message,
            severity: severity,
            category: category,
            lintId: match.rule.id,
            suggestions: match.replacements.prefix(5).map(\.value)
        )
    }

    /// Map LT rule metadata onto TextWarden's Harper-derived category names
    /// (UserPreferences.allCategories) so category filtering keeps working.
    static func categorize(_ match: LTMatch) -> (category: String, severity: GrammarErrorSeverity) {
        let issueType = match.rule.issueType ?? ""
        let categoryID = match.rule.category?.id ?? ""

        if issueType == "misspelling" || categoryID == "TYPOS" {
            return ("Spelling", .error)
        }
        switch categoryID {
        case "CASING": return ("Capitalization", .warning)
        case "PUNCTUATION", "TYPOGRAPHY": return ("Punctuation", .warning)
        case "REDUNDANCY": return ("Redundancy", .info)
        case "STYLE", "PLAIN_ENGLISH", "WIKIPEDIA": return ("Style", .info)
        case "CONFUSED_WORDS": return ("WordChoice", .error)
        case "COLLOCATIONS": return ("Usage", .warning)
        default: break
        }
        switch issueType {
        case "typographical", "whitespace": return ("Punctuation", .warning)
        case "style", "register", "non-conformance": return ("Style", .info)
        case "duplication": return ("Repetition", .warning)
        default: return ("Grammar", .error)
        }
    }

    // MARK: - Encoding

    private static func formEncode(_ fields: [(String, String)]) -> Data {
        var allowed = CharacterSet.alphanumerics
        allowed.insert(charactersIn: "-._*")
        let encoded = fields.map { key, value in
            let escaped = value.addingPercentEncoding(withAllowedCharacters: allowed) ?? value
            return "\(key)=\(escaped)"
        }
        return encoded.joined(separator: "&").data(using: .utf8) ?? Data()
    }
}

// MARK: - LanguageTool /v2/check response models

struct LTResponse: Decodable {
    let matches: [LTMatch]
}

struct LTMatch: Decodable {
    let message: String
    let shortMessage: String?
    let offset: Int
    let length: Int
    let replacements: [LTReplacement]
    let rule: LTRule
}

struct LTReplacement: Decodable {
    let value: String
}

struct LTRule: Decodable {
    let id: String
    let issueType: String?
    let category: LTRuleCategory?
}

struct LTRuleCategory: Decodable {
    let id: String?
}
