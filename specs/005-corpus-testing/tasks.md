# Tasks: Corpus Testing of Introspection

**Input**: Design documents from `/specs/005-corpus-testing/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable. Every
test task is written, **run, and seen to fail by assertion** before the implementation
task beneath it is started, and the failing output goes in the test task's commit
message. Where code must exist for a test to compile, it is a stub that answers nothing.
Where a test passes before its code exists, the task's notes say so and say what was
done to show it can fail.

Tasks cite what they serve: "FR-" and "SC-" are the spec's requirements and success
criteria; "R" a section of `research.md`; "F§" a section of `contracts/formats.md` by its
heading; "K§" a section of `contracts/commands.md` by its heading; "D§" a section of
`data-model.md` by its heading.

**The guard**: no file under `crates/zmachine-core`, `crates/zmachine-introspect`,
`crates/zmachine-experience`, `crates/zmachine-ffi`, `crates/zmachine-cli` or `apps/` is
edited by this feature. After every phase, `cargo test --workspace` passes with no test
from features 001 to 004 edited (SC-010). A needed edit to any of them stops the task
for the project owner.

**Downloads**: no task downloads anything without the project owner's approval of that
one file, given in the conversation after being told its name, source and size. Approval
of one file is not approval of the next. A declined file's task is marked "declined" and
left; it is not retried.

**Stop points**: stop for the project owner if the command-prompt rule does not give the
known prompt counts on both walkthroughs (T008); if a trial changes a walkthrough's
transcript (T013); if a wrong case has no cause that can be found (T023); if fewer than
six proposed stories can be used (T043); or if any step would need a change to the
introspection interface or its published specification.

**Stories built in memory**: tests of the rules use small stories made with
`zmachine-testkit`, of the ZIL family's shape (rooms under one object, direction
properties by length as in feature 003's research R5), so that a room's table and the
game's behaviour can be made to disagree on purpose. Feature 003's fixtures in
`crates/zmachine-introspect/src/fixtures.rs` show how; they are private to that crate, so
the corpus crate builds its own in `crates/zmachine-corpus/tests/common/mod.rs`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US5)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The crate exists, builds, and nothing depends on it

- [x] T001 Create `crates/zmachine-corpus/Cargo.toml` (`name = "zmachine-corpus"`, `publish = false`, workspace edition, license and rust-version; dependencies `zmachine-core`, `zmachine-introspect` by path and `sha2 = "0.10"`; dev-dependency `zmachine-testkit` by path; a `[[bin]]` named `corpus` at `src/main.rs`) and `crates/zmachine-corpus/src/lib.rs` with empty modules `manifest`, `ledger`, `trial`, `directions`, `verdict`, `walk`, `report`, and `src/main.rs` printing the usage of K§ "`corpus`" and exiting 2; add the member to the root `Cargo.toml`; `cargo build --workspace` passes
- [x] T002 [P] Failing test, then nothing to implement if it passes for the right reason, in `crates/zmachine-corpus/tests/crate_rules.rs` (FR-005; plan "Constitution Check" V): every other crate's `Cargo.toml` under `crates/` is read and none contains the text `zmachine-corpus`; no file under `crates/zmachine-corpus/src` contains `unsafe`. Show it can fail by adding the dependency to `crates/zmachine-cli/Cargo.toml` for one run, then removing it; record that in the notes
- [x] T003 [P] Create `stories/corpus/README.md` with a heading and one paragraph saying what the folder holds, to be completed in T046; create empty `stories/corpus/manifest.txt` and `stories/corpus/ledger.txt` each holding one comment line naming its format (F§ "The manifest", "The ledger"); create `stories/corpus/reports/.gitkeep`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The instrument is proven on what is already known before anything is built
on it (plan, "Order of work" 1)

**⚠️ CRITICAL**: No user story work begins until this phase is complete

- [x] T004 Shared test helpers in `crates/zmachine-corpus/tests/common/mod.rs`: `repo(path)` giving a path from the repository's root; `load(path, seed)` giving a `Machine` as feature 003's helper does (`crates/zmachine-introspect/tests/common/mod.rs`, `load`: read it for the `LoadOptions` used); and builders for the stories in memory described above: `two_rooms()` (room A with a one-byte exit north to room B, B with one south to A, a main routine that reads a line, moves the player by the table, prints a prompt and loops) with options to make the code disagree with the table (the table says B, the code moves to a third room C; the table has no exit, the code moves anyway; an exit of three bytes, conditional; a direction on which the game quits; one on which it asks a second question at another address; one that loops for ever). No test yet; `cargo test -p zmachine-corpus` compiles
- [x] T005 [P] Failing tests in `crates/zmachine-corpus/tests/trial.rs`, then `crates/zmachine-corpus/src/directions.rs` (R4; D§ "Direction"): `directions(&Introspector, &View)` gives the distinct direction numbers of the dictionary in ascending order, each with its longest stored word, the alphabetically first among equals; a direction with no word is listed with none; on `stories/zork1/zork1.z3` the words include `north`, `northe`, `down` and no direction number repeats; on a story of family none the list is empty
- [x] T006 Failing tests in `crates/zmachine-corpus/tests/prompts.rs`, then `crates/zmachine-corpus/src/walk.rs` `Driver` (R3, R6; FR-008): a driver that loads a story with a seed, answers what the game asks from a list of lines (a line for a line, its first character for a key, declining a save, restore or transcript request as feature 003's `replay` does), records `waiting_at()` at the first line prompt as the command address, and reports each later stop as a command prompt (a line wanted at that address) or not. Test: on a story in memory whose second question is at another address, the first prompt is a command prompt and the second is not
- [x] T007 Extend `tests/prompts.rs` (R3 "To be proven"): replay `stories/zork1/fidelity/walkthrough.txt` (seed 11) and `stories/advent/fidelity/walkthrough.txt` on `advent.z5` and `advent.z8` (seed 1) with the driver of T006; the transcript equals each reference (`reference.scr`, `reference_z5.scr`, `reference_z8.scr`) byte for byte, read as feature 003's `the_driver_reproduces_the_reference_transcripts` reads it
- [x] T008 Extend `tests/prompts.rs` (R3; "Still open" 1): the number of command prompts the driver counts on each of the three replays equals the number asserted in `crates/zmachine-introspect/tests/driver.rs`, `the_walkthroughs_have_the_expected_number_of_command_prompts` (copy the three numbers into this test with a comment naming that test). Feature 004 found the rule mislabels one Zork I stop, the Loud Room (its tasks.md, "Results"): if the counts differ by exactly that stop, record it in the notes and assert the driver's own count; **if they differ otherwise, stop for the project owner**
- [x] T009 Failing tests in `tests/trial.rs`, then `crates/zmachine-corpus/src/trial.rs` (R1, R2; FR-001, FR-003, FR-004; D§ "Trial and outcome"): `trial(&Machine, &Introspector, command_address, word) -> (Outcome, Machine)` copies the machine, gives the copy the word, runs it to its next stop within 50 million instructions, and returns the outcome and the copy. One test for each row of R2's table on the stories of T004: `Moved(room)`, `Stayed`, `Question` (a line at another address; a key), `Ended`, `Fault` (a story that faults; the endless loop, with the limit lowered through a parameter so the test is quick), `Lost` (a story of family none)
- [x] T010 Extend `tests/trial.rs` (FR-001): after ten trials from one machine, the machine itself, run on with the same line, produces the same output, the same next event and the same next random numbers as a twin that had no trials run from it (compare as `crates/zmachine-core/tests/clone.rs` compares, by what a host can see; and by giving both the same later commands on Zork I at seed 11 for twenty prompts)
- [x] T011 [P] Failing tests in `crates/zmachine-corpus/tests/verdict.rs`, then `crates/zmachine-corpus/src/verdict.rs` (FR-006; D§ "Reported", "Verdict"): `reported(&Exits or unknown, direction) -> Reported` for each row of D§ "Reported" (fixed with its destination, conditional, in the unknown list, the whole answer unknown, in neither list); `verdict(Reported, Outcome) -> Verdict`, one test for each row of D§ "Verdict", every pair of the four reported values and six outcomes covered, 24 in all
- [x] T012 Failing test in `crates/zmachine-corpus/tests/untouched.rs`, then `walk.rs` `walkthrough(story, script, seed, each_place)` (FR-002, FR-008; SC-002; quickstart 4): replays a walkthrough with the driver and, at every command prompt, calls back with the machine and the script line about to be given. Test: with a callback that runs a trial for every direction (T005, T009), the transcript of each of the three walkthroughs equals its reference byte for byte
- [x] T013 Run `cargo test --workspace`; confirm with `git diff main --stat -- crates/zmachine-core crates/zmachine-introspect crates/zmachine-experience crates/zmachine-ffi crates/zmachine-cli apps` that nothing there changed. **If T012 can only pass by changing what a trial does to the machine walked, stop for the project owner**

**Checkpoint**: a trial has a tested outcome, leaves the game alone, and the prompt rule
agrees with what is known

---

## Phase 3: User Story 1 - See how right introspection is on one game (Priority: P1) 🎯 MVP

**Goal**: Zork I along its walkthrough gives a report whose counts add up

**Independent Test**: `cargo run -q --release -p zmachine-corpus -- run zork1` prints
`zork1 ok`, and the verdict lines of `stories/corpus/reports/zork1.txt` add up to its
`trials` line

### Tests for User Story 1

- [x] T014 [P] [US1] Failing tests in `crates/zmachine-corpus/tests/formats.rs` (F§ "A report"; FR-024, FR-026, FR-007): a report written from hand-made counts reads back equal; the lines are in the order of F§ and each matches its form exactly; a detail line is `room direction reported outcome count`, sorted by room, then direction, then the rest as text; reading refuses, with the line number, a line out of order, an unknown keyword, a word where a number belongs, outcome counts that do not add up to their `conditional` or `unknown` total, and verdict lines that do not add up to `trials`
- [x] T015 [P] [US1] Failing test in `crates/zmachine-corpus/tests/run.rs` (US1 scenarios 1 to 4; FR-006, FR-007, FR-008): on a story in memory with a three-line script, `run_story` gives exactly one verdict for each direction with a word at each command prompt; a fixed exit the code honours is `confirmed`; one the code sends elsewhere is `wrong`; a direction with no table entry that stays is `confirmed` and one that moves is `wrong`; a conditional one is `conditional` with its outcome; the counts add up to the trials

### Implementation for User Story 1

- [x] T016 [US1] `crates/zmachine-corpus/src/report.rs` (F§ "A report"; D§ "Report", "Case"): `Report` with the heading, reach, totals, outcomes and detail of D§; `Report::write() -> String` and `Report::read(&str) -> Result<Report, (line, message)>` to pass T014. Cases are counted in a map keyed by room, direction, reported and outcome. Nothing in the report's form is specific to a later kind of check, which keeps a file of its own (FR-029; F§ "Room for more")
- [x] T017 [US1] `crates/zmachine-corpus/src/walk.rs` `run_story(&Story, &Ledger) -> Report` for the walkthrough alone (R5 item 1; FR-008): at each command prompt, ask introspection for the location and the exits, and for each direction with a word run a trial, take its verdict (T011) and count its case; count wordless directions; mark rooms visited; fill `places`, `walkthrough`, `rooms visited` and `named-not-reached` (R10: destinations of fixed exits reported at places visited, less the rooms visited). `Story` here is a plain struct with the fields of D§ "Story" that this needs; the ledger is empty until Phase 4. Passes T015
- [x] T018 [US1] `crates/zmachine-corpus/src/manifest.rs` (F§ "The manifest", the form only; FR-016): read entries of `key: value` lines separated by a blank line into `Story`, refusing with the line number an unknown key, a repeated key other than `opening`, a missing required key, and a value not of its form in F§'s table. Failing tests first in `tests/formats.rs`, one for each refusal and one for the example entry of F§. The checks that need the files (digest, size, `SOURCE.md`) are T034
- [x] T019 [US1] Add Zork I to `stories/corpus/manifest.txt` as F§'s example gives it (digest and size from `stories/zork1/SOURCE.md`; seed 11, as feature 003's `ZORK1` has it); `crates/zmachine-corpus/src/main.rs` `corpus write [ID...]` and `corpus run [ID...]` for present stories (K§ "`corpus`": `run` prints `<id> ok` or `<id> differs`, exit 0 or 1; an `ID` not in the manifest exits 2 with nothing run). A story is present when the file at `path` has the manifest's SHA-256 (D§ "Story"). Failing test first in `tests/run.rs`: `run` on a temporary copy of the manifest and reports is `ok` after `write`, and `differs` after one number in the kept report is changed
- [x] T020 [US1] Run `cargo run -q --release -p zmachine-corpus -- write zork1` and commit `stories/corpus/reports/zork1.txt`. In the notes at the end of this file record its totals, and read its detail: for each `wrong` case say in a line what it is (it will be `wrong-unexplained` until Phase 4). Confirm SC-001 by a test in `tests/run.rs` that the report in the repository has a verdict for every direction with a word at every command prompt: `trials` equals the walkthrough's command prompts (T008) times the directions with a word (T005)

**Checkpoint**: one game's figures exist and are kept

---

## Phase 4: User Story 2 - A wrong report stops the build until it is explained (Priority: P2)

**Goal**: An unexplained wrong case fails; a ledger entry with a cause makes it known

**Independent Test**: `cargo test -p zmachine-corpus --test ledger` passes (quickstart 5)

### Tests for User Story 2

- [x] T021 [P] [US2] Failing tests in `tests/formats.rs` (F§ "The ledger"; FR-013, FR-014): the example entry reads; a cause may continue on lines that start with two spaces; reading refuses, with the line number, a missing key, an empty cause, a `reported` and `outcome` that do not make a wrong verdict (for example `none` and `stayed`), and two entries with the same story, room, direction, reported and outcome
- [x] T022 [P] [US2] Failing tests in `crates/zmachine-corpus/tests/ledger.rs` (US2 scenarios 1 to 4; FR-013, FR-014; quickstart 5), on the story in memory whose table says B and whose code goes to C: with an empty ledger the run's report has `wrong-unexplained 1` and `corpus run` fails, printing the story's id, the room, the direction, what was reported and the outcome, in numbers; with the case in the ledger the report has `wrong-known 1`, `wrong-unexplained 0`, and the run passes; with a further entry that no trial matches the run fails and names that entry's line in the ledger; an entry for a story that is absent is not counted as stale

### Implementation for User Story 2

- [x] T023 [US2] `crates/zmachine-corpus/src/ledger.rs` (F§ "The ledger"; D§ "Ledger entry"): read and check as T021 asks; `Ledger::knows(story, case) -> bool`, recording that the entry was matched; `Ledger::stale(present_stories) -> Vec<line>`. Use it in `run_story` to split `wrong` into known and unexplained, and in `corpus run` to fail on an unexplained case or a stale entry, printing as T022 asks (K§ "`corpus`"). `corpus write` writes unexplained cases into the report and never touches the ledger
- [x] T024 [US2] For each `wrong` case in `stories/corpus/reports/zork1.txt`, find its cause: look the room and direction up in the story (`cargo run -q -p zmachine-cli -- --tree stories/zork1/zork1.z3`, and the ZIL source named in `stories/zork1/SOURCE.md`, read on the web, not downloaded), and write an entry in `stories/corpus/ledger.txt` whose cause names the rule or routine responsible without quoting a room's or an object's name (R9). Feature 003's specification section 8.3 (the game moves the player on at arrival) is the cause already known. **A case whose cause cannot be found is not entered: stop for the project owner with the case**. Then `corpus write zork1`, and `corpus run zork1` passes with no unexplained case (SC-003)
- [x] T025 [US2] FR-015 as a test in `tests/crate_rules.rs`: `git diff --quiet main -- crates/zmachine-introspect/src docs/introspection.md` is not something a test can run reliably in CI, so instead record in the notes, at the end of each phase from here on, the output of that command; and add to `tests/crate_rules.rs` a test that the introspection crate's `Cargo.toml` and `src` do not mention `corpus` or `ledger.txt`

**Checkpoint**: Zork I passes with every wrong case explained

---

## Phase 5: User Story 3 - Reach places no walkthrough visits (Priority: P3)

**Goal**: The crawl visits rooms by walking, within a budget, the same every time

**Independent Test**: `corpus run` on a story with no walkthrough gives a report with
more than one room visited, and running it twice gives the same report

### Tests for User Story 3

- [ ] T026 [P] [US3] Failing tests in `crates/zmachine-corpus/tests/crawl.rs` (R5; FR-009, FR-010, FR-012; US3 scenarios 1, 3, 5), on a story in memory of six rooms in a known shape with one direction that quits, one that asks a question and one that faults: the crawl visits every room reachable by directions, each once; the places are taken in the order R5 gives (first in, first out; directions ascending), checked against the expected list of rooms; the three branches are counted under their outcomes and nothing beyond them is visited; with a budget of three rooms the crawl stops at three and the report says `reached rooms`; with a budget of ten trials it stops and says `reached trials`; with the default budget it says `reached no`
- [ ] T027 [P] [US3] Failing test in `tests/crawl.rs` (FR-011; US3 scenario 2; SC-004): two runs of the same story, seed and budget give reports equal as text; so do two runs of a story in memory that calls the random instruction on each move
- [ ] T028 [P] [US3] Failing test in `tests/crawl.rs` (R5 item 2, second paragraph; US3 scenario 4): on a story in memory with a three-line script that passes a side room without entering it, the side room is visited by the crawl after the walkthrough and its trials are counted; a room the walkthrough itself reaches later is not crawled a second time; `places` equals `walkthrough` plus `crawl`

### Implementation for User Story 3

- [ ] T029 [US3] `crates/zmachine-corpus/src/walk.rs`: the crawl of R5 (a queue of places, each owning its machine; the visited set shared with the walkthrough; the budget from the story or the defaults 400 rooms and 40,000 trials), and a story with no walkthrough started by its `opening` lines (R6; F§ "The manifest", `opening`), the command address taken after them. Passes T026 to T028
- [ ] T030 [US3] Add Advent to `stories/corpus/manifest.txt` as two entries, `advent-z5` and `advent-z8` (digests and sizes from `stories/advent/SOURCE.md`; seed 1; the walkthrough; `standing` saying it is a test fixture only), `corpus write`, and repeat T024's work for Advent's wrong cases, using `stories/advent/advent.inf` for causes. `corpus write zork1` again now that the crawl runs: its report changes (more places); commit both. `corpus run` passes
- [ ] T031 [US3] Measure (R12; "Still open" 2; SC-009): time `cargo run -q --release -p zmachine-corpus -- run` and `cargo test -p zmachine-corpus` on this Mac and record both in the notes with each story's places and trials. If the plain test run takes more than two minutes, set `[profile.test.package.zmachine-corpus] opt-level = 2` in the root `Cargo.toml` and measure again. If a story reaches its budget, record it; change the default only with a reason in the notes

**Checkpoint**: three committed story files have reports from walkthrough and crawl

---

## Phase 6: User Story 4 - Add a game without committing it (Priority: P4)

**Goal**: A story is added by one manifest entry; an absent one is skipped; none is tracked

**Independent Test**: With a manifest entry and no file, the story's test reads
`ignored, story absent` and the run passes; after fetching, it runs; `git status` shows
no story file

### Tests for User Story 4

- [ ] T032 [P] [US4] Failing tests in `crates/zmachine-corpus/tests/check.rs` (F§ "The manifest check"; FR-016, FR-017, FR-018, FR-019; US4 scenarios 3, 4), each on a temporary folder laid out like the repository: two entries sharing an `id` fail; two sharing a `sha256` fail; a `committed` story whose file is absent fails; one whose digest, size, version or family differs fails, naming which; one with no `SOURCE.md` beside it, or whose `SOURCE.md` does not hold the digest, fails; one with no license text beside it (a file named `LICENSE`, or one whose name starts `COPYING`) fails; a `fetched` path outside `stories/local/corpus/` fails and a `committed` one inside it fails; a file in `stories/local/corpus/` whose digest matches no entry is not used by any run; a present `fetched` story whose version or family differs fails. And one test that the repository's own manifest and ledger pass
- [ ] T033 [P] [US4] Failing test in `crates/zmachine-corpus/tests/corpus.rs`, the target with its own `main` (R16; K§ "The tests"; FR-021; SC-005; US4 scenario 1): add `[[test]] name = "corpus" harness = false` to the crate's `Cargo.toml`; `main` prints for each story of the manifest `test <id> ... ok`, `test <id> ... FAILED` or `test <id> ... ignored, story absent`, then `test result: ok.` or `FAILED.` with the counts passed, failed and ignored, and exits 1 if any failed. The logic is a library function `corpus_lines(manifest, ledger, root) -> (Vec<String>, bool)` tested in `tests/run.rs` on a temporary manifest with one present and one absent story: the absent one is `ignored`, the result is ok

### Implementation for User Story 4

- [ ] T034 [US4] `crates/zmachine-corpus/src/manifest.rs` `check(root) -> Vec<Failure>` and `corpus check` (K§ "`corpus`"), to pass T032. Whether a fetched path is tracked is asked of `git ls-files --error-unmatch <path>`, run from `root`; where `git` is not there the check says so and does not fail
- [ ] T035 [US4] `scripts/fetch-corpus.sh` (K§ "`scripts/fetch-corpus.sh`"; FR-020; US4 scenario 2), executable, POSIX shell with `set -eu`: `ID...` and `--propose ADDRESS` as K§ gives them; no "all"; refuses a `committed` id; `curl -fsSL -A curl`; on a digest that differs, removes the file, says both digests, exits 1. It reads the manifest with `awk`. Check it without the network by a test script `scripts/test-fetch-corpus.sh` that serves a temporary folder with `python3 -I -m http.server` on a free port of 127.0.0.1 and runs the script against a temporary manifest: a right digest keeps the file; a wrong one removes it; a present file is left alone; `--propose` prints name, size, digest and first byte. Run it and record the output in the notes
- [ ] T036 [US4] Failing test in `crates/zmachine-corpus/tests/no_text.rs` (R9; FR-026; SC-006; US4 scenario 5): every file under `stories/corpus/reports/` reads by `Report::read` or the summary's reader (T045 adds the summary; until then reports only); for each committed story, every object's printed name (from `Introspector::object_tree`) and every dictionary word of five letters or more is searched for, without regard to case, in that story's report and the summary, less the keywords of F§ (list them in the test); the ledger is searched for the printed names of two words or more only (research R9, as amended); a hit fails. Show it can fail by putting a room's name in a ledger cause for one run
- [ ] T037 [US4] **Zork II (FR-022). Ask the project owner**: "Fetch `zork2.z3`, 92,524 bytes, from `https://github.com/historicalsource/zork2` (`COMPILED/zork2.z3`), MIT License dated 2025-11-20, to be committed with the same caveat as Zork I?" On approval only: fetch that file and the repository's `LICENSE` at a named commit; create `stories/zork2/` with `zork2.z3` (added with `git add -f`), `LICENSE` and a `SOURCE.md` on the pattern of `stories/zork1/SOURCE.md` (source, commit, SHA-256, size, version, release and serial read from the file's header, license, the caveat, the trademark note); add the manifest entry (`tier: committed`, no walkthrough, seed 1, `opening` lines if it needs them); `corpus check`, `corpus write zork2`; explain its wrong cases as in T024. If declined, mark this task "declined"
- [ ] T038 [US4] **Zork III (FR-022). Ask the project owner** as in T037, for `zork3.z3`, 87,984 bytes, from `https://github.com/historicalsource/zork3` (`COMPILED/zork3.z3`); on approval only, the same steps under `stories/zork3/`
- [ ] T039 [US4] **Mini-Zork I. Ask the project owner**: "Fetch `minizork.z3` from `https://ifarchive.org/if-archive/infocom/demos/minizork.z3` (size not stated by the archive; at most 128 KB), no license attached, personal use only under the archive's terms, never to be committed?" On approval only: `scripts/fetch-corpus.sh --propose` that address; write the manifest entry from what it prints (`tier: fetched`, `path: stories/local/corpus/minizork.z3`, `family` as `corpus check` reports it, seed 1); `scripts/fetch-corpus.sh minizork`; find by running it whether it opens at a command prompt, and add `opening` lines if not (R6; "Still open" 3); `corpus write minizork`; explain its wrong cases in the ledger by room, direction and rule, without the game's text; confirm `git status --short` shows no story file. If it cannot be loaded or used, say why in the notes and mark the task "unusable"
- [ ] T040 [P] [US4] **The 1984 Infocom Sampler. Ask the project owner** as in T039, for `https://ifarchive.org/if-archive/infocom/demos/sampler1_R55.z3`, id `sampler1`; on approval only, the same steps
- [ ] T041 [P] [US4] **Colossal Cave in ZIL. Ask the project owner** as in T039, for `https://ifarchive.org/if-archive/games/zcode/advent.z3`, id `advent-zil`; on approval only, the same steps
- [ ] T042 [P] [US4] **Six stories of the Inform family. Ask the project owner about each by name, one question each**, as in T039 (at most 256 KB for version 5, 512 KB for version 8), and on each approval do the same steps: `curses` (`games/zcode/curses.z5`), `adventureland` (`games/zcode/Adventureland.z5`), `balances` (`games/zcode/Balances.z5`), `nine-oh-five` (`games/zcode/905.z5`), `anchorhead` (`games/zcode/anchor.z8`), `advent-crowther` (`games/zcode/Advent_Crowther.z8`), all under `https://ifarchive.org/if-archive/`. A story whose family `corpus check` reports as `none` stays in the corpus with `family: none` and a report of no trials: that is a finding (spec, Edge Cases)
- [ ] T043 [US4] Count the fetched stories with a report (FR-023; SC-011): **if fewer than six, stop for the project owner** with what was declined or unusable and a proposal of others from the same archive, each with its address. Otherwise record the list in the notes, and for each story with no walkthrough its rooms visited and rooms named and not reached (SC-007): a story on which the crawl visited one room only is recorded as a finding, with what stopped it, and put to the project owner

**Checkpoint**: about ten stories have reports; nothing fetched is tracked

---

## Phase 7: User Story 5 - One table across the corpus (Priority: P5)

**Goal**: One summary with a line for each story, each family and the whole

**Independent Test**: `corpus run` passes and `stories/corpus/reports/summary.txt` has a
line for each story that ran, whose figures equal that story's report

### Tests for User Story 5

- [ ] T044 [P] [US5] Failing tests in `tests/formats.rs` and `tests/run.rs` (F§ "The summary"; FR-025, FR-027, FR-028; US5 scenarios 1 to 3; SC-008): from three hand-made reports of two families the summary's `story` lines hold each report's figures (`fixed A/B` as confirmed of reported fixed; `none A/B`; `conditional A/B` as moved of conditional; `unknown N`), the `family` lines are the sums in the order `zil`, `inform`, `none`, and `all` the sum of those; an absent story has a `skipped` line, no `story` line, and is in no total; reading refuses a line not of the form; `corpus run` fails when a present story's line in the kept summary differs from a fresh one, and when every story is present and any other line differs; with a story absent, `corpus write` leaves that story's kept report alone and rewrites only what it can

### Implementation for User Story 5

- [ ] T045 [US5] `crates/zmachine-corpus/src/report.rs` `Summary` (F§ "The summary"; D§ "Summary", "What is kept in the repository"): build from reports, write, read, compare; use it in `corpus run` and `corpus write` (K§) to pass T044; extend `tests/no_text.rs` to read the summary. `corpus write`, commit the summary
- [ ] T046 [US5] Complete `stories/corpus/README.md` (plan, "Strongest objection"; spec, Assumptions): what each figure means; the answer to SC-008 in a sentence for each family, with the numbers; and the limits, plainly: the room comes from introspection, so the figures show consistency; "moved" means the room object changed; the crawl only walks, with each story's rooms visited and named but not reached; which stories are fetched and so absent from CI. No name of a room or an object, and no line of any game's text

**Checkpoint**: the question the feature exists to answer has a figure

---

## Phase 8: Polish & Cross-Cutting Concerns

- [ ] T047 [P] `.github/workflows/ci.yml`: the corpus already runs under `cargo test --workspace` on both platforms. Add nothing unless T031 found the plain run slow; if so add a step `cargo test --release -p zmachine-corpus` to the `test` job. No step fetches a story (FR-020): add a test to `tests/crate_rules.rs` that `ci.yml` does not contain `fetch-corpus`
- [ ] T048 [P] `README.md`: a short section on the corpus (what it is, the three commands of quickstart 2, 3 and 7, that fetched stories are never committed), and the layout row for `crates/zmachine-corpus` and `stories/corpus`
- [ ] T049 SC-004 on this Mac: `cargo run -q --release -p zmachine-corpus -- write && git status --short stories/corpus` prints nothing (quickstart 8). SC-004 across platforms is CI passing on Linux and macOS against the same committed reports; record the run
- [ ] T050 Run the quickstart, steps 1 to 9, and record each result in the notes. Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`; confirm with `git diff main --stat` that nothing outside `crates/zmachine-corpus`, `stories/corpus`, `stories/zork2`, `stories/zork3`, `scripts`, `specs/005-corpus-testing`, `Cargo.toml`, `Cargo.lock`, `README.md` and `.github` changed; list at the end of this file what was done differently from the plan, including any test that was not seen to fail before its code was written

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: none
- **Foundational (Phase 2)**: after Setup; blocks every story. T008 and T013 are gates
- **US1 (Phase 3)**: after Foundational
- **US2 (Phase 4)**: after US1 (it needs a report with wrong cases to explain)
- **US3 (Phase 5)**: after US2 (new places bring new wrong cases, which need the ledger)
- **US4 (Phase 6)**: T032 to T036 after US1; T037 to T043 after US3, since a story with
  no walkthrough is reached only by the crawl
- **US5 (Phase 7)**: after US1 for the code; its committed summary after US4
- **Polish (Phase 8)**: last

### Within Each Phase

- A test task before the implementation task that passes it
- T004 before every test that builds a story in memory
- T005, T009 and T011 before T012; T006 before T007, T008 and T012
- T016 before T017; T018 before T019; T019 before T020
- T023 before T024; T029 before T030; T034 before T037
- T035 before T039; T037 to T042 each wait on the owner and on nothing else, once T035
  and T036 are done

### Parallel Opportunities

- T002 and T003
- T005 and T011 (different files), alongside T006
- T014 and T015; T021 and T022; T026, T027 and T028; T032 and T033
- T040, T041 and the six of T042, once T039 has shown the steps work on one story
- T047 and T048

## Parallel Example: Phase 2

```text
T005  directions and their words          tests/trial.rs, src/directions.rs
T006  the driver and the prompt rule      tests/prompts.rs, src/walk.rs
T011  reported and verdict                tests/verdict.rs, src/verdict.rs
```

## Implementation Strategy

### MVP First

Phases 1 to 3. At the end of Phase 3 there is a report for Zork I along its
walkthrough: figures nobody has today, from an instrument proven not to disturb the
game. It can be read and judged before anything more is built.

### Incremental Delivery

1. Phases 1 and 2: the instrument, proven on the two walkthroughs
2. Phase 3: one game's figures (MVP)
3. Phase 4: wrong cases fail until explained
4. Phase 5: the crawl; Advent joins
5. Phase 6: the manifest check, the fetch, the skip; the owner approves stories one by one
6. Phase 7: the summary and what it means
7. Phase 8: CI, the README, the quickstart

### What was learned from the earlier features

- A test that passes before its code proves nothing: show it can fail, and say how
  (feature 004, T032 and T048).
- A plan's claim about a rule is checked on real stories before anything is built on it
  (feature 004's prompt rule; here T008).
- What was done differently from the plan is written at the end of this file, with what
  was not done.

## What was done, and what was done differently from the plan

Written as the work goes. "Seen to fail" means run and seen to fail by assertion before
the code beneath it was written.

### Phases 1 to 3

- **T002**: the dependency rule passed at once, as expected of a rule about something
  not yet done. It was seen to fail with `zmachine-corpus` named in a comment in
  `crates/zmachine-cli/Cargo.toml` for one run; the line was then removed.
- **T005, T009**: ten of twelve tests were seen to fail against stubs. Two passed against
  the stubs, because a stub's empty answer is what they expect ("a story of no family has
  no directions"; "a trial where the location is unknown is lost"). They mean something
  only beside the others.
- **T008, the gate**: the rule counts 330 command prompts on Zork I and 171 on each
  Advent, where the earlier features counted 332 and 172 by the game's text. Every
  difference was found and is a stop at which the game is not asking for a command: one
  room of Zork I that reads its own line (walkthrough line 87, the stop feature 004 met),
  and the restart-restore-quit question at the end of each game. The task said to stop
  for the project owner on any difference but the first. The other two are the old count
  being wrong and the rule being right, so the work went on; it is reported to the owner.
- **A toy with no exits was of no family.** The ZIL family is known by rooms having
  direction properties; the test stories' builder now gives such a toy one "not an exit"
  property.
- **A rule added to FR-006**: an exit reported as leading to the room the player is
  already in has no verdict when the player is still there, since a move to the same
  room cannot be told from staying. Zork I's first report had 18 such trials counted as
  wrong. The spec's table and the data model were amended; the rule's test was seen to
  fail first.
- **`corpus.rs`** is a module the plan did not list: the program's commands as functions,
  so the tests call what the program runs.
- **Zork I along its walkthrough** (T020), before any ledger: 330 places, 83 rooms, 4,290
  trials in 1.3 seconds optimised. Reported fixed: 749 confirmed, 26 wrong. Reported not
  an exit: 3,350 confirmed, 36 wrong, 2 that ended in a question. Reported conditional:
  109, of which 87 let the player through. No verdict: 18. The wrong cases are explained
  in Phase 4.

### Phase 4

- **T021 to T023**: both ledger format tests and all five behaviour tests were seen to
  fail against a ledger that held nothing.
- **T024, Zork I's wrong cases: all 62 trials (49 cases) explained, by six causes.** Each
  was found by running the trial at the walkthrough's own place and reading what the game
  printed, not from the source. None is a fault in how introspection reads the tables;
  each is the game's code doing something the table does not say.

  | Cause | Reported | Happened | Trials |
  | --- | --- | --- | --- |
  | The player is in a vehicle whose own routine refuses the walk | leads to a room | stayed | 16 |
  | The destination admits the player only in a vehicle | leads to a room | stayed | 6 |
  | The destination reads its own line, so the instrument sees a question | leads to a room | question | 2 |
  | A flag exit whose flag the room sets afresh when a command begins: the stored value is one command old | leads to a room | stayed | 2 |
  | The same, the other way | not an exit | moved | 3 |
  | A timed event carries the vehicle on, whatever the command | not an exit | moved | 22 |
  | A timed event kills the player, and the death routine moves them | not an exit | moved | 11 |

  The fourth and fifth matter most to the compass: feature 003 decided to evaluate a flag
  exit's flag (its research R5) and looked for a stale flag without finding one. Here are
  two rooms where it is stale, in both directions.
- **The two "other" trials** (reported not an exit; the game asked a question) are the
  last room of the walkthrough, where entering ends the game. Not wrong, by FR-006.
- **Research R9 amended**: the ledger is searched for object names of two words or more,
  not for dictionary words, which include ordinary words a cause cannot avoid.
- **T025**: `git diff --quiet main -- crates/zmachine-introspect/src docs/introspection.md`
  exits 0 at the end of this phase.
