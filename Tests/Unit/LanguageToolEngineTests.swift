//
//  LanguageToolEngineTests.swift
//  TextWarden
//
//  Tests for LanguageTool match → GrammarErrorModel conversion:
//  UTF-16 (LT/Java) → Unicode-scalar (TextWarden) offset conversion and category mapping.
//

@testable import TextWarden
import XCTest

final class LanguageToolEngineTests: XCTestCase {
    private func makeMatch(
        offset: Int,
        length: Int,
        message: String = "Test error",
        replacements: [String] = ["fix"],
        ruleId: String = "TEST_RULE",
        issueType: String? = "misspelling",
        categoryId: String? = "TYPOS"
    ) -> LTMatch {
        LTMatch(
            message: message,
            shortMessage: nil,
            offset: offset,
            length: length,
            replacements: replacements.map { LTReplacement(value: $0) },
            rule: LTRule(id: ruleId, issueType: issueType, category: LTRuleCategory(id: categoryId))
        )
    }

    // MARK: - Offset conversion

    func testAsciiOffsetsPassThrough() {
        let text = "Ths is a test"
        let match = makeMatch(offset: 0, length: 3)
        let error = LanguageToolEngine.errorModel(from: match, in: text)

        XCTAssertNotNil(error)
        XCTAssertEqual(error?.start, 0)
        XCTAssertEqual(error?.end, 3)
        XCTAssertEqual(TextIndexConverter.extractErrorText(start: error!.start, end: error!.end, from: text), "Ths")
    }

    func testEmojiBeforeErrorShiftsScalarOffset() {
        // 😀 = 2 UTF-16 code units but 1 Unicode scalar.
        // "😀 Ths is wrong" — LT reports "Ths" at UTF-16 offset 3 (2 for emoji + 1 space).
        let text = "😀 Ths is wrong"
        let match = makeMatch(offset: 3, length: 3)
        let error = LanguageToolEngine.errorModel(from: match, in: text)

        XCTAssertNotNil(error)
        // Scalar offset: 1 (emoji) + 1 (space) = 2
        XCTAssertEqual(error?.start, 2)
        XCTAssertEqual(error?.end, 5)
        XCTAssertEqual(TextIndexConverter.extractErrorText(start: error!.start, end: error!.end, from: text), "Ths")
    }

    func testMultiScalarEmojiFamily() {
        // Family emoji: one grapheme cluster, multiple scalars, more UTF-16 units than scalars.
        // Both prefix lengths are computed from the string so the test stays self-consistent.
        let family = "👨‍👩‍👧"
        let text = family + " Ths"
        let utf16PrefixLength = (family as NSString).length + 1
        let scalarPrefixLength = family.unicodeScalars.count + 1

        let match = makeMatch(offset: utf16PrefixLength, length: 3)
        let error = LanguageToolEngine.errorModel(from: match, in: text)

        XCTAssertNotNil(error)
        XCTAssertEqual(error?.start, scalarPrefixLength)
        XCTAssertEqual(error?.end, scalarPrefixLength + 3)
        XCTAssertEqual(TextIndexConverter.extractErrorText(start: error!.start, end: error!.end, from: text), "Ths")
    }

    func testOutOfBoundsOffsetReturnsNil() {
        let match = makeMatch(offset: 100, length: 3)
        XCTAssertNil(LanguageToolEngine.errorModel(from: match, in: "short"))
    }

    func testZeroLengthMatchReturnsNil() {
        let match = makeMatch(offset: 0, length: 0)
        XCTAssertNil(LanguageToolEngine.errorModel(from: match, in: "some text"))
    }

    func testReplacementsCappedAtFive() {
        let match = makeMatch(offset: 0, length: 4, replacements: ["a", "b", "c", "d", "e", "f", "g"])
        let error = LanguageToolEngine.errorModel(from: match, in: "word here")
        XCTAssertEqual(error?.suggestions.count, 5)
    }

    // MARK: - Category mapping

    func testCategoryMapping() {
        let cases: [(issueType: String?, categoryId: String?, expected: String)] = [
            ("misspelling", nil, "Spelling"),
            (nil, "TYPOS", "Spelling"),
            (nil, "CASING", "Capitalization"),
            (nil, "PUNCTUATION", "Punctuation"),
            (nil, "TYPOGRAPHY", "Punctuation"),
            (nil, "REDUNDANCY", "Redundancy"),
            (nil, "STYLE", "Style"),
            (nil, "CONFUSED_WORDS", "WordChoice"),
            (nil, "COLLOCATIONS", "Usage"),
            ("style", nil, "Style"),
            ("duplication", nil, "Repetition"),
            ("grammar", "GRAMMAR", "Grammar"),
            (nil, nil, "Grammar"),
        ]
        for testCase in cases {
            let match = makeMatch(offset: 0, length: 1, issueType: testCase.issueType, categoryId: testCase.categoryId)
            let (category, _) = LanguageToolEngine.categorize(match)
            XCTAssertEqual(category, testCase.expected, "issueType=\(testCase.issueType ?? "nil") categoryId=\(testCase.categoryId ?? "nil")")
        }
        // All mapped categories must exist in the app's filter list
        for testCase in cases {
            XCTAssertTrue(UserPreferences.allCategories.contains(testCase.expected))
        }
    }

    // MARK: - Payload cap

    func testOversizedTextReturnsNilWithoutNetworkCall() {
        // The size guard fires before any HTTP request, so this must return nil
        // immediately even with no server configured.
        let engine = LanguageToolEngine()
        let hugeText = String(repeating: "a", count: LanguageToolEngine.maxPayloadCharacters + 1)
        XCTAssertNil(engine.check(hugeText, dialect: "American"))
    }

    func testEmptyTextReturnsEmptyResultNotNil() {
        let engine = LanguageToolEngine()
        let result = engine.check("", dialect: "American")
        XCTAssertNotNil(result)
        XCTAssertEqual(result?.errors.count, 0)
    }

    // MARK: - Dialect mapping

    func testDialectMapping() {
        XCTAssertEqual(LanguageToolEngine.languageCode(forDialect: "American"), "en-US")
        XCTAssertEqual(LanguageToolEngine.languageCode(forDialect: "British"), "en-GB")
        XCTAssertEqual(LanguageToolEngine.languageCode(forDialect: "Canadian"), "en-CA")
        XCTAssertEqual(LanguageToolEngine.languageCode(forDialect: "Australian"), "en-AU")
        XCTAssertEqual(LanguageToolEngine.languageCode(forDialect: "Unknown"), "en-US")
    }
}
