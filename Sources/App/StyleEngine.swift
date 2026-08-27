//
//  StyleEngine.swift
//  TextWarden
//
//  Abstraction over the AI style/rewrite backends so the user can switch between
//  Apple Intelligence (FoundationModelsEngine) and a local Ollama model
//  (OllamaStyleEngine). Both run entirely on-device: no cloud, no per-use cost.
//

import Foundation

// MARK: - Engine selection

/// Which AI backend powers style suggestions, rewrites, and simplification.
/// Persisted in UserDefaults; read by StyleEngineFactory and the settings UI.
enum StyleEngineMode: String, CaseIterable {
    case appleIntelligence
    case ollama

    static let defaultsKey = "styleEngineMode"

    var displayName: String {
        switch self {
        case .appleIntelligence: "Apple Intelligence (built-in)"
        case .ollama: "Ollama (local model)"
        }
    }

    static var current: StyleEngineMode {
        StyleEngineMode(rawValue: UserDefaults.standard.string(forKey: defaultsKey) ?? "") ?? .appleIntelligence
    }
}

// MARK: - Protocol

/// Common interface for style/rewrite engines.
/// Mirrors FoundationModelsEngine's API so it can be adopted without changes.
@MainActor
protocol StyleEngine: AnyObject {
    var status: StyleEngineStatus { get }

    func checkAvailability()
    func prewarm() async

    func analyzeStyle(
        _ text: String,
        style: WritingStyle,
        temperaturePreset: StyleTemperaturePreset,
        customVocabulary: [String]
    ) async throws -> [StyleSuggestionModel]

    func regenerateStyleSuggestion(
        originalText: String,
        previousSuggestion: StyleSuggestionModel,
        style: WritingStyle,
        customVocabulary: [String]
    ) async throws -> StyleSuggestionModel?

    func generateText(
        instruction: String,
        context: GenerationContext,
        style: WritingStyle,
        variationSeed: UInt64?
    ) async throws -> String

    func simplifySentence(
        _ sentence: String,
        targetAudience: TargetAudience,
        writingStyle: WritingStyle,
        previousSuggestion: String?
    ) async throws -> [String]

    func generateReadabilityTips(
        for text: String,
        score: Int,
        targetAudience: TargetAudience
    ) async throws -> [String]
}

@available(macOS 26.0, *)
extension FoundationModelsEngine: StyleEngine {}

// MARK: - Factory

/// Creates the style engine matching the user's selected mode.
@available(macOS 26.0, *)
@MainActor
enum StyleEngineFactory {
    static func make() -> any StyleEngine {
        switch StyleEngineMode.current {
        case .appleIntelligence:
            FoundationModelsEngine()
        case .ollama:
            OllamaStyleEngine()
        }
    }
}
