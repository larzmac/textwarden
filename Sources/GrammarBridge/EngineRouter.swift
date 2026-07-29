//
//  EngineRouter.swift
//  TextWarden
//
//  Routes grammar analysis to the selected engine:
//  - .languageTool: local LanguageTool server, Harper fallback when unreachable/oversized
//  - .harper: built-in Harper engine only
//  - .auto (default): LanguageTool when reachable, Harper otherwise (with failure cooldown)
//
//  Conforms to GrammarAnalyzing so it drops into DependencyContainer unchanged.
//

import Foundation

final class EngineRouter: GrammarAnalyzing, @unchecked Sendable {
    private let languageTool: LanguageToolEngine
    private let harper: GrammarAnalyzing

    init(languageTool: LanguageToolEngine = LanguageToolEngine(), harper: GrammarAnalyzing = GrammarEngine.shared) {
        self.languageTool = languageTool
        self.harper = harper
    }

    /// LanguageTool check wrapped in performance signposts so LT latency shows up in
    /// PerformanceProfiler percentiles (Harper profiles itself inside GrammarEngine).
    private func checkWithProfiling(_ text: String, dialect: String) -> GrammarAnalysisResult? {
        PerformanceProfiler.shared.measure(.textAnalysis, context: "languagetool") {
            languageTool.check(text, dialect: dialect)
        }
    }

    func analyzeText(
        _ text: String,
        dialect: String,
        enableInternetAbbrev: Bool,
        enableGenZSlang: Bool,
        enableITTerminology: Bool,
        enableBrandNames: Bool,
        enablePersonNames: Bool,
        enableLastNames: Bool,
        enableLanguageDetection: Bool,
        excludedLanguages: [String],
        enforceOxfordComma: Bool,
        checkEllipsis: Bool,
        checkUnclosedQuotes: Bool,
        checkDashes: Bool
    ) -> GrammarAnalysisResult {
        let runHarper = {
            self.harper.analyzeText(
                text,
                dialect: dialect,
                enableInternetAbbrev: enableInternetAbbrev,
                enableGenZSlang: enableGenZSlang,
                enableITTerminology: enableITTerminology,
                enableBrandNames: enableBrandNames,
                enablePersonNames: enablePersonNames,
                enableLastNames: enableLastNames,
                enableLanguageDetection: enableLanguageDetection,
                excludedLanguages: excludedLanguages,
                enforceOxfordComma: enforceOxfordComma,
                checkEllipsis: checkEllipsis,
                checkUnclosedQuotes: checkUnclosedQuotes,
                checkDashes: checkDashes
            )
        }

        switch GrammarEngineMode.current {
        case .harper:
            return runHarper()

        case .languageTool:
            return checkWithProfiling(text, dialect: dialect)
                ?? runHarper()

        case .auto:
            if languageTool.isInFailureCooldown {
                return runHarper()
            }
            if let result = checkWithProfiling(text, dialect: dialect) {
                return result
            }
            Logger.info("EngineRouter: LanguageTool unavailable, falling back to Harper", category: Logger.analysis)
            return runHarper()
        }
    }
}
