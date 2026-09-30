# GrammarEngine — Security Caps

## Input Size Cap
- **Limit:** ≤50,000 characters per input text
- **Enforcement point:** FFI bridge boundary in `bridge.rs`
- **Behavior:** Truncates input to 50K characters with `tracing::warn!` log when exceeded
- **Rationale:** Harper-core analysis scales linearly; beyond ~50K chars latency unacceptable in menu-bar app context

## Iteration Cap  
- **Limit:** ≈50 rules per pass (documented, inherent to Harper-core)
- **Source:** `LintGroup::new_curated()` in Harper-core applies approximately 50 linting rules
- **Note:** This is not an explicit cap we set; it is a property of Harper-core's curated rule configuration
- **Configurability:** Rules can be toggled via `linter.config.set_rule_enabled("RuleName", false)`

## Source Code References
- Core implementation: `GrammarEngine/src/analyzer.rs`, `GrammarEngine/src/bridge.rs`