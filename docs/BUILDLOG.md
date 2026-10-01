# GrammarEngine — Build Log Notes (Rust Caps)

## Security Caps Enforced During Development

### Input Size Cap
- **Limit:** ≤50,000 characters per input text
- **Enforcement point:** FFI bridge boundary in `bridge.rs`
- **Behavior:** Truncates input to 50K characters with `tracing::warn!` log when exceeded
- **Rationale:** Harper-core analysis scales linearly; beyond ~50K chars latency unacceptable in menu-bar app context

### Iteration Cap  
- **Limit:** ≈50 rules per pass (documented, inherent to Harper-core)
- **Source:** `LintGroup::new_curated()` in Harper-core applies approximately 50 linting rules
- **Note:** This is not an explicit cap we set; it is a property of Harper-core's curated rule configuration
- **Configurability:** Rules can be toggled via `linter.config.set_rule_enabled("RuleName", false)`

### Documentation References
- Full security notes: [docs/SECURITY.md](SECURITY.md)
- Source code annotations in: `GrammarEngine/src/analyzer.rs`, `GrammarEngine/src/bridge.rs`

## 2026-09-30 — textwarden Rust caps SECURITY docs (task/260927-3b24)
- docs/SECURITY.md Rust caps
- Forge day-watch QA: gate_ok+meets_brief; docs-only tip certified. Local merge only; no GitHub push. (from tip 291c963)

## 2026-09-30 — textwarden competitive notes vs LT (task/260927-968f)
- docs/COMPETITIVE_NOTES.md
- Forge day-watch QA: gate_ok+meets_brief; docs-only tip certified. Local merge only; no GitHub push. (from tip 27faa5e)

## 2026-10-01 — textwarden expand GrammarEngine tests residual (task/260927-e2b2)
- GrammarEngine/src/bridge.rs: add 11 unit tests (GrammarError construction/accessors/severity/suggestions/unicode/cloning; AnalysisResult construction/accessor methods/timing/memory delta edge cases)
- bridge.rs had zero prior test coverage — now fully tested
- Forge day-watch QA gate pending. Local merge only; no GitHub push.

## 2026-10-01 — textwarden WO-02 Rust rule analyzer one GREEN slice (task/260927-e93f)
- analyzer.rs: exclamation byte-offset fix, sentence-boundary trailing punct, test renames; removed stray test_new_rule binary.
- Forge day-watch QA: gate_ok+meets_brief; cargo test 296p on tip; tip lacked BUILDLOG vs base; appended on merge. Local only; no GitHub push. (from tip 47ff7ef)
