# textwarden (fork) — Local Agent Guide

1. **One work order, one `task/<id>` branch**, forked from
   `languagetool-engine` (the default branch). Never push to shared
   branches, never tag, never touch `appcast.xml` — it is the live
   Sparkle update feed for an installed app.
2. **AGENTS.md and ARCHITECTURE.md are law.** Logger not print(); no
   force unwraps on AXValue or external data; Result + `?` in Rust, no
   library panics; FFI boundary stays minimal and is architect-only.
3. **Rust is your home turf.** `GrammarEngine` changes are provable with
   `make test-rust` — that is the gate. Swift you may edit only when the
   WO says so, and you must report it as "unverified pending architect
   build", because clones cannot run `make test-swift`.
4. **Never run the app.** No `make build`, `make run`, `make install`,
   or xcodebuild in a clone — it installs to /Applications and requests
   Accessibility permissions on Larz's Mac.
5. **Determinism + honesty.** Seed anything random in tests; never
   delete or weaken a test to pass the gate; blocked questions are one
   plain-English sentence a non-developer can answer, in
   `.studio/RESULT.json`.
