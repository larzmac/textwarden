//
//  OllamaStyleEngineTests.swift
//  TextWarden
//
//  Tests for the Ollama style engine. The live tests talk to a local Ollama
//  server and skip (not fail) when it isn't running.
//

@testable import TextWarden
import XCTest

final class OllamaStyleEngineTests: XCTestCase {
    // MARK: - Response parsing (no server needed)

    func testChatResponseParsing() throws {
        let json = #"{"message": {"role": "assistant", "content": "{\"suggestions\": []}"}, "done": true}"#
        let decoded = try JSONDecoder().decode(OllamaChatResponse.self, from: Data(json.utf8))
        XCTAssertEqual(decoded.message.content, #"{"suggestions": []}"#)
    }

    func testStyleSuggestionListParsing() throws {
        let json = #"{"suggestions": [{"original": "in order to", "suggested": "to", "explanation": "Wordy"}]}"#
        let decoded = try JSONDecoder().decode(OllamaStyleSuggestionList.self, from: Data(json.utf8))
        XCTAssertEqual(decoded.suggestions.count, 1)
        XCTAssertEqual(decoded.suggestions[0].suggested, "to")
    }

    func testErrorResponseParsing() throws {
        let json = #"{"error": "model not found"}"#
        let decoded = try JSONDecoder().decode(OllamaErrorResponse.self, from: Data(json.utf8))
        XCTAssertEqual(decoded.error, "model not found")
    }

    // MARK: - Live integration (skips when no local server)

    @MainActor
    func testLiveStyleAnalysis() async throws {
        guard #available(macOS 26.0, *) else { throw XCTSkip("Requires macOS 26") }

        let engine = OllamaStyleEngine()
        guard engine.status.isAvailable else {
            throw XCTSkip("Ollama server not running - skipping live test")
        }

        let text = "In order to facilitate the achievement of our objectives, it is imperative that we leverage our core competencies at this point in time."
        let suggestions = try await engine.analyzeStyle(
            text,
            style: .default,
            temperaturePreset: .consistent,
            customVocabulary: []
        )

        XCTAssertFalse(suggestions.isEmpty, "Expected at least one style suggestion for wordy corporate text")
        for suggestion in suggestions {
            // The conversion pipeline guarantees these invariants
            XCTAssertTrue(text.contains(suggestion.originalText), "original must be a verbatim substring")
            XCTAssertNotEqual(suggestion.originalText, suggestion.suggestedText)
            XCTAssertGreaterThanOrEqual(suggestion.originalStart, 0)
            XCTAssertGreaterThan(suggestion.originalEnd, suggestion.originalStart)
        }
    }
}
