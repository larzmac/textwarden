# textwarden (fork) — Build Plan for local agents

Read `LOCAL-AGENT-GUIDE.md`, then `AGENTS.md` (coding rules) and
`ARCHITECTURE.md` before structural changes — both bind. Default branch
is `languagetool-engine`; that is what task branches fork from.

**Gate (must be green before RESULT.json):** `make test-rust`
Baseline 2026-08-28: 137 passed, 0 failed, 10 ignored (~30s warm).
Swift tests (`make test-swift`) need an Xcode build — clones don't run
them; Swift-touching work is YELLOW for that reason.

## Ratings

- **GREEN** — attemptable unattended; verified by `make test-rust`.
- **YELLOW** — proposal branch; human walks the diff and runs the Swift
  side (`make test-swift`, or a real `make run`) before merge.
- **RED** — architect-only. Local agents refuse.

## Work orders

### WO-01 — GrammarEngine test strengthening  `GREEN` (smoke)
Tests only in `GrammarEngine`: analyzer rules, language detection,
dictionary/possessive filters — synthetic sentences as fixtures. No
`src` changes, no existing test removed. Gate: `make test-rust`.

### WO-02 — Rust-side rule and analyzer work  `GREEN`
Changes confined to `GrammarEngine` (no FFI signature changes), each
with tests proving the new behavior. Respect AGENTS.md Rust rules:
`Result` + `?`, no panics in library code, log before returning errors.

### WO-03 — Swift app changes  `YELLOW`
Authorable in a clone; verified only by an Xcode build/test the
architect runs. AGENTS.md Swift rules bind (no force unwraps on AX
data, Logger not print, @MainActor for UI state).

### WO-04 — Browser extension (Safari/Chrome) changes  `YELLOW`

### WO-05 — FFI boundary changes  `RED`
swift-bridge signatures ship Rust + Swift together, architect-only —
a one-sided change compiles in the clone and breaks the app.

### WO-06 — Entitlements, signing, `Info.plist` versioning,
`appcast.xml` (update feed), release/CI plumbing, Accessibility
permission flows  `RED`
