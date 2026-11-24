# HANDOFF — TextWarden (Larz's fork)

> **Development is PAUSED (2026-11-24, by Larz).** Grok Bot (Chief of Staff) wrote this file so that someone with no prior knowledge, human or AI, can pick the project up. It is a snapshot as of `main` = `a782967` (2026-10-30). Where another doc in this repo disagrees with this one, this file has the newer facts as of 2026-11-24, and section 12 lists the known stale spots.

- **GitHub:** `larzmac/textwarden` · **local checkout:** `~/AI/studio/repos/textwarden`
- **Status:** Rank 11, NEVER-START and parked. Paused 2026-11-24. NEVER push or PR to upstream (PhilipSchmid/textwarden).
- **Priority:** rank 11 of 11 · Lane A

## 0. Read these next (existing docs; this file links rather than repeats them)

- [`README.md`](README.md)
- [`AGENTS.md`](AGENTS.md)
- [`CLAUDE.md`](CLAUDE.md)
- [`LOCAL-AGENT-GUIDE.md`](LOCAL-AGENT-GUIDE.md)
- [`ARCHITECTURE.md`](ARCHITECTURE.md)
- [`BUILD-PLAN.md`](BUILD-PLAN.md)
- [`BUILD.md`](BUILD.md)
- Cross-repo overview: `~/AI/studio/HANDOFF-INDEX.md` (also in the local-studio repo as `HANDOFF-INDEX.md`)

## 1. What this is and who it's for

Larz's personal fork of the TextWarden macOS grammar checker, as a free Grammarly replacement. The fork adds a local LanguageTool engine with automatic Harper fallback (branch `languagetool-engine`). Everything runs locally.

## 2. Architecture and key files

- Swift macOS menu-bar app (`Sources/`, `TextWarden.xcodeproj`), the Rust `GrammarEngine/` via swift-bridge FFI, `BrowserExtension/` and `SafariExtension/`.
- Docs: `README.md`, `ARCHITECTURE.md`, `BUILD.md`, `CONFIGURATION.md`, `TROUBLESHOOTING.md`, `CONTRIBUTING.md`, `CLAUDE.md`/`AGENTS.md`, `BUILD-PLAN.md`.

## 3. Stack, setup, run, test, smoke

**Stack:** Swift/SwiftUI, Rust, Xcode, LanguageTool (`brew services start languagetool`, port 8081).

- Build: `make build`. Tests (registry `test_cmd`): `make test-rust`. Everything: `make help`

Studio registry entry (`~/AI/studio/repo/config/projects.json`):

- `test_cmd`: `export PATH=/opt/homebrew/bin:/usr/local/bin:/Users/larz/.cargo/bin:$PATH && make test-rust`
- Protected files (need Larz's own words to change): `appcast.xml`, `Info.plist`, `TextWarden.entitlements`, `.github/workflows/*`
- onboarded: True

## 4. Data and storage (paths only, never contents)

- No user data in the repo. `appcast.xml`, `Info.plist` and `TextWarden.entitlements` are PROTECTED.

## 5. Current state

- Never-start: don't build. Remotes: `origin` = larzmac/textwarden (Larz's fork), `upstream` = PhilipSchmid/textwarden (never push), plus an `icloud` mirror remote.

**Last commits on main:**

```
a782967 2026-10-30 docs(BUILDLOG): fork work moved to main (Larz t62u; Forge GREEN 013256b)
013256b 2026-10-30 Forge review: fix UB in swift_logger test; console.json cwd to new clone
2155833 2026-10-01 task 260927-e93f: textwarden WO-02 Rust rule analyzer one GREEN slice
8d01989 2026-10-01 task 260927-e93f: textwarden WO-02 Rust rule analyzer one GREEN slice
c7a41d2 2026-10-01 task 260927-e2b2: textwarden WO-01 GrammarEngine tests expand residual
94111f2 2026-09-30 docs(BUILDLOG): Forge QA append for 260927-968f
c82ef9f 2026-09-30 task 260927-968f: textwarden competitive notes vs LT engine docs
1fdda4c 2026-09-30 docs(BUILDLOG): Forge QA append for 260927-3b24
```

**Latest tags:** `release-20261030-2120`, `pre-push-20261030-2120`, `v0.3.1`, `v0.3.1-beta.1`

**Queue snapshot** (`~/AI/studio/repo/queue/tasks.jsonl`, latest event per id): abandoned: 5, failed: 1, merged: 16, parked: 33

Most recent tasks (id · status · title). The branch for a worker task is `task/<id>`:

- `260922-020c` · parked · textwarden language_filter hardening (2026-10-12T23:37) — _This build failed its checks twice; the architect will investigate — nothing for you to do._
- `260922-8f01` · parked · textwarden FFI contract restore if needed (2026-10-12T23:37) — _This build keeps tripping mechanical review; the architect will look - nothing for you to do._
- `260924-d49d` · parked · textwarden WO-01 GrammarEngine test strengthening GREEN (2026-10-12T23:37) — _This build failed its checks twice; the architect will investigate — nothing for you to do._
- `260924-5515` · parked · textwarden FFI contract restore (2026-10-12T23:37) — _This build keeps tripping mechanical review; the architect will look - nothing for you to do._
- `260927-0404` · parked · textwarden rule id stability snapshot tests (2026-10-12T23:37) — _This build failed its checks twice; the architect will investigate — nothing for you to do._
- `261001-644a` · parked · textwarden WO-01 GrammarEngine coverage expand (2026-10-12T23:37) — _deferred until higher-priority repos exhausted (Larz 2026-10-01)_
- `261001-05c1` · parked · textwarden WO-02 analyzer one rule improvement (2026-10-12T23:37) — _deferred until higher-priority repos exhausted (Larz 2026-10-01)_
- `261001-6dc2` · parked · textwarden WO-02 false-positive fixture add (2026-10-12T23:37) — _deferred until higher-priority repos exhausted (Larz 2026-10-01)_

Open `queued`/`failed` tasks (mostly older than the 2026-11-09 queue rule, so they're NOT the work unless they match the lowest roadmap number):

- `260927-e710` · failed · textwarden analyzer error path unit tests

Branches on `origin` (3): `origin/languagetool-engine`, `origin/main`, `origin/task/260909-088f`. Other remotes (for example `icloud`) are old mirrors and not the authority; GitHub `origin/main` is.

## 6. Decisions

**Made by Larz (don't re-litigate):**

- Never push upstream. Never-start.

**Waiting on Larz:**

- None.

## 7. Known bugs and Forge findings

- None current.

## 8. Next steps (roadmap order)

This repo has no active roadmap QUEUE in the studio. It is never-start, parked or closed, so nothing should be queued. If Larz ever unparks it, he will add a numbered QUEUE first. Any planning docs it has (`BUILD-PLAN.md`, `docs/ROADMAP.md`) are design history.

## 9. Process rules every builder must follow

Larz (the owner) approves anything that is sent, published, spent, traded, deployed, restarted, or turned into a calendar invite. Builders work only inside these rules:

1. **Only roadmap QUEUE work.** Take the lowest unfinished number in this repo's queue file (see section 8), and only one number at a time per repo. If the behavior is already on main, write a single BUILDLOG line `XX-NN already done` and move to the next number. Don't invent numbers past the end; when the list is done, stay idle. The rules are in `~/AI/studio/repo/queue/PRIORITY.md`.
2. **Only Larz edits the ROADMAP / BUILD-PLAN queue file.** Builders never edit it, and build briefs never name the roadmap file.
3. **Every tip/brief carries a `Notice:` line** copied verbatim from the roadmap item, saying what a person would notice. A tip is never tests-only, and it must name app files. "The suite is green" is not acceptance.
4. **Branches:** work on `task/<queue-id>` (worker) or `task/cos-<date>-<item>` (CoS). Never commit to main directly.
5. **BUILDLOG.md is append-only at end of file.** Never edit earlier entries.
6. **Review chain:** builder → CoS review → **Forge** review (verdict GREEN or FIX, written to `~/bots/forge/review-<item>-<sha>-<date>.md`). A FIX always goes back to the builder. Ping Forge directly when a review is needed.
7. **Push only with the script:** `python3 ~/AI/studio/bin/push_reviewed_main.py --project <name>` (a dry run first, then `--push`). Before that: `tools/forge_green.py` record and check must show **ELIGIBLE**, and you back up `~/AI/studio/bin/push_approvals.json` and the queue. The script creates `pre-push-*` and `release-*` tags and fast-forwards main only. Afterwards, mark the queue task `merged` (back up `tasks.jsonl` first) and list the merge in the nightly packet.
8. **Never:** `--force`, `--no-verify`, `--skip-tests`, a hand-set `STUDIO_PUSH_REVIEWED`, a hard reset without Larz's confirmation, opening any `.env`, touching Keychain or `~/.claude`, or putting secrets, keys, real financial data, health data or sensitive Overcome content in code, prompts, logs or docs. Protected-file waivers need Larz's own words.
9. **Supervised repos** (mac-money, larz-pass, credit-freeze) need Larz's own ship-it before merge. A card pick counts; cite it as a card answer.
10. **Three strikes → Grok prompt** (Larz, 2026-11-19): when an item fails three times, CoS doesn't build it by hand. It writes a self-contained build prompt for Grok (the app) to `~/AI/studio/grok-prompts/<date>-<item>.md`, with no secrets, keys, real data or sensitive content, and parks the task with a pointer to the prompt. What Grok returns still goes through CoS review, Forge GREEN and the push script.
11. **One build per repo** at a time. Lanes: Lane A takes odd priority ranks, Lane B even ranks (see `~/AI/studio/HANDOFF-INDEX.md`).

## 10. Pending Grok build prompt

- None for this repo. (Pending prompts elsewhere: `~/AI/studio/grok-prompts/20261120-ST-06.md`, `20261120-MM-05.md`, `20261120-CN-03.md`.)

## 11. How to resume after the pause

Development was paused by Larz on 2026-11-24 ("Pause development. Document everything in each Repo's git…"). The full record and the exact resume commands are in `~/AI/studio/PAUSED.md`. In short:

1. Get Larz's explicit go-ahead to resume.
2. Re-enable the studio pipeline (worker, worker2, gate-merge-ready, qa-cycle LaunchAgents) with the commands in `~/AI/studio/PAUSED.md`, and re-enable the CoS "Keep builders busy" routine.
3. In this repo: `git fetch origin && git checkout main && git merge --ff-only origin/main`, then confirm `git rev-parse main` equals `origin/main`.
4. Re-read sections 5 and 8 here, then check the queue (`python3 ~/AI/studio/repo/tools/task.py list`) before filing the lowest unfinished number. If a Grok prompt is pending (section 10), wait for Larz to bring Grok's result back, and then review it.

## 12. Stale docs corrected here

- Nothing known to be stale.
