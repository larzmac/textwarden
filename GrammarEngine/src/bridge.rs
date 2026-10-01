// FFI Bridge - Swift-Rust interop using swift-bridge
//
// Defines FFI-safe structs and functions for Swift integration.

// Allow non-camel-case types for swift-bridge generated code
#![allow(non_camel_case_types)]

use crate::analyzer;
use crate::swift_logger::{register_swift_callback, SwiftLogCallback, SwiftLoggerLayer};
use std::sync::Once;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static INIT_LOGGING: Once = Once::new();

#[swift_bridge::bridge]
mod ffi {
    pub enum ErrorSeverity {
        Error,
        Warning,
        Info,
    }

    // Opaque type for GrammarError to support Vec<GrammarError>
    extern "Rust" {
        type GrammarError;

        fn start(&self) -> usize;
        fn end(&self) -> usize;
        fn message(&self) -> String;
        fn severity(&self) -> ErrorSeverity;
        fn category(&self) -> String;
        fn lint_id(&self) -> String;
        fn suggestions(&self) -> Vec<String>;
    }

    // Opaque type for AnalysisResult
    extern "Rust" {
        type AnalysisResult;

        fn errors(&self) -> Vec<GrammarError>;
        fn word_count(&self) -> usize;
        fn analysis_time_ms(&self) -> u64;
        fn memory_before_bytes(&self) -> u64;
        fn memory_after_bytes(&self) -> u64;
        fn memory_delta_bytes(&self) -> i64;
        fn is_non_english_document(&self) -> bool;

        // Timing breakdown for performance profiling
        fn early_language_check_ms(&self) -> u64;
        fn dictionary_build_ms(&self) -> u64;
        fn linter_setup_ms(&self) -> u64;
        fn document_parse_ms(&self) -> u64;
        fn harper_lint_ms(&self) -> u64;
        fn post_process_ms(&self) -> u64;
    }

    extern "Rust" {
        fn initialize_logging(log_level: String);

        fn analyze_text(
            text: String,
            dialect: String,
            enable_internet_abbrev: bool,
            enable_genz_slang: bool,
            enable_it_terminology: bool,
            enable_brand_names: bool,
            enable_person_names: bool,
            enable_last_names: bool,
            enable_language_detection: bool,
            excluded_languages: Vec<String>,
            enable_sentence_start_capitalization: bool,
            // Individual rule toggles
            enforce_oxford_comma: bool,
            check_ellipsis: bool,
            check_unclosed_quotes: bool,
            check_dashes: bool,
        ) -> AnalysisResult;
    }
}

/// Lightweight helper to read current process memory in bytes
fn read_process_memory(pid: Option<Pid>) -> u64 {
    if let Some(pid) = pid {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, false);
        return sys.process(pid).map(|p| p.memory()).unwrap_or(0);
    }
    0
}

// FFI type implementations
#[derive(Clone)]
pub struct GrammarError {
    start: usize,
    end: usize,
    message: String,
    severity: analyzer::ErrorSeverity,
    category: String,
    lint_id: String,
    suggestions: Vec<String>,
}

impl GrammarError {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn message(&self) -> String {
        self.message.clone()
    }

    fn severity(&self) -> ffi::ErrorSeverity {
        match self.severity {
            analyzer::ErrorSeverity::Error => ffi::ErrorSeverity::Error,
            analyzer::ErrorSeverity::Warning => ffi::ErrorSeverity::Warning,
            analyzer::ErrorSeverity::Info => ffi::ErrorSeverity::Info,
        }
    }

    fn category(&self) -> String {
        self.category.clone()
    }

    fn lint_id(&self) -> String {
        self.lint_id.clone()
    }

    fn suggestions(&self) -> Vec<String> {
        self.suggestions.clone()
    }
}

pub struct AnalysisResult {
    errors: Vec<GrammarError>,
    word_count: usize,
    analysis_time_ms: u64,
    memory_before_bytes: u64,
    memory_after_bytes: u64,
    memory_delta_bytes: i64,
    /// True if document is primarily non-English (>60% excluded language)
    is_non_english_document: bool,

    // Timing breakdown for performance profiling
    early_language_check_ms: u64,
    dictionary_build_ms: u64,
    linter_setup_ms: u64,
    document_parse_ms: u64,
    harper_lint_ms: u64,
    post_process_ms: u64,
}

impl AnalysisResult {
    fn errors(&self) -> Vec<GrammarError> {
        self.errors.clone()
    }

    fn word_count(&self) -> usize {
        self.word_count
    }

    fn analysis_time_ms(&self) -> u64 {
        self.analysis_time_ms
    }

    fn memory_before_bytes(&self) -> u64 {
        self.memory_before_bytes
    }

    fn memory_after_bytes(&self) -> u64 {
        self.memory_after_bytes
    }

    fn memory_delta_bytes(&self) -> i64 {
        self.memory_delta_bytes
    }

    fn is_non_english_document(&self) -> bool {
        self.is_non_english_document
    }

    fn early_language_check_ms(&self) -> u64 {
        self.early_language_check_ms
    }

    fn dictionary_build_ms(&self) -> u64 {
        self.dictionary_build_ms
    }

    fn linter_setup_ms(&self) -> u64 {
        self.linter_setup_ms
    }

    fn document_parse_ms(&self) -> u64 {
        self.document_parse_ms
    }

    fn harper_lint_ms(&self) -> u64 {
        self.harper_lint_ms
    }

    fn post_process_ms(&self) -> u64 {
        self.post_process_ms
    }
}

// Initialize the logging system with unified Swift logging support
// This should be called once from Swift at application startup, AFTER register_log_callback
fn initialize_logging(log_level: String) {
    INIT_LOGGING.call_once(|| {
        let level = match log_level.to_lowercase().as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO, // default to info
        };

        let filter = EnvFilter::from_default_env().add_directive(level.into());

        // Create the Swift logger layer for unified logging
        let swift_layer = SwiftLoggerLayer::new(level);

        // Also keep console logging for debugging during development
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_line_number(true);

        // Initialize with both layers
        tracing_subscriber::registry()
            .with(filter)
            .with(swift_layer)
            .with(fmt_layer)
            .init();

        tracing::info!(
            "Grammar Engine logging initialized at level: {} (unified with Swift)",
            log_level
        );
    });
}

/// Register the Swift log callback for unified logging
///
/// # Safety
/// The callback pointer must be valid and point to a function with signature:
/// `extern "C" fn(level: i32, message: *const c_char)`
///
/// This must be called BEFORE initialize_logging for the callback to be active
/// during initialization.
#[no_mangle]
pub extern "C" fn register_rust_log_callback(callback: SwiftLogCallback) {
    register_swift_callback(callback);
}

// FFI wrapper that calls the analyzer and converts types
#[tracing::instrument(skip(text), fields(text_len = text.len()))]
fn analyze_text(
    text: String,
    dialect: String,
    enable_internet_abbrev: bool,
    enable_genz_slang: bool,
    enable_it_terminology: bool,
    enable_brand_names: bool,
    enable_person_names: bool,
    enable_last_names: bool,
    enable_language_detection: bool,
    excluded_languages: Vec<String>,
    enable_sentence_start_capitalization: bool,
    // Individual rule toggles
    enforce_oxford_comma: bool,
    check_ellipsis: bool,
    check_unclosed_quotes: bool,
    check_dashes: bool,
) -> AnalysisResult {
    // SECURITY: Never log the actual text content - only metadata
    tracing::debug!(
        "FFI analyze_text called: dialect={}, text_len={}, lang_detection={}",
        dialect,
        text.len(),
        enable_language_detection
    );

    let pid = sysinfo::get_current_pid().ok();
    let memory_before = read_process_memory(pid);

    let result = analyzer::analyze_text(
        &text,
        &dialect,
        enable_internet_abbrev,
        enable_genz_slang,
        enable_it_terminology,
        enable_brand_names,
        enable_person_names,
        enable_last_names,
        enable_language_detection,
        excluded_languages,
        enable_sentence_start_capitalization,
        enforce_oxford_comma,
        check_ellipsis,
        check_unclosed_quotes,
        check_dashes,
    );

    // Capture memory after analysis
    let memory_after = read_process_memory(pid);

    let memory_delta = (memory_after as i64) - (memory_before as i64);

    tracing::info!(
        "Analysis completed: {} errors found, {} words, {}ms, mem_delta={}KB",
        result.errors.len(),
        result.word_count,
        result.analysis_time_ms,
        memory_delta / 1024
    );

    // Convert analyzer types to FFI types
    let errors = result
        .errors
        .into_iter()
        .map(|err| GrammarError {
            start: err.start,
            end: err.end,
            message: err.message,
            severity: err.severity,
            category: err.category,
            lint_id: err.lint_id,
            suggestions: err.suggestions,
        })
        .collect();

    AnalysisResult {
        errors,
        word_count: result.word_count,
        analysis_time_ms: result.analysis_time_ms,
        memory_before_bytes: memory_before,
        memory_after_bytes: memory_after,
        memory_delta_bytes: memory_delta,
        is_non_english_document: result.is_non_english_document,
        early_language_check_ms: result.early_language_check_ms,
        dictionary_build_ms: result.dictionary_build_ms,
        linter_setup_ms: result.linter_setup_ms,
        document_parse_ms: result.document_parse_ms,
        harper_lint_ms: result.harper_lint_ms,
        post_process_ms: result.post_process_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_error_construction() {
        let err = GrammarError {
            start: 0,
            end: 5,
            message: "Test error".to_string(),
            severity: analyzer::ErrorSeverity::Warning,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec!["test".to_string(), "tes".to_string()],
        };

        assert_eq!(err.start(), 0);
        assert_eq!(err.end(), 5);
        assert_eq!(err.message(), "Test error");
        assert_eq!(err.category(), "spelling");
        assert_eq!(err.lint_id(), "spell");
    }

    #[test]
    fn test_grammar_error_severity_mapping() {
        let err_error = GrammarError {
            start: 0,
            end: 3,
            message: "error".to_string(),
            severity: analyzer::ErrorSeverity::Error,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec![],
        };

        let err_warning = GrammarError {
            start: 0,
            end: 3,
            message: "warning".to_string(),
            severity: analyzer::ErrorSeverity::Warning,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec![],
        };

        let err_info = GrammarError {
            start: 0,
            end: 3,
            message: "info".to_string(),
            severity: analyzer::ErrorSeverity::Info,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec![],
        };

        match err_error.severity() {
            ffi::ErrorSeverity::Error => {}
            _ => panic!("expected Error severity"),
        }

        match err_warning.severity() {
            ffi::ErrorSeverity::Warning => {}
            _ => panic!("expected Warning severity"),
        }

        match err_info.severity() {
            ffi::ErrorSeverity::Info => {}
            _ => panic!("expected Info severity"),
        }
    }

    #[test]
    fn test_grammar_error_suggestions() {
        let err = GrammarError {
            start: 0,
            end: 4,
            message: "typo".to_string(),
            severity: analyzer::ErrorSeverity::Error,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec!["they're".to_string(), "their".to_string()],
        };

        let suggestions = err.suggestions();
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "they're");
        assert_eq!(suggestions[1], "their");
    }

    #[test]
    fn test_grammar_error_empty_suggestions() {
        let err = GrammarError {
            start: 0,
            end: 3,
            message: "empty".to_string(),
            severity: analyzer::ErrorSeverity::Error,
            category: "grammar".to_string(),
            lint_id: "comma".to_string(),
            suggestions: vec![],
        };

        let suggestions = err.suggestions();
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_grammar_error_unicode_message() {
        let err = GrammarError {
            start: 0,
            end: 3,
            message: "错误消息".to_string(),
            severity: analyzer::ErrorSeverity::Warning,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec!["correct".to_string()],
        };

        assert_eq!(err.message(), "错误消息");
    }

    #[test]
    fn test_analysis_result_construction() {
        let result = AnalysisResult {
            errors: vec![],
            word_count: 10,
            analysis_time_ms: 42,
            memory_before_bytes: 1_000_000,
            memory_after_bytes: 1_005_000,
            memory_delta_bytes: 5_000,
            is_non_english_document: false,
            early_language_check_ms: 2,
            dictionary_build_ms: 5,
            linter_setup_ms: 3,
            document_parse_ms: 8,
            harper_lint_ms: 20,
            post_process_ms: 4,
        };

        assert_eq!(result.word_count(), 10);
        assert_eq!(result.analysis_time_ms(), 42);
        assert_eq!(result.memory_before_bytes(), 1_000_000);
        assert_eq!(result.memory_after_bytes(), 1_005_000);
        assert_eq!(result.memory_delta_bytes(), 5_000);
        assert_eq!(result.is_non_english_document(), false);
        assert_eq!(result.early_language_check_ms(), 2);
        assert_eq!(result.dictionary_build_ms(), 5);
        assert_eq!(result.linter_setup_ms(), 3);
        assert_eq!(result.document_parse_ms(), 8);
        assert_eq!(result.harper_lint_ms(), 20);
        assert_eq!(result.post_process_ms(), 4);
    }

    #[test]
    fn test_analysis_result_with_errors() {
        let errors = vec![
            GrammarError {
                start: 0,
                end: 3,
                message: "typo1".to_string(),
                severity: analyzer::ErrorSeverity::Error,
                category: "spelling".to_string(),
                lint_id: "spell".to_string(),
                suggestions: vec!["the".to_string()],
            },
            GrammarError {
                start: 20,
                end: 25,
                message: "typo2".to_string(),
                severity: analyzer::ErrorSeverity::Warning,
                category: "grammar".to_string(),
                lint_id: "comma".to_string(),
                suggestions: vec![],
            },
        ];

        let result = AnalysisResult {
            errors,
            word_count: 100,
            analysis_time_ms: 50,
            memory_before_bytes: 2_000_000,
            memory_after_bytes: 2_002_500,
            memory_delta_bytes: 2_500,
            is_non_english_document: true,
            early_language_check_ms: 1,
            dictionary_build_ms: 10,
            linter_setup_ms: 5,
            document_parse_ms: 15,
            harper_lint_ms: 25,
            post_process_ms: 4,
        };

        let errs = result.errors();
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].start(), 0);
        assert_eq!(errs[1].end(), 25);
        assert_eq!(result.is_non_english_document(), true);
    }

    #[test]
    fn test_analysis_result_zero_timing() {
        let result = AnalysisResult {
            errors: vec![],
            word_count: 0,
            analysis_time_ms: 0,
            memory_before_bytes: 0,
            memory_after_bytes: 0,
            memory_delta_bytes: 0,
            is_non_english_document: false,
            early_language_check_ms: 0,
            dictionary_build_ms: 0,
            linter_setup_ms: 0,
            document_parse_ms: 0,
            harper_lint_ms: 0,
            post_process_ms: 0,
        };

        assert_eq!(result.word_count(), 0);
        assert_eq!(result.analysis_time_ms(), 0);
    }

    #[test]
    fn test_analysis_result_negative_delta() {
        let result = AnalysisResult {
            errors: vec![],
            word_count: 5,
            analysis_time_ms: 10,
            memory_before_bytes: 5_000_000,
            memory_after_bytes: 4_900_000,
            memory_delta_bytes: -100_000,
            is_non_english_document: false,
            early_language_check_ms: 0,
            dictionary_build_ms: 0,
            linter_setup_ms: 0,
            document_parse_ms: 0,
            harper_lint_ms: 8,
            post_process_ms: 2,
        };

        assert_eq!(result.memory_delta_bytes(), -100_000);
    }

    #[test]
    fn test_grammar_error_clone() {
        let err = GrammarError {
            start: 5,
            end: 10,
            message: "clone me".to_string(),
            severity: analyzer::ErrorSeverity::Info,
            category: "style".to_string(),
            lint_id: "style".to_string(),
            suggestions: vec!["suggested".to_string()],
        };

        let cloned = err.clone();
        assert_eq!(cloned.start(), 5);
        assert_eq!(cloned.end(), 10);
        assert_eq!(cloned.message(), "clone me");
    }

    #[test]
    fn test_analysis_result_clone() {
        let errors = vec![GrammarError {
            start: 0,
            end: 4,
            message: "orig".to_string(),
            severity: analyzer::ErrorSeverity::Error,
            category: "spelling".to_string(),
            lint_id: "spell".to_string(),
            suggestions: vec!["or".to_string()],
        }];

        let result = AnalysisResult {
            errors,
            word_count: 1,
            analysis_time_ms: 5,
            memory_before_bytes: 100,
            memory_after_bytes: 200,
            memory_delta_bytes: 100,
            is_non_english_document: false,
            early_language_check_ms: 0,
            dictionary_build_ms: 0,
            linter_setup_ms: 0,
            document_parse_ms: 0,
            harper_lint_ms: 3,
            post_process_ms: 2,
        };

        // Verify AnalysisResult accessor methods work by constructing a fresh copy
        let errors_copy = result.errors();
        assert_eq!(result.word_count(), 1);
        assert_eq!(errors_copy.len(), 1);
        assert_eq!(result.harper_lint_ms(), 3);
    }
}
