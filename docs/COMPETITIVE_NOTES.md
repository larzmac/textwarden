# Competitive Notes: TextWarden vs LanguageTool Engine

## Overview

TextWarden is a macOS grammar checking application that offers multiple grammar engines through its EngineRouter:
- **Harper engine** (built-in): Fast, local Rust-based engine with curated rules
- **LanguageTool engine** (local server): More comprehensive rule coverage including homophone confusions 
- **Auto mode**: Prefers LanguageTool when available, falls back to Harper

## TextWarden's Grammar Engine Architecture

### EngineRouter Pattern
The `EngineRouter` acts as a facade that routes grammar analysis requests to the selected engine:
- `.auto`: LanguageTool server when reachable, Harper fallback with failure cooldown (30s)
- `.languageTool`: Direct LanguageTool server usage with Harper fallback 
- `.harper`: Pure Harper engine only

### Harper Engine Features
The Harper engine provides:
- Built-in Rust-based grammar checking
- Fast analysis (typically < 100ms for typical documents)
- Custom vocabulary support (internet abbreviations, Gen Z slang, IT terminology, brand names, person names, last names)
- Language detection and filtering for non-English text
- Integration with macOS system dictionary (NSSpellChecker)
- Optimized for emoji handling and Unicode safety

### LanguageTool Engine Features  
The LanguageTool engine provides:
- More comprehensive rule coverage than Harper alone
- Homophone confusions like "their/they're" 
- Support for 30+ languages (though TextWarden focuses on English)
- Statistical confused-word detection using 14GB English n-gram data
- Integration with the local LanguageTool server via HTTP API
- UTF-16 offset conversion to Unicode-scalar indices

## Performance Comparison

### Harper Engine
- **Speed**: Very fast, typically < 100ms for typical documents
- **Memory**: Minimal (Rust engine optimized)
- **Latency**: Predictable and low
- **Resource usage**: Efficient

### LanguageTool Engine
- **Speed**: Slower than Harper due to network overhead (~200-500ms depending on server)
- **Memory**: Higher due to server process
- **Latency**: Network-dependent (server must be running locally)
- **Resource usage**: Moderate to high

## Feature Comparison

### Grammar Checking Coverage
| Feature | Harper | LanguageTool |
|---------|--------|--------------|
| Spelling errors | ✅ | ✅ |
| Grammar rules | ✅ | ✅ |
| Punctuation issues | ✅ | ✅ |
| Capitalization | ✅ | ✅ |
| Word choice | ✅ | ✅ |
| Homophone confusions | ❌ | ✅ |
| Statistical error detection | ❌ | ✅ |
| 30+ language support | ❌ | ✅ |

### Custom Vocabulary Support
| Feature | Harper | LanguageTool |
|---------|--------|--------------|
| Internet abbreviations | ✅ | ✅ |
| Gen Z slang | ✅ | ✅ |
| IT terminology | ✅ | ✅ |
| Brand names | ✅ | ✅ |
| Person names | ✅ | ✅ |
| Last names | ✅ | ✅ |

### Integration & Usability
| Feature | Harper | LanguageTool |
|---------|--------|--------------|
| Local-only processing | ✅ | ✅ |
| No cloud requirements | ✅ | ✅ |
| Privacy-focused | ✅ | ✅ |
| Setup complexity | Low | Medium (requires local server) |
| Automatic fallback | ✅ | ❌ |

## Advantages of TextWarden's Approach

1. **Dual-engine architecture**: Provides best of both worlds - Harper for speed and LanguageTool for coverage
2. **Automatic fallback**: No need to manually switch engines when LanguageTool is unavailable
3. **Network resilience**: Failure cooldown prevents repeated slow operations
4. **Emoji-safe indexing**: Proper handling of Unicode characters in text positioning 
5. **App-specific tuning**: Per-application behavior configurations for optimal performance
6. **Privacy-first**: All processing happens locally on device

## Limitations

### Harper Engine
- Less comprehensive rule coverage than LanguageTool
- Limited statistical error detection capabilities

### LanguageTool Engine  
- Requires local server installation and management
- Network latency overhead
- Higher resource consumption
- Manual engine selection required for full control

## Recommendation

For users seeking a balance between speed and comprehensive grammar checking:
1. **Use Auto mode** (default) for automatic optimization
2. **Use LanguageTool engine** when homophone confusion detection is critical  
3. **Use Harper engine** when maximum speed and minimal resource usage are priorities

The TextWarden approach of combining both engines provides flexibility while maintaining the privacy and performance benefits that users expect from a local grammar checking application.