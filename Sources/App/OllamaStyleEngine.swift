//
//  OllamaStyleEngine.swift
//  TextWarden
//
//  Style/rewrite engine backed by a local Ollama server (default localhost:11434).
//  Fully local: no cloud, no accounts, no per-use cost — an alternative to
//  Apple Intelligence selectable in Settings → Style.
//
//  Uses Ollama's structured outputs (`format` = JSON schema) so responses decode
//  into the same shapes as Foundation Models results, then reuses the existing
//  FMStyleSuggestion → StyleSuggestionModel conversion (validation + diffing).
//

import Foundation

/// Ollama configuration shared with the settings UI (not availability-gated)
enum OllamaConfig {
    static let serverURLDefaultsKey = "ollamaServerURL"
    static let modelDefaultsKey = "ollamaModel"
    static let defaultServerURL = "http://localhost:11434"
    static let defaultModel = "qwen3:30b-a3b-instruct-2507-q4_K_M"
}

@available(macOS 26.0, *)
@MainActor
final class OllamaStyleEngine: StyleEngine {
    /// How long Ollama keeps the model in memory after a request
    private static let keepAlive = "30m"

    private(set) var status: StyleEngineStatus = .unknown("Not checked yet")

    private let session: URLSession

    init() {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = 180 // cold model load can take a minute
        config.timeoutIntervalForResource = 300
        session = URLSession(configuration: config)
        checkAvailability()
    }

    private static var serverURL: String {
        let stored = UserDefaults.standard.string(forKey: OllamaConfig.serverURLDefaultsKey)?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return stored.isEmpty ? OllamaConfig.defaultServerURL : stored
    }

    private static var model: String {
        let stored = UserDefaults.standard.string(forKey: OllamaConfig.modelDefaultsKey)?
            .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return stored.isEmpty ? OllamaConfig.defaultModel : stored
    }

    // MARK: - Availability

    /// Quick synchronous probe of the local server. Localhost connection refusal is
    /// instant when the server is down; a running server answers in milliseconds.
    func checkAvailability() {
        guard let url = URL(string: Self.serverURL + "/api/version") else {
            status = .unknown("Invalid Ollama server URL")
            return
        }
        var request = URLRequest(url: url)
        request.timeoutInterval = 1.5

        var reachable = false
        let semaphore = DispatchSemaphore(value: 0)
        session.dataTask(with: request) { _, response, _ in
            reachable = (response as? HTTPURLResponse)?.statusCode == 200
            semaphore.signal()
        }.resume()
        semaphore.wait()

        if reachable {
            status = .available
            Logger.debug("Ollama: Server reachable at \(Self.serverURL)", category: Logger.llm)
        } else {
            status = .unknown("Ollama server not running at \(Self.serverURL)")
            Logger.info("Ollama: Server not reachable at \(Self.serverURL)", category: Logger.llm)
        }
    }

    /// Ask Ollama to load the model into memory so the first real request is fast
    func prewarm() async {
        guard status.isAvailable else { return }
        guard let url = URL(string: Self.serverURL + "/api/generate") else { return }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try? JSONSerialization.data(withJSONObject: [
            "model": Self.model, "keep_alive": Self.keepAlive,
        ])
        _ = try? await session.data(for: request)
        Logger.debug("Ollama: Prewarmed model \(Self.model)", category: Logger.llm)
    }

    // MARK: - Core chat call

    /// POST /api/chat with a JSON-schema-constrained response format.
    /// Returns the assistant message content (a JSON string matching the schema).
    private func chat(
        system: String,
        user: String,
        schema: [String: Any],
        temperature: Double
    ) async throws -> Data {
        guard let url = URL(string: Self.serverURL + "/api/chat") else {
            throw FoundationModelsError.notAvailable(status)
        }

        let body: [String: Any] = [
            "model": Self.model,
            "stream": false,
            "keep_alive": Self.keepAlive,
            "messages": [
                ["role": "system", "content": system],
                ["role": "user", "content": user],
            ],
            "format": schema,
            "options": ["temperature": temperature],
        ]

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONSerialization.data(withJSONObject: body)

        let startTime = CFAbsoluteTimeGetCurrent()
        let (data, response) = try await session.data(for: request)
        let elapsed = CFAbsoluteTimeGetCurrent() - startTime

        guard (response as? HTTPURLResponse)?.statusCode == 200 else {
            let serverMessage = (try? JSONDecoder().decode(OllamaErrorResponse.self, from: data))?.error
                ?? "HTTP \((response as? HTTPURLResponse)?.statusCode ?? 0)"
            Logger.error("Ollama: Request failed - \(serverMessage)", category: Logger.llm)
            throw FoundationModelsError.generationFailed("Ollama: \(serverMessage)")
        }

        let decoded = try JSONDecoder().decode(OllamaChatResponse.self, from: data)
        Logger.info("Ollama: Response in \(String(format: "%.1f", elapsed))s (model: \(Self.model))", category: Logger.llm)
        return Data(decoded.message.content.utf8)
    }

    // MARK: - Style Analysis

    func analyzeStyle(
        _ text: String,
        style: WritingStyle,
        temperaturePreset: StyleTemperaturePreset = .balanced,
        customVocabulary: [String] = []
    ) async throws -> [StyleSuggestionModel] {
        guard status.isAvailable else { throw FoundationModelsError.notAvailable(status) }

        // Reuse the same instructions Apple Intelligence gets, plus schema-specific rules
        let instructions = StyleInstructions.build(for: style, customVocabulary: customVocabulary) + """


        Output rules:
        - "original" must be a VERBATIM substring of the input text (character-for-character).
        - "suggested" must differ from "original" while keeping the meaning.
        - "explanation" is a brief reason, maximum 10 words, without quoting the text.
        - Return at most 5 suggestions, most important first. Empty array if the text is already well-written.
        """

        let content = try await chat(
            system: instructions,
            user: "Analyze this text for style improvements:\n\n\(text)",
            schema: OllamaSchemas.styleSuggestions,
            temperature: temperaturePreset.temperature
        )

        let parsed = try JSONDecoder().decode(OllamaStyleSuggestionList.self, from: content)
        let fmResult = FMStyleAnalysisResult(
            suggestions: parsed.suggestions.map {
                FMStyleSuggestion(original: $0.original, suggested: $0.suggested, explanation: $0.explanation)
            }
        )
        let models = fmResult.toStyleSuggestionModels(in: text, style: style)
        Logger.info("Ollama: [Style] \(models.count) valid suggestion(s) from \(parsed.suggestions.count) raw", category: Logger.llm)
        return models
    }

    func regenerateStyleSuggestion(
        originalText: String,
        previousSuggestion: StyleSuggestionModel,
        style: WritingStyle,
        customVocabulary: [String] = []
    ) async throws -> StyleSuggestionModel? {
        guard status.isAvailable else { throw FoundationModelsError.notAvailable(status) }

        let instructions = StyleInstructions.build(for: style, customVocabulary: customVocabulary) + """


        IMPORTANT: You must provide a DIFFERENT suggestion than this previous one:
        Previous suggestion: "\(previousSuggestion.suggestedText)"

        Provide an alternative way to improve the text. Be creative but accurate.
        - "original" must be a VERBATIM substring of the input text.
        """

        let content = try await chat(
            system: instructions,
            user: "Provide an alternative style improvement for this text:\n\n\(originalText)",
            schema: OllamaSchemas.styleSuggestions,
            temperature: 0.5
        )

        let parsed = try JSONDecoder().decode(OllamaStyleSuggestionList.self, from: content)
        let fmResult = FMStyleAnalysisResult(
            suggestions: parsed.suggestions.map {
                FMStyleSuggestion(original: $0.original, suggested: $0.suggested, explanation: $0.explanation)
            }
        )
        return fmResult
            .toStyleSuggestionModels(in: originalText, style: style)
            .first { $0.suggestedText != previousSuggestion.suggestedText }
    }

    // MARK: - Text Generation

    func generateText(
        instruction: String,
        context: GenerationContext,
        style: WritingStyle,
        variationSeed: UInt64? = nil
    ) async throws -> String {
        guard status.isAvailable else { throw FoundationModelsError.notAvailable(status) }

        var promptParts: [String] = []
        promptParts.append("User instruction: \(instruction)")
        promptParts.append("\nWriting style: \(style.displayName)")

        // Local models have far larger context windows than Apple's 4096 tokens,
        // but stay conservative to keep latency predictable
        let maxContextChars = 8000
        if let selected = context.selectedText, !selected.isEmpty {
            promptParts.append("\n[Optional reference - selected text in document]:\n\"\"\"\n\(selected.prefix(maxContextChars))\n\"\"\"")
        } else if let surrounding = context.surroundingText, !surrounding.isEmpty, context.source != .none {
            promptParts.append("\n[Optional reference - nearby text for context only]:\n\"\"\"\n\(surrounding.prefix(maxContextChars))\n\"\"\"")
        }

        let instructions = """
        You are a text generation assistant. Your ONLY job is to follow the user's instruction exactly.

        Critical rules:
        - The user's instruction is ABSOLUTE - follow it precisely
        - If the user asks for "unrelated" or "random" text, generate completely NEW content
        - Do NOT copy, paraphrase, or base your output on any provided context unless explicitly asked
        - Context is ONLY provided as optional reference - ignore it unless the instruction specifically refers to it
        - Output ONLY the generated text - no explanations, labels, or meta-commentary
        - Match the specified writing style
        """

        // A variation seed means the user asked to regenerate: raise temperature for variety
        let temperature = variationSeed != nil ? 0.8 : 0.3

        let content = try await chat(
            system: instructions,
            user: promptParts.joined(separator: "\n"),
            schema: OllamaSchemas.textGeneration,
            temperature: temperature
        )

        return try JSONDecoder().decode(OllamaTextGenerationResult.self, from: content).generatedText
    }

    // MARK: - Sentence Simplification

    func simplifySentence(
        _ sentence: String,
        targetAudience: TargetAudience,
        writingStyle: WritingStyle,
        previousSuggestion: String? = nil
    ) async throws -> [String] {
        guard status.isAvailable else { throw FoundationModelsError.notAvailable(status) }

        var instructions = """
        You are a readability expert. Your task is to simplify sentences for a specific target audience.

        Target audience: \(targetAudience.displayName) (\(targetAudience.audienceDescription))
        Target reading level: \(targetAudience.gradeLevel)
        Writing style: \(writingStyle.displayName)

        Simplification guidelines:
        - Break long sentences into shorter ones if needed
        - Replace complex words with simpler alternatives
        - Use active voice instead of passive voice
        - Remove unnecessary jargon and filler words
        - Preserve the core meaning exactly - all key information must be retained
        - Match the specified writing style
        - Do NOT add information that wasn't in the original

        If the sentence is already simple enough for the target audience, return an empty array.
        """

        if let previous = previousSuggestion, !previous.isEmpty {
            instructions += """


            CRITICAL: The user rejected this previous simplification, you MUST provide a completely different alternative:
            Rejected: "\(previous)"
            """
        }

        let content = try await chat(
            system: instructions,
            user: "Simplify this sentence:\n\n\"\(sentence)\"",
            schema: OllamaSchemas.simplification,
            temperature: previousSuggestion != nil ? 0.9 : 0.3
        )

        let alternatives = try JSONDecoder().decode(OllamaSimplificationResult.self, from: content).alternatives
        let originalTrimmed = sentence.trimmingCharacters(in: .whitespacesAndNewlines)
        let previousTrimmed = previousSuggestion?.trimmingCharacters(in: .whitespacesAndNewlines)
        return alternatives.filter { alt in
            let trimmed = alt.trimmingCharacters(in: .whitespacesAndNewlines)
            return !trimmed.isEmpty && trimmed != originalTrimmed && trimmed != previousTrimmed
        }
    }

    // MARK: - Readability Tips

    func generateReadabilityTips(
        for text: String,
        score: Int,
        targetAudience: TargetAudience
    ) async throws -> [String] {
        guard status.isAvailable else { throw FoundationModelsError.notAvailable(status) }

        let wordCount = text.split { $0.isWhitespace || $0.isNewline }.count
        guard wordCount >= 5 else { return [] }

        let analysisText = text.count > 1000 ? String(text.prefix(1000)) + "..." : text
        let needsTips = score < 60

        let instructions = """
        You are a readability analyst. Analyze the text and provide helpful, actionable tips.

        Text statistics: \(wordCount) words, readability score \(score)/100 (Flesch Reading Ease)
        Target audience: \(targetAudience.displayName)

        Current score (\(score)) indicates: \(needsTips ? "text needs improvement" : "text is readable")

        RULES:
        - Provide general, actionable tips based on patterns you observe
        - Do NOT quote specific text or mention exact word/sentence counts
        - Keep tips concise (under 15 words each)
        - Focus on: sentence length, word complexity, passive voice, clarity
        - For scores below 60, ALWAYS provide at least 1-2 helpful tips
        - For scores 70+, return empty array (text is already good)
        """

        let content = try await chat(
            system: instructions,
            user: "Analyze this text for readability and provide helpful tips:\n\n\"\(analysisText)\"",
            schema: OllamaSchemas.readabilityTips,
            temperature: 0.3
        )

        return try JSONDecoder().decode(OllamaReadabilityTipsResult.self, from: content)
            .tips.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }
}

// MARK: - Response schemas (Ollama structured outputs)

enum OllamaSchemas {
    static let styleSuggestions: [String: Any] = [
        "type": "object",
        "properties": [
            "suggestions": [
                "type": "array",
                "items": [
                    "type": "object",
                    "properties": [
                        "original": ["type": "string"],
                        "suggested": ["type": "string"],
                        "explanation": ["type": "string"],
                    ],
                    "required": ["original", "suggested", "explanation"],
                ],
            ],
        ],
        "required": ["suggestions"],
    ]

    static let textGeneration: [String: Any] = [
        "type": "object",
        "properties": ["generatedText": ["type": "string"]],
        "required": ["generatedText"],
    ]

    static let simplification: [String: Any] = [
        "type": "object",
        "properties": ["alternatives": ["type": "array", "items": ["type": "string"]]],
        "required": ["alternatives"],
    ]

    static let readabilityTips: [String: Any] = [
        "type": "object",
        "properties": ["tips": ["type": "array", "items": ["type": "string"]]],
        "required": ["tips"],
    ]
}

// MARK: - Response models

struct OllamaChatResponse: Decodable {
    struct Message: Decodable {
        let content: String
    }

    let message: Message
}

struct OllamaErrorResponse: Decodable {
    let error: String
}

struct OllamaStyleSuggestionList: Decodable {
    struct Suggestion: Decodable {
        let original: String
        let suggested: String
        let explanation: String
    }

    let suggestions: [Suggestion]
}

struct OllamaTextGenerationResult: Decodable {
    let generatedText: String
}

struct OllamaSimplificationResult: Decodable {
    let alternatives: [String]
}

struct OllamaReadabilityTipsResult: Decodable {
    let tips: [String]
}
