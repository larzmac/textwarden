// Wordlist Dictionary Loader
//
// Extensible system for loading predefined wordlists for Harper grammar engine
// Supports internet abbreviations, Gen Z slang, and future wordlists (IT, medical, legal, etc.)

use harper_core::CharString;
use harper_core::DictWordMetadata;

/// Wordlist categories available in the system
/// Add new variants here when introducing new wordlists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordlistCategory {
    /// Internet abbreviations (BTW, FYI, LOL, ASAP, etc.)
    InternetAbbreviations,
    /// Gen Z slang (ghosting, sus, slay, etc.)
    GenZSlang,
    /// IT and technical terminology (API, JSON, localhost, kubernetes, etc.)
    ITTerminology,
    /// Brand and product names with correct capitalization (iPhone, GitHub, eBay, etc.)
    BrandNames,
    /// International first names from US SSA and 106 countries
    PersonNames,
    /// US Census Bureau surnames (last names)
    LastNames,
}

/// Metadata about a wordlist
pub struct WordlistInfo {
    pub category: WordlistCategory,
    pub name: &'static str,
    pub description: &'static str,
    pub word_count_estimate: usize,
}

impl WordlistCategory {
    /// Get metadata about this wordlist category
    pub fn info(&self) -> WordlistInfo {
        match self {
            WordlistCategory::InternetAbbreviations => WordlistInfo {
                category: *self,
                name: "Internet Abbreviations",
                description: "Common internet abbreviations and initialisms",
                word_count_estimate: 3200,
            },
            WordlistCategory::GenZSlang => WordlistInfo {
                category: *self,
                name: "Gen Z Slang",
                description: "Modern slang and informal language",
                word_count_estimate: 270,
            },
            WordlistCategory::ITTerminology => WordlistInfo {
                category: *self,
                name: "IT Terminology",
                description: "Technical IT terms from NIST, IANA, Linux, CNCF, and more",
                word_count_estimate: 10000,
            },
            WordlistCategory::BrandNames => WordlistInfo {
                category: *self,
                name: "Brand Names",
                description: "Fortune 500, Forbes 2000, and technology brand names",
                word_count_estimate: 2400,
            },
            WordlistCategory::PersonNames => WordlistInfo {
                category: *self,
                name: "Person Names",
                description: "International first names from US SSA and 106 countries",
                word_count_estimate: 100000,
            },
            WordlistCategory::LastNames => WordlistInfo {
                category: *self,
                name: "Last Names",
                description: "US Census Bureau surnames (151,670 names)",
                word_count_estimate: 151670,
            },
        }
    }

    /// Load words for this wordlist category
    /// Returns a Vec of (CharString, DictWordMetadata) tuples
    /// All words are stored in lowercase - Harper's spell checker will match any case automatically
    pub fn load_words(&self) -> Vec<(CharString, DictWordMetadata)> {
        match self {
            WordlistCategory::InternetAbbreviations => {
                const ABBREVIATIONS: &str = include_str!("../wordlists/internet_abbreviations.txt");
                load_words_lowercase_only(ABBREVIATIONS)
            }
            WordlistCategory::GenZSlang => {
                const GENZ_SLANG: &str = include_str!("../wordlists/genz_slang.txt");
                load_words_lowercase_only(GENZ_SLANG)
            }
            WordlistCategory::ITTerminology => {
                const IT_TERMS: &str = include_str!("../wordlists/it_terminology.txt");
                load_words_lowercase_only(IT_TERMS)
            }
            WordlistCategory::BrandNames => {
                const BRAND_NAMES: &str = include_str!("../wordlists/brand_names.txt");
                load_words_lowercase_only(BRAND_NAMES)
            }
            WordlistCategory::PersonNames => {
                const PERSON_NAMES: &str = include_str!("../wordlists/person_names.txt");
                load_words_lowercase_only(PERSON_NAMES)
            }
            WordlistCategory::LastNames => {
                const LAST_NAMES: &str = include_str!("../wordlists/last_names.txt");
                load_words_lowercase_only(LAST_NAMES)
            }
        }
    }
}

/// Parse text file and convert all words to lowercase
/// Harper's spell checker automatically accepts any case when words are stored in lowercase
/// This works because SpellCheck checks both exact match and to_lower() match
/// Returns a Vec of (CharString, DictWordMetadata) tuples
fn load_words_lowercase_only(text: &str) -> Vec<(CharString, DictWordMetadata)> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            // Convert to lowercase CharString (Vec<char>)
            let lowercase: CharString = trimmed.to_lowercase().chars().collect();

            // Create default metadata for the word
            let metadata = DictWordMetadata::default();

            Some((lowercase, metadata))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::analyze_text;

    #[test]
    fn test_load_internet_abbreviations() {
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();

        // Should have loaded many abbreviations (lowercase only now)
        assert!(abbrevs.len() > 3000, "Should have 3000+ abbreviations");

        // Check for some common ones (all lowercase)
        let words: Vec<String> = abbrevs
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        assert!(words.contains(&"btw".to_string()), "Should contain 'btw'");
        assert!(words.contains(&"fyi".to_string()), "Should contain 'fyi'");
        assert!(words.contains(&"lol".to_string()), "Should contain 'lol'");
        assert!(words.contains(&"asap".to_string()), "Should contain 'asap'");
        assert!(words.contains(&"imho".to_string()), "Should contain 'imho'");
        assert!(
            words.contains(&"afaict".to_string()),
            "Should contain 'afaict'"
        );
    }

    #[test]
    fn test_load_genz_slang() {
        let slang = WordlistCategory::GenZSlang.load_words();

        // Should have loaded slang words
        assert!(slang.len() > 100, "Should have 100+ slang terms");

        // Check for some modern slang
        let words: Vec<String> = slang
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Note: these are lowercase in the file
        assert!(
            words.iter().any(|w| w.to_lowercase() == "ghosting"),
            "Should contain 'ghosting'"
        );
        assert!(
            words.iter().any(|w| w.to_lowercase() == "sus"),
            "Should contain 'sus'"
        );
        assert!(
            words.iter().any(|w| w.to_lowercase() == "slay"),
            "Should contain 'slay'"
        );
    }

    #[test]
    fn test_comments_and_empty_lines_ignored() {
        let test_text = "# This is a comment\nBTW\n\n  \nFYI\n# Another comment\nLOL";
        let words = load_words_lowercase_only(test_text);

        assert_eq!(
            words.len(),
            3,
            "Should only load 3 words, ignoring comments and empty lines"
        );

        let word_strings: Vec<String> = words
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        assert_eq!(word_strings, vec!["btw", "fyi", "lol"]);
    }

    #[test]
    fn test_lowercase_only_generation() {
        // Test that only lowercase variants are generated
        let test_text = "BTW\nFYI\nLOL";
        let words = load_words_lowercase_only(test_text);

        // Should generate only 3 words (one per line, all lowercase)
        assert_eq!(words.len(), 3, "Should generate 3 lowercase words");

        let word_strings: Vec<String> = words
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Check that we have only lowercase variants
        assert!(
            word_strings.contains(&"btw".to_string()),
            "Should contain lowercase 'btw'"
        );
        assert!(
            word_strings.contains(&"fyi".to_string()),
            "Should contain lowercase 'fyi'"
        );
        assert!(
            word_strings.contains(&"lol".to_string()),
            "Should contain lowercase 'lol'"
        );

        // Should NOT contain uppercase or title case
        assert!(
            !word_strings.contains(&"BTW".to_string()),
            "Should NOT contain uppercase 'BTW'"
        );
        assert!(
            !word_strings.contains(&"Btw".to_string()),
            "Should NOT contain title case 'Btw'"
        );
    }

    #[test]
    fn test_real_abbreviations_loaded() {
        // Test that real abbreviations are loaded (lowercase only)
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();

        // Should have 3000+ abbreviations (no case variants)
        assert!(abbrevs.len() > 3000, "Should have 3000+ abbreviations");

        let abbrev_strings: Vec<String> = abbrevs
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Test specific critical abbreviations (all lowercase)
        assert!(
            abbrev_strings.contains(&"btw".to_string()),
            "Should contain 'btw'"
        );
        assert!(
            abbrev_strings.contains(&"fyi".to_string()),
            "Should contain 'fyi'"
        );
        assert!(
            abbrev_strings.contains(&"lol".to_string()),
            "Should contain 'lol'"
        );
        assert!(
            abbrev_strings.contains(&"afaict".to_string()),
            "Should contain 'afaict'"
        );
        assert!(
            abbrev_strings.contains(&"imho".to_string()),
            "Should contain 'imho'"
        );

        // Should NOT contain uppercase variants
        assert!(
            !abbrev_strings.contains(&"BTW".to_string()),
            "Should NOT contain 'BTW'"
        );
        assert!(
            !abbrev_strings.contains(&"AFAICT".to_string()),
            "Should NOT contain 'AFAICT'"
        );

        tracing::debug!(
            "Loaded {} total abbreviations (lowercase only)",
            abbrevs.len()
        );
    }

    #[test]
    fn test_load_it_terminology() {
        let it_terms = WordlistCategory::ITTerminology.load_words();

        // Should have 10000+ terms
        assert!(
            it_terms.len() > 10000,
            "Should have 10000+ IT terms, got {}",
            it_terms.len()
        );

        // Check for some common IT terms
        let term_strings: Vec<String> = it_terms
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Cloud/DevOps terms
        assert!(
            term_strings.contains(&"kubernetes".to_string()),
            "Should contain 'kubernetes'"
        );
        assert!(
            term_strings.contains(&"docker".to_string()),
            "Should contain 'docker'"
        );
        assert!(
            term_strings.contains(&"nginx".to_string()),
            "Should contain 'nginx'"
        );

        // Programming terms
        assert!(
            term_strings.contains(&"javascript".to_string()),
            "Should contain 'javascript'"
        );
        assert!(
            term_strings.contains(&"python".to_string()),
            "Should contain 'python'"
        );
        assert!(
            term_strings.contains(&"api".to_string()),
            "Should contain 'api'"
        );
        assert!(
            term_strings.contains(&"json".to_string()),
            "Should contain 'json'"
        );

        // Networking terms
        assert!(
            term_strings.contains(&"http".to_string()),
            "Should contain 'http'"
        );
        assert!(
            term_strings.contains(&"tcp".to_string()),
            "Should contain 'tcp'"
        );
        assert!(
            term_strings.contains(&"ssh".to_string()),
            "Should contain 'ssh'"
        );

        // Security terms
        assert!(
            term_strings.contains(&"encryption".to_string()),
            "Should contain 'encryption'"
        );
        assert!(
            term_strings.contains(&"firewall".to_string()),
            "Should contain 'firewall'"
        );

        // Linux terms
        assert!(
            term_strings.contains(&"chmod".to_string()),
            "Should contain 'chmod'"
        );

        tracing::debug!("Loaded {} total IT terms", it_terms.len());
    }

    #[test]
    fn test_it_terminology_hyphenated_compounds() {
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let term_strings: Vec<String> = it_terms
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Test that important hyphenated compounds are preserved
        assert!(
            term_strings.contains(&"real-time".to_string()),
            "Should contain 'real-time'"
        );
        assert!(
            term_strings.contains(&"end-to-end".to_string()),
            "Should contain 'end-to-end'"
        );
        assert!(
            term_strings.contains(&"peer-to-peer".to_string()),
            "Should contain 'peer-to-peer'"
        );
        assert!(
            term_strings.contains(&"serverless".to_string()),
            "Should contain 'serverless'"
        );
    }

    #[test]
    fn test_it_terminology_vendor_and_technology_names() {
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let term_strings: Vec<String> = it_terms
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Technology names should be included (both individual and compound forms)
        assert!(
            term_strings.contains(&"apache".to_string()),
            "Should contain 'apache'"
        );
        assert!(
            term_strings.contains(&"kafka".to_string()),
            "Should contain 'kafka'"
        );
        // Compound forms may also be included for common technology stacks
        assert!(
            term_strings.contains(&"apache-kafka".to_string()),
            "Should contain 'apache-kafka'"
        );
    }

    #[test]
    fn test_wordlist_category_info() {
        // Test metadata for all categories
        let internet_info = WordlistCategory::InternetAbbreviations.info();
        assert_eq!(internet_info.name, "Internet Abbreviations");
        assert!(internet_info.word_count_estimate > 3000);

        let genz_info = WordlistCategory::GenZSlang.info();
        assert_eq!(genz_info.name, "Gen Z Slang");
        assert!(genz_info.word_count_estimate > 200);

        let it_info = WordlistCategory::ITTerminology.info();
        assert_eq!(it_info.name, "IT Terminology");
        assert!(it_info.word_count_estimate > 9000);
    }

    // MARK: - WO-01: GrammarEngine test strengthening

    #[test]
    fn test_wordlist_synthetic_coverage() {
        // Test that wordlists contain expected synthetic content
        
        // Test internet abbreviations
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();
        assert!(abbrevs.len() > 3000, "Should have 3000+ abbreviations");
        
        // Check for some common abbreviations that should be present
        let abbrev_strings: Vec<String> = abbrevs
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();
            
        assert!(abbrev_strings.contains(&"btw".to_string()), "Should contain 'btw'");
        assert!(abbrev_strings.contains(&"fyi".to_string()), "Should contain 'fyi'");
        assert!(abbrev_strings.contains(&"lol".to_string()), "Should contain 'lol'");
        
        // Test IT terminology
        let it_terms = WordlistCategory::ITTerminology.load_words();
        assert!(it_terms.len() > 10000, "Should have 10000+ IT terms");
        
        let term_strings: Vec<String> = it_terms
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();
            
        assert!(term_strings.contains(&"kubernetes".to_string()), "Should contain 'kubernetes'");
        assert!(term_strings.contains(&"docker".to_string()), "Should contain 'docker'");
        assert!(term_strings.contains(&"nginx".to_string()), "Should contain 'nginx'");
        
        // Test Gen Z slang
        let slang = WordlistCategory::GenZSlang.load_words();
        assert!(slang.len() > 100, "Should have 100+ slang terms");
        
        let slang_strings: Vec<String> = slang
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();
            
        assert!(slang_strings.contains(&"ghosting".to_string()), "Should contain 'ghosting'");
        assert!(slang_strings.contains(&"sus".to_string()), "Should contain 'sus'");
    }

    #[test]
    fn test_wordlist_lowercase_only_generation() {
        // Test that all wordlists are generated with lowercase only (as required)
        
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();
        let slang = WordlistCategory::GenZSlang.load_words();
        let it_terms = WordlistCategory::ITTerminology.load_words();
        
        // Check that no uppercase variants are present in any wordlist
        let all_words: Vec<String> = abbrevs
            .iter()
            .chain(slang.iter())
            .chain(it_terms.iter())
            .map(|(chars, _)| chars.iter().collect())
            .collect();
            
        for word in &all_words {
            assert_eq!(word.to_lowercase(), *word, "All words should be lowercase only");
        }
    }

    #[test]
    fn test_wordlist_integration_with_analyzer() {
        // Test that wordlists integrate properly with the analyzer
         
        // Test with internet abbreviations
        let result = analyze_text(
            "BTW this is a test.",
            "American",
            true,  // enable_internet_abbrev
            false,
            false,
            false,
            false,
            false,
            false,
            vec![],
            true,
            true,  // enforce_oxford_comma
            true,  // check_ellipsis
            true,  // check_unclosed_quotes
            true,  // check_dashes
        );
        
        assert!(result.word_count > 0, "Should analyze text with internet abbreviations");
         
        // Test with IT terminology
        let result2 = analyze_text(
            "Kubernetes is great.",
            "American",
            false,
            false,
            true,  // enable_it_terminology
            false,
            false,
            false,
            false,
            vec![],
            true,
            true,  // enforce_oxford_comma
            true,  // check_ellipsis
            true,  // check_unclosed_quotes
            true,  // check_dashes
        );
        
        assert!(result2.word_count > 0, "Should analyze text with IT terminology");
    }

    #[test]
    fn test_it_terminology_source_variety() {
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let term_strings: Vec<String> = it_terms
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Terms from different sources to verify comprehensive coverage

        // NIST cybersecurity terms
        assert!(
            term_strings.contains(&"authentication".to_string()),
            "Should contain NIST term 'authentication'"
        );

        // IANA protocols
        assert!(
            term_strings.contains(&"ssh".to_string()),
            "Should contain IANA protocol 'ssh'"
        );

        // Linux syscalls
        assert!(
            term_strings.contains(&"open".to_string()),
            "Should contain Linux syscall 'open'"
        );

        // Programming languages from GitHub Linguist
        assert!(
            term_strings.contains(&"rust".to_string()),
            "Should contain programming language 'rust'"
        );

        // CNCF technologies
        assert!(
            term_strings.contains(&"prometheus".to_string()),
            "Should contain CNCF tech 'prometheus'"
        );
    }

    #[test]
    fn test_load_brand_names() {
        let brands = WordlistCategory::BrandNames.load_words();

        // Should have loaded brand names from Fortune 500 + Forbes 2000 + curated brands
        assert!(brands.len() > 2000, "Should have 2000+ brand names");

        let brand_strings: Vec<String> = brands
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Test special capitalization brands (stored lowercase, Harper matches case-insensitively)
        assert!(
            brand_strings.contains(&"iphone".to_string()),
            "Should contain 'iphone'"
        );
        assert!(
            brand_strings.contains(&"github".to_string()),
            "Should contain 'github'"
        );
        assert!(
            brand_strings.contains(&"ebay".to_string()),
            "Should contain 'ebay'"
        );
        assert!(
            brand_strings.contains(&"macos".to_string()),
            "Should contain 'macos'"
        );

        // Standard brands
        assert!(
            brand_strings.contains(&"apple".to_string()),
            "Should contain 'apple'"
        );
        assert!(
            brand_strings.contains(&"microsoft".to_string()),
            "Should contain 'microsoft'"
        );
        assert!(
            brand_strings.contains(&"amazon".to_string()),
            "Should contain 'amazon'"
        );

        tracing::debug!("Loaded {} brand names", brands.len());
    }

    #[test]
    fn test_load_person_names() {
        let names = WordlistCategory::PersonNames.load_words();

        // Should have loaded many names from US SSA and 106 countries
        assert!(names.len() > 100000, "Should have 100000+ person names");

        let name_strings: Vec<String> = names
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Test common international names (stored lowercase)
        assert!(
            name_strings.contains(&"muhammad".to_string()),
            "Should contain 'muhammad'"
        );
        assert!(
            name_strings.contains(&"maria".to_string()),
            "Should contain 'maria'"
        );
        assert!(
            name_strings.contains(&"ahmed".to_string()),
            "Should contain 'ahmed'"
        );
        assert!(
            name_strings.contains(&"sophia".to_string()),
            "Should contain 'sophia'"
        );
        assert!(
            name_strings.contains(&"oliver".to_string()),
            "Should contain 'oliver'"
        );

        tracing::debug!("Loaded {} person names", names.len());
    }

    #[test]
    fn test_load_last_names() {
        let names = WordlistCategory::LastNames.load_words();

        // Should have loaded many surnames from US Census
        assert!(names.len() > 150000, "Should have 150000+ last names");

        let name_strings: Vec<String> = names
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Test common surnames (stored lowercase)
        assert!(
            name_strings.contains(&"smith".to_string()),
            "Should contain 'smith'"
        );
        assert!(
            name_strings.contains(&"johnson".to_string()),
            "Should contain 'johnson'"
        );
        assert!(
            name_strings.contains(&"williams".to_string()),
            "Should contain 'williams'"
        );
        assert!(
            name_strings.contains(&"garcia".to_string()),
            "Should contain 'garcia'"
        );
        assert!(
            name_strings.contains(&"martinez".to_string()),
            "Should contain 'martinez'"
        );

        tracing::debug!("Loaded {} last names", names.len());
    }

    // MARK: - WO-02: Security/QA Regression Tests (bounded inputs with behavioral assertions)

    #[test]
    fn test_load_words_lowercase_only_edge_cases() {
        // Test load_words_lowercase_only handles edge cases correctly
        let cases = vec![
            ("", 0, "empty string"),
            ("   ", 0, "whitespace only"),
            ("# comment only", 0, "comment only"),
            ("#\n# another comment\n", 0, "multiple comments"),
            ("\n\n\n", 0, "newlines only"),
            ("word", 1, "single word no newline"),
            ("WORD", 1, "uppercase single word"),
            ("wo\0rd", 1, "null byte in middle"), // null byte is preserved but lowercased
        ];

        for (input, expected_count, desc) in cases {
            let words = load_words_lowercase_only(input);
            assert_eq!(words.len(), expected_count, 
                "{}: expected {} words, got {} for '{}'", desc, expected_count, words.len(), input);
            
            // All loaded words must be lowercase
            for (chars, _) in &words {
                let word: String = chars.iter().collect();
                assert_eq!(word.to_lowercase(), word, "Word '{}' must be lowercase", word);
            }
        }
    }

    #[test]
    fn test_load_words_lowercase_only_whitespace_variants() {
        // Various whitespace characters should be trimmed correctly
        let input = "\tBTW\t\n\x0CFYI\r\n\rvia";
        let words = load_words_lowercase_only(input);
        
        assert_eq!(words.len(), 3, "Should extract 3 words from mixed whitespace");
        let word_strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();
        assert!(word_strings.contains(&"btw".to_string()));
        assert!(word_strings.contains(&"fyi".to_string()));
        assert!(word_strings.contains(&"via".to_string()));
    }

    #[test]
    fn test_load_words_lowercase_only_unicode() {
        // Unicode characters should be lowercased correctly
        let input = "HELLO\nWORLD\ncafé";
        let words = load_words_lowercase_only(input);
        
        assert_eq!(words.len(), 3, "Should handle unicode correctly");
        let word_strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();
        assert_eq!(word_strings[0], "hello");
        assert_eq!(word_strings[1], "world");
        assert_eq!(word_strings[2], "café"); // Unicode lowercase is identity for this char
    }

    #[test]
    fn test_wordlist_category_info_all_categories() {
        // All wordlist categories must have valid metadata (non-empty names, reasonable estimates)
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let info = category.info();
            assert!(!info.name.is_empty(), "{:?}: name must not be empty", category);
            assert!(!info.description.is_empty(), "{:?}: description must not be empty", category);
            assert!(info.word_count_estimate > 0, 
                "{:?}: word_count_estimate must be positive", category);
        }
    }

    #[test]
    fn test_wordlist_lowercase_only_all_categories() {
        // All loaded words from every category must be lowercase only
        let categories = vec![
            ("abbreviations", WordlistCategory::InternetAbbreviations),
            ("genz_slang", WordlistCategory::GenZSlang),
            ("it_terms", WordlistCategory::ITTerminology),
            ("brand_names", WordlistCategory::BrandNames),
        ];

        for (name, category) in categories {
            let words = category.load_words();
            assert!(words.len() > 0, "{} must load at least one word", name);
            
            for (chars, _) in &words {
                let word: String = chars.iter().collect();
                assert_eq!(word.to_lowercase(), word, 
                    "Word '{}' from {} is not lowercase", word, name);
            }
        }
    }

    // MARK: - Additional Expansion Tests

    #[test]
    fn test_all_wordlist_categories_loaded() {
        // Test that all 6 wordlist categories load successfully
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let words = category.load_words();
            assert!(words.len() > 0, "{:?} should load at least one word", category);
        }
    }

    #[test]
    fn test_unique_wordlists() {
        // Ensure that different wordlist categories have unique sets of words
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();
        let slang = WordlistCategory::GenZSlang.load_words();
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let brands = WordlistCategory::BrandNames.load_words();

        // Convert to strings for comparison
        let abbrev_strings: Vec<String> = abbrevs.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let slang_strings: Vec<String> = slang.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let it_strings: Vec<String> = it_terms.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let brand_strings: Vec<String> = brands.iter().map(|(chars, _)| chars.iter().collect()).collect();

        // Verify each category has expected minimum counts
        assert!(abbrev_strings.len() > 3000, "Internet abbreviations should have >3000 words");
        assert!(slang_strings.len() > 100, "Gen Z slang should have >100 words");
        assert!(it_strings.len() > 10000, "IT terminology should have >10000 words");
        assert!(brand_strings.len() > 2000, "Brand names should have >2000 words");

        // Verify that different categories contain different content
        assert!(
            !abbrev_strings.contains(&"kubernetes".to_string()),
            "Internet abbreviations should not contain IT terms"
        );
        assert!(
            !it_strings.contains(&"btw".to_string()),
            "IT terminology should not contain internet abbreviations"
        );
    }

    #[test]
    fn test_wordlist_metadata_consistency() {
        // Test that all wordlist categories have proper metadata
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let info = category.info();
            
            // All metadata should be non-empty
            assert!(!info.name.is_empty(), "{:?}: name should not be empty", category);
            assert!(!info.description.is_empty(), "{:?}: description should not be empty", category);
            
            // Word count estimates should be reasonable (positive)
            assert!(info.word_count_estimate > 0, "{:?}: word_count_estimate must be positive", category);
            
            // All categories should have a name
            assert!(
                !info.name.contains("Unknown"),
                "{:?}: name should not contain 'Unknown'", category
            );
        }
    }

    #[test]
    fn test_wordlist_word_count_estimates() {
        // Test that word count estimates match actual counts for all categories
        let categories = vec![
            (WordlistCategory::InternetAbbreviations, 3200),
            (WordlistCategory::GenZSlang, 270),
            (WordlistCategory::ITTerminology, 10000),
            (WordlistCategory::BrandNames, 2400),
            (WordlistCategory::PersonNames, 100000),
            (WordlistCategory::LastNames, 151670),
        ];

        for (category, expected_estimate) in categories {
            let words = category.load_words();
            let info = category.info();
            
            // Word counts should be reasonable
            let lower = (expected_estimate as f64 * 0.8) as usize;
            let upper = (expected_estimate as f64 * 1.2) as usize;
            assert!(
                words.len() > lower,
                "{:?}: actual count {} should be at least 80% of estimate {}",
                category, words.len(), expected_estimate
            );
            
            assert!(
                words.len() < upper,
                "{:?}: actual count {} should be no more than 120% of estimate {}",
                category, words.len(), expected_estimate
            );
            
            // Info metadata should match expectations
            assert_eq!(info.word_count_estimate, expected_estimate, 
                "{:?}: info word_count_estimate should match expected value", category);
        }
    }

    #[test]
    fn test_load_words_lowercase_only_no_duplicates() {
        // Test that no duplicate words are loaded in a single wordlist
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();
        let slang = WordlistCategory::GenZSlang.load_words();
        let it_terms = WordlistCategory::ITTerminology.load_words();
        
        // Check for duplicates within each category
        let abbrev_strings: Vec<String> = abbrevs.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let slang_strings: Vec<String> = slang.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let it_strings: Vec<String> = it_terms.iter().map(|(chars, _)| chars.iter().collect()).collect();

        // Convert to HashSet to check for duplicates
        use std::collections::HashSet;
        let abbrev_set: HashSet<&String> = abbrev_strings.iter().collect();
        let slang_set: HashSet<&String> = slang_strings.iter().collect();
        let it_set: HashSet<&String> = it_strings.iter().collect();

        assert_eq!(
            abbrev_strings.len(), 
            abbrev_set.len(),
            "Internet abbreviations should not contain duplicates"
        );
        assert_eq!(
            slang_strings.len(), 
            slang_set.len(),
            "Gen Z slang should not contain duplicates"
        );
        assert_eq!(
            it_strings.len(), 
            it_set.len(),
            "IT terminology should not contain duplicates"
        );
    }

    #[test]
    fn test_all_wordlists_have_lowercase_only() {
        // Test that all wordlist categories are loaded with only lowercase words
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let words = category.load_words();
            
            // All words should be lowercase
            for (chars, _) in &words {
                let word: String = chars.iter().collect();
                assert_eq!(word.to_lowercase(), word, 
                    "Word '{}' from {:?} is not lowercase", word, category);
            }
        }
    }

    #[test]
    fn test_wordlist_load_performance() {
        // Basic performance check - should load within reasonable time
        let start = std::time::Instant::now();
        
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();
        let slang = WordlistCategory::GenZSlang.load_words();
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let brands = WordlistCategory::BrandNames.load_words();
        
        let duration = start.elapsed();
        
        // Should load within 10 seconds (should be much faster)
        assert!(duration < std::time::Duration::from_secs(10), 
            "Wordlist loading took too long: {:?}", duration);
        
        // Verify counts
        assert!(abbrevs.len() > 3000, "Internet abbreviations should have >3000 words");
        assert!(slang.len() > 100, "Gen Z slang should have >100 words");
        assert!(it_terms.len() > 10000, "IT terminology should have >10000 words");
        assert!(brands.len() > 2000, "Brand names should have >2000 words");
    }

    #[test]
    fn test_wordlist_load_consistency() {
        // Test that loading the same wordlist multiple times gives consistent results
        let category = WordlistCategory::InternetAbbreviations;
        
        let words1 = category.load_words();
        let words2 = category.load_words();
        
        assert_eq!(words1.len(), words2.len(), "Wordlist should load consistently");
        
        // Convert to strings and compare
        let strings1: Vec<String> = words1.iter().map(|(chars, _)| chars.iter().collect()).collect();
        let strings2: Vec<String> = words2.iter().map(|(chars, _)| chars.iter().collect()).collect();
        
        assert_eq!(strings1.len(), strings2.len(), "String representations should match");
    }

    // MARK: - Bounded Expansion Tests (WO-03)

    #[test]
    fn test_metadata_all_non_empty_fields() {
        // All categories must have non-empty metadata fields - structural invariant.
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let info = category.info();
            assert!(!info.name.is_empty(), "{:?}: name must not be empty", category);
            assert!(!info.description.is_empty(), "{:?}: description must not be empty", category);
            assert!(info.word_count_estimate > 0, "{:?}: estimate must be positive", category);

            // Name should not use placeholder patterns
            let placeholder_patterns = ["Unknown", "TBD", "TODO", "XXX"];
            for pattern in placeholder_patterns {
                assert!(
                    !info.name.contains(pattern),
                    "{:?}: name '{}' contains placeholder '{}'",
                    category, info.name, pattern
                );
            }
        }
    }

    #[test]
    fn test_word_count_bounds_percentage() {
        // Word counts should fall within percentage bounds of estimates.
        // Uses relative tolerance (80%-120%) so future wordlist growth/shrinking doesn't break tests.
        let categories: Vec<(WordlistCategory, f64)> = vec![
            (WordlistCategory::InternetAbbreviations, 0.8),
            (WordlistCategory::GenZSlang, 0.5),   // Gen Z slang is smaller, use wider tolerance
            (WordlistCategory::ITTerminology, 0.8),
            (WordlistCategory::BrandNames, 0.8),
            (WordlistCategory::PersonNames, 0.9),  // Larger wordlists have tighter expected bounds
            (WordlistCategory::LastNames, 0.9),
        ];

        for (category, lower_tolerance) in categories {
            let words = category.load_words();
            let info = category.info();

            let min_expected = (info.word_count_estimate as f64 * lower_tolerance) as usize;
            let max_expected = (info.word_count_estimate as f64 * 1.2) as usize;

            assert!(
                words.len() >= min_expected,
                "{:?}: count {} below lower bound {} ({:.0}% of estimate {})",
                category, words.len(), min_expected, lower_tolerance * 100.0, info.word_count_estimate
            );
            assert!(
                words.len() <= max_expected,
                "{:?}: count {} above upper bound {} ({:.0}% of estimate {})",
                category, words.len(), max_expected, 120.0, info.word_count_estimate
            );
        }
    }

    #[test]
    fn test_all_words_lowercase_across_all_categories() {
        // Every word in every loaded wordlist must be pure lowercase (ASCII a-z + digits only for base check).
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let words = category.load_words();
            assert!(words.len() > 0, "{:?}: must load at least one word", category);

            for (chars, _meta) in &words {
                let word: String = chars.iter().collect();
                assert_eq!(
                    word.to_lowercase(),
                    word,
                    "{:?}: word '{}' is not lowercase - expected '{}'",
                    category, word, word.to_lowercase()
                );
            }
        }
    }

    #[test]
    fn test_no_duplicates_in_wordlists() {
        // Each wordlist must have zero internal duplicates.
        use std::collections::HashSet;

        let categories = vec![
            ("abbreviations", WordlistCategory::InternetAbbreviations),
            ("genz_slang", WordlistCategory::GenZSlang),
            ("it_terms", WordlistCategory::ITTerminology),
            ("brand_names", WordlistCategory::BrandNames),
            ("person_names", WordlistCategory::PersonNames),
            ("last_names", WordlistCategory::LastNames),
        ];

        for (name, category) in categories {
            let words = category.load_words();
            let strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();

            let mut seen = HashSet::new();
            for word in &strings {
                assert!(
                    seen.insert(word.clone()),
                    "{:?}: duplicate word '{}' found in {}",
                    category, word, name
                );
            }

            assert_eq!(
                strings.len(),
                seen.len(),
                "{:?}: {} has duplicates ({} unique of {} total)",
                category, name, seen.len(), strings.len()
            );
        }
    }

    #[test]
    fn test_category_isolation_no_cross_contamination() {
        // Different categories should not share words outside of obvious overlap zones.
        let abbrevs: Vec<String> = WordlistCategory::InternetAbbreviations
            .load_words()
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        let it_terms: Vec<String> = WordlistCategory::ITTerminology
            .load_words()
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        let slang: Vec<String> = WordlistCategory::GenZSlang
            .load_words()
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        let brands: Vec<String> = WordlistCategory::BrandNames
            .load_words()
            .iter()
            .map(|(chars, _)| chars.iter().collect())
            .collect();

        // Known IT terms should NOT appear in internet abbreviations
        let it_exclusives = ["kubernetes", "docker", "nginx", "redis", "mongodb"];
        for term in it_exclusives {
            assert!(
                !abbrevs.contains(&term.to_string()),
                "'{}' should not be in internet abbreviations",
                term
            );
        }

        // Known abbreviations should NOT appear in IT terminology
        let abbrev_exclusives = ["btw", "fyi", "afaict", "imho", "tl;dr"];
        for abbr in abbrev_exclusives {
            assert!(
                !it_terms.contains(&abbr.to_string()),
                "'{}' should not be in IT terminology",
                abbr
            );
        }

        // Slang should not contain IT jargon (outside of slang-meaning overlaps)
        let slang_in_it: Vec<&str> = it_exclusives.iter()
            .filter(|t| slang.contains(&(*t).to_string()))
            .copied()
            .collect();
        assert!(
            slang_in_it.is_empty(),
            "SLANG overlap with IT terms (acceptable but worth noting): {:?}",
            slang_in_it
        );
    }

    #[test]
    fn test_load_words_lowercase_only_null_byte_handling() {
        // Null bytes should be handled gracefully (preserved but lowercased).
        let test_text = "word1\x00word2\nWORD3";
        let words = load_words_lowercase_only(test_text);
        assert_eq!(words.len(), 2, "Should produce 2 lines of words");

        for (chars, _) in &words {
            let word: String = chars.iter().collect();
            assert_eq!(word.to_lowercase(), word, "Word '{}' must be lowercase", word);
        }
    }

    #[test]
    fn test_load_words_lowercase_only_leading_trailing_whitespace() {
        // Whitespace around words should be trimmed correctly.
        let test_text = "  BTW  \n\tFYI\t\n   LOL   ";
        let words = load_words_lowercase_only(test_text);

        assert_eq!(words.len(), 3, "Should extract 3 words despite surrounding whitespace");
        let word_strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();
        assert!(word_strings.contains(&"btw".to_string()));
        assert!(word_strings.contains(&"fyi".to_string()));
        assert!(word_strings.contains(&"lol".to_string()));

        // No word should have leading/trailing whitespace after loading
        for (chars, _) in &words {
            let word: String = chars.iter().collect();
            assert_eq!(word.trim(), word, "Word '{}' should not have surrounding whitespace", word);
        }
    }

    #[test]
    fn test_load_words_lowercase_only_hyphenated_compounds() {
        // Hyphenated words should be preserved as single entries (not split).
        let test_text = "end-to-end\nreal-time\npeer-to-peer\nhy-phen-a-ted";
        let words = load_words_lowercase_only(test_text);

        assert_eq!(words.len(), 4, "Should preserve hyphenated compounds");
        let word_strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();
        assert!(word_strings.contains(&"end-to-end".to_string()));
        assert!(word_strings.contains(&"real-time".to_string()));
        assert!(word_strings.contains(&"peer-to-peer".to_string()));
    }

    #[test]
    fn test_wordlist_info_category_matches() {
        // The info().category field must match the category used to call info().
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let info = category.info();
            assert_eq!(
                info.category, category,
                "WordlistInfo.category must match the calling category"
            );
        }
    }

    #[test]
    fn test_wordlist_minimum_counts_structural() {
        // Structural bounds: each wordlist must have a minimum threshold that reflects its purpose.
        // These are absolute minimums to catch accidental truncation, not exact counts.
        let bounds: Vec<(WordlistCategory, usize)> = vec![
            (WordlistCategory::InternetAbbreviations, 100),    // At least 100 abbreviations
            (WordlistCategory::GenZSlang, 30),                  // At least 30 slang terms  
            (WordlistCategory::ITTerminology, 500),             // At least 500 IT terms
            (WordlistCategory::BrandNames, 100),                // At least 100 brands
            (WordlistCategory::PersonNames, 1000),              // At least 1000 names
            (WordlistCategory::LastNames, 5000),                // At least 5000 surnames
        ];

        for (category, min_count) in bounds {
            let words = category.load_words();
            assert!(
                words.len() >= min_count,
                "{:?}: loaded {} words, expected at least {}",
                category, words.len(), min_count
            );
        }
    }

    #[test]
    fn test_all_categories_have_wordlist_files() {
        // All WordlistCategory variants must have corresponding embedded wordlist data.
        // This catches compile-time include_str! panics if a file goes missing.
        let categories = vec![
            WordlistCategory::InternetAbbreviations,
            WordlistCategory::GenZSlang,
            WordlistCategory::ITTerminology,
            WordlistCategory::BrandNames,
            WordlistCategory::PersonNames,
            WordlistCategory::LastNames,
        ];

        for category in categories {
            let words = category.load_words();
            assert!(
                !words.is_empty(),
                "{:?}: load_words must not return empty vec (wordlist file may be missing)",
                category
            );
        }
    }

    #[test]
    fn test_load_words_lowercase_only_case_preservation_patterns() {
        // Verify that common abbreviation patterns are lowercased correctly.
        let test_text = "HTTP\nHTTPS\nJSON\nXML\nURL\nAPI\nREST";
        let words = load_words_lowercase_only(test_text);

        assert_eq!(words.len(), 7, "Should load all 7 entries");
        let word_strings: Vec<String> = words.iter().map(|(chars, _)| chars.iter().collect()).collect();

        let expected_lower = vec!["http", "https", "json", "xml", "url", "api", "rest"];
        for (i, expected) in expected_lower.iter().enumerate() {
            assert_eq!(word_strings[i], *expected, "Entry {} should be '{}'", i, expected);
        }

        // Verify none contain uppercase
        for word in &word_strings {
            let has_upper = word.chars().any(|c| c.is_ascii_uppercase());
            assert!(!has_upper, "Word '{}' contains uppercase characters", word);
        }
    }

    #[test]
    fn test_wordlist_count_reasonableness() {
        // Cross-check that counts are in a reasonable range relative to each other.
        let last_names = WordlistCategory::LastNames.load_words();
        let person_names = WordlistCategory::PersonNames.load_words();
        let it_terms = WordlistCategory::ITTerminology.load_words();
        let abbrevs = WordlistCategory::InternetAbbreviations.load_words();

        // Sanity: surnames should exceed person names in count (US Census covers common + rare)
        assert!(
            last_names.len() >= person_names.len(),
            "Last names ({}) should be >= person names ({})",
            last_names.len(),
            person_names.len()
        );

        // IT terms should significantly outnumber abbreviations
        assert!(
            it_terms.len() > abbrevs.len(),
            "IT terms ({}) should exceed abbreviations ({})",
            it_terms.len(),
            abbrevs.len()
        );
    }
}
