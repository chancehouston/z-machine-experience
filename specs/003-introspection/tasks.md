# Tasks: Introspection Interface

**Input**: Design documents from `/specs/003-introspection/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Required. Constitution Principle II makes test-first non-negotiable, and the
last feature broke it for its fidelity tests. Here every test task is written **and run
and seen to fail by assertion** (not by a compile error alone) before the implementation
task beneath it is started; the implementer records the failing output in the commit
message of the test task. Every test of a query carries a comment citing the section of
`docs/introspection.md` it verifies (written "I§"); tests of the Z-machine's own tables
cite the Standard ("§").

**The guard**: after every task that touches `crates/zmachine-core` or
`crates/zmachine-cli`, `cargo test --workspace` must pass with no test from features 001
or 002 edited. A needed edit to an earlier test stops the task for the project owner.

**No per-game knowledge**: no source file under `crates/zmachine-introspect/src` may name
a game, a serial number, a release or a checksum. T009 adds the test that enforces it.

**Stop points**: stop for the project owner if a research rule turns out false on the
committed files; if Zork I's inventory is unknown at more than 3% of prompts or any
answer is wrong; or if a hand-made expectation and the game disagree and the source does
not settle which is right.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an unfinished task)
- **[Story]**: The user story the task belongs to (US1 to US6)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: The new crate, and the specification's skeleton that every test will cite

- [x] T001 Create crate `crates/zmachine-introspect` (edition 2024, `#![forbid(unsafe_code)]`, depends only on `zmachine-core`; dev-dependencies `zmachine-testkit` and `sha2`), add it to the workspace `Cargo.toml`, with an empty `src/lib.rs` that builds
- [x] T002 Write `docs/introspection.md` as a skeleton at version 0.1.0: every section numbered and titled, one sentence each, so tests can cite them from now on. Sections: 1 Purpose and rules for every query (contracts/queries.md "Rules for every query", numbered 1.1 to 1.7); 2 Answers, unknown and its reasons, exactness, the JSON form (research R13); 3 Families and recognition (3.1 Inform 6, 3.2 ZIL, 3.3 unrecognised); 4 `object_tree`; 5 `dictionary`; 6 `location`; 7 `inventory`; 8 `exits`; 9 `scope`; 10 Known limits; 11 Versioning. Each query section is headed exactly `` ## N. `name` `` so T046 can read the names
- [x] T003 [P] Add `crates/zmachine-introspect/README.md` stating what the crate is, that the contract is `docs/introspection.md`, and the no-per-game rule

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: What every query needs: a read-only view of the machine, a copyable machine
for the tests, table readers, the answer types, and the walkthrough driver

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### The core's two additions

- [x] T004 Failing tests in `crates/zmachine-core/tests/view.rs` (contracts/rust-api.md "Addition to `zmachine-core`"): `view().version()` and `len()` for a version 3, 5 and 8 image; `byte` and `word` return current memory, so a value the game stored is seen; `shipped_byte` and `shipped_word` return the file as loaded even after the game has written there; an address at and beyond the end gives `None` for all four; a view can be held across nothing that mutates (compile-time: the test takes two views at once)
- [x] T005 Implement `crates/zmachine-core/src/inspect.rs` with `View` and `Machine::view` in `lib.rs` until T004 passes; run the guard
- [x] T006 Failing tests in `crates/zmachine-core/tests/clone.rs`: a clone of a machine waiting for a line, given a different line from the original, diverges while the original's output, transcript, memory (through `view`) and next event are what they are without the clone; a clone made mid-walkthrough of Zork I and of Advent, given the same remaining input, produces the same output and the same save bytes as the original; a clone of a machine waiting for a key, for a save result and after a fault answers as the original does; undo history and the random generator are copied (same `random` results, same undo depth)
- [x] T007 Implement `Clone for Machine` in `crates/zmachine-core/src/lib.rs` (derive where every field allows it) until T006 passes; run the guard
- [x] T008 Failing test in `crates/zmachine-core/tests/coverage.rs`: no file under the core's `src` contains `Cell<`, `RefCell`, `UnsafeCell`, `static mut`, `Mutex`, `RwLock` or `Atomic` (rust-api.md guarantee 1). It passes at once if the core is as believed; confirm it can fail by temporarily adding a `Cell` and seeing it caught, and say so in the commit message

### The new crate's ground floor

- [x] T009 [P] Failing tests in `crates/zmachine-introspect/tests/crate_rules.rs`: the crate's `Cargo.toml` lists no dependency but `zmachine-core`; no source file contains `unsafe`, `std::fs`, `std::io`, `println`, or any of the words `zork`, `advent`, `adventure`, `czech`, `praxix` (case-insensitive), nor a six-digit serial number literal (rust-api.md guarantees 4 and 6)
- [x] T010 [P] Failing unit tests in `crates/zmachine-introspect/src/answer_tests.rs` (I§2): an `Answer` is a value or an `Unknown` with one of the six reasons of data-model.md, and an `Exactness`; `Part<T>` and `Tri` as in rust-api.md; each reason's text form is the hyphenated word the data model gives
- [x] T011 Implement `crates/zmachine-introspect/src/answer.rs` until T010 passes
- [x] T012 Failing unit tests in `crates/zmachine-introspect/src/table_tests.rs`, on images built by the testkit for versions 3, 5 and 8 (§12, §13, §3): the last object is the one before the lowest property table; an object's parent, sibling, child, attributes and stored name; its properties with numbers and data, in table order, for both property formats; text decoding with abbreviations and the custom alphabet and Unicode tables where the header names them; the dictionary's entry length, word count, each word's text and data bytes; globals by index, current and as shipped; every reader returns `None`, never panics, on an address outside the file, an object number of 0 or past the last, a property table that runs off the end, and a text with no end bit
- [x] T013 Implement `crates/zmachine-introspect/src/table.rs`, reading only through `View`, until T012 passes
- [x] T014 [P] Extend `crates/zmachine-testkit` so adapter unit tests can build small story files of each family: for version 3, dictionary entries with chosen data bytes, a `ROOMS` object with rooms carrying direction properties of each of the five lengths, and unplaced objects; for version 5, the header's Inform version bytes, the four metaclass objects, `compass` with children carrying property 21, `(self object)`, `(darkness object)`, `(Inform Library)`, chosen attributes and properties on any object, and routines and strings at chosen places in high memory. Tests of the builders in `crates/zmachine-testkit/src/tests.rs`

### The walkthrough driver and the hand-made expectations

- [x] T015 Write `crates/zmachine-introspect/tests/common/mod.rs`: a driver that replays a script through `Machine` with a seed, and at every stop records the event kind, whether it is a command prompt (a line is wanted and the text ends with `>`), the lower-window text since the last stop, the canvas's first row, and the command then given; a helper that clones the machine at a command prompt, gives the clone one command, and returns its reply; a hook called at every stop with `&Machine`. With tests of the driver itself in `tests/driver.rs`: both walkthroughs replay to the committed reference transcripts, the count of command prompts is at least 320 for Zork I and at least 165 for each Advent build and is recorded in the test, and the clone helper's `inventory` reply at the first prompt of each game is the game's empty reply
- [x] T016 [P] Write `stories/zork1/introspection/exits.txt` and its `README.md`: at least 25 rooms' exits worked out by hand from the game's ZIL source at the revision `stories/zork1/SOURCE.md` names (read from <https://github.com/historicalsource/zork1>; not committed), each entry giving the walkthrough step at which it holds, the room's name, and for each direction `fixed <destination name>`, `conditional`, or its absence; chosen to include every one of the five exit forms, a flag exit before and after its flag is set, a room with no exits listed if there is one, and an exit that leads to its own room if there is one. The README records the source revision, the file format, and that the list was made without reference to this interpreter's output
- [x] T017 [P] Write `stories/advent/introspection/exits.txt` and its `README.md` the same way from `stories/advent/advent.inf`: at least 25 rooms, including exits to rooms, to the two grates (doors), by routine, refused by a string, `in` and `out`, a maze room whose printed name is not its stored name, and a room reached in the dark
- [x] T018 [P] Write `stories/zork1/introspection/scope.txt` and `stories/advent/introspection/scope.txt`: at least 20 positions in each game, each naming a walkthrough step, the objects the game's room description names there, and at least one object in another room that must not be listed; for Advent, positions that include a closed opaque container with something in it, an open or transparent one, a supporter, an object present by `found_in`, and a dark room; for Zork I, positions that include a container holding something and the player carrying a container

**Checkpoint**: The view, the clone, the table readers and the driver work; the
expectations exist and were written before any adapter

---

## Phase 3: User Story 4 - Read the object tree and the dictionary (Priority: P4, built first)

**Goal**: The two queries that need no family, for every story file

**Why first**: the plan's order of work puts these ahead of recognition, because every
other query is built on them. The spec ranks the story fourth by value to a host.

**Independent Test**: `cargo test -p zmachine-introspect --test machine_level`

- [x] T019 [US4] Failing tests in `crates/zmachine-introspect/tests/machine_level.rs` (I§4, I§5; SC-007): for each of the seven committed story files, the object tree through `Introspector::object_tree` has every object once, and its names, parents and children in order equal those the core's own decoder gives when the story is run far enough to print each name (or, where that is impractical, those computed by an independent reader written in the test from the Standard alone); the dictionary has the word count the dictionary header states, in order, each word equal to what the core's lexer finds when given that word; both answers are `exact`; both are answered for czech and praxix; SHA-256 digests of the story files match their `SOURCE.md` records
- [x] T020 [P] [US4] Failing unit tests in `crates/zmachine-introspect/src/lib_tests.rs` on built images (I§4, I§1.6): a loop of parents, a child whose parent disagrees, a sibling chain longer than the object count, and an object number past the end each give `damaged` for the objects concerned and full answers for the rest; an object with an empty name; a dictionary whose entries are unsorted is still listed in table order; an entry length shorter than the text gives `damaged`
- [x] T021 [US4] Implement `Introspector::new`, `spec_version`, `object_tree` and `dictionary` (word list only; `parts` unknown) in `crates/zmachine-introspect/src/lib.rs`, with `QUERIES` and `SPEC_VERSION`, until T019 and T020 pass
- [x] T022 [US4] Fill in sections 4 and 5 of `docs/introspection.md`: what each returns, field by field, when a part is `damaged`, that both are exact and work for every story file, that only the header's dictionary is read

**Checkpoint**: Any story file's tree and words can be read

---

## Phase 4: User Story 1 - See where the player is and what they carry (Priority: P1) 🎯 MVP

**Goal**: Recognise the family; report location and inventory exactly

**Independent Test**: `cargo test -p zmachine-introspect --test families --test zork1 --test advent location inventory`

### Recognition

- [x] T023 [US1] Failing tests in `crates/zmachine-introspect/tests/families.rs` (I§3; SC-008): Zork I is `zil`; both Advent builds are `inform6`; czech in versions 3, 5 and 8 and praxix are `unrecognised`, and for them `location`, `inventory`, `exits` and `scope` each answer unknown with `unrecognised-family` while `object_tree` and `dictionary` answer in full; recognition reads the file as shipped, so the family is the same at the first stop and at the last stop of each walkthrough
- [x] T024 [P] [US1] Failing unit tests in `crates/zmachine-introspect/src/family_tests.rs` on built images (I§3.1, I§3.2): one image that satisfies every Inform rule of research R1 is recognised, and six more, each breaking exactly one of rules 1 to 6 (wrong header bytes; a metaclass renamed; `(darkness object)` missing; two objects named `compass`; a compass child without property 21, and two children sharing a value; `(self object)` with an extra attribute; no global, and two globals, holding `(Inform Library)`), are each unrecognised; likewise one image satisfying every ZIL rule of research R2, with `ROOMS` itself carrying a direction property as Zork I's does, and five more each breaking one rule (version 5; 9-byte entries; three direction words; a direction property on an object under a different parent; a 6-byte direction property, and a 1-byte value that is not a room); an image satisfying both sets of rules cannot be built (rule 1 of each excludes the other) and the test says so
- [x] T025 [US1] Implement `crates/zmachine-introspect/src/family.rs` and `Introspector::family` until T023's recognition cases and T024 pass; the family-dependent queries exist and answer `unrecognised-family` for every file for now
- [x] T026 [US1] Fill in section 3 of `docs/introspection.md` with the rules of research R1 and R2 exactly as implemented, their limits, and that a file satisfying neither set in full is unrecognised

### Location

- [x] T027 [US1] Failing tests in `crates/zmachine-introspect/tests/zork1.rs` and `tests/advent.rs` (I§6; SC-002, FR-037), using the driver: at every command prompt the location is known and `exact`; a reply whose first non-empty line is a room's printed name names the reported room; whenever the reported room changes the reply has such a heading or says it is dark (Zork I: contains "pitch black"; Advent: `lit` is `no`); for Advent the canvas's first row begins with the reported printed name, or with "Darkness" when `lit` is `no`; at the opening the room is "West of House" and "At End Of Road"; the Advent maze rooms report the printed name "Maze"; `lit` is `unknown` throughout Zork I, and is `no` at least once and `yes` at least 150 times in Advent; `holder` is `none` wherever the game has not put the player in something, and is the boat at the Zork I steps where the player is in it
- [x] T028 [P] [US1] Failing unit tests in `crates/zmachine-introspect/src/inform_tests.rs` and `src/zil_tests.rs` on built images (I§6; research R3, R4): Inform: the room is the top of the player's parent chain; a player inside a cage inside a room gives the room and the cage as `holder`; a player with no parent gives `no-player`; the `location` variable holding some third object gives `inconsistent`; holding the darkness object gives `lit` `no`; no global holding the player's number gives `inconsistent`; `printed_name` is the `short_name` string when there is one, the stored name when there is none, and unknown when `short_name` is a routine. ZIL: the room is global 0; global 0 holding a non-room, 0, or a number past the last object gives `no-player`; `lit` is `unknown`
- [x] T029 [US1] Implement `location` for both families in `crates/zmachine-introspect/src/inform.rs` and `src/zil.rs` until T027 and T028 pass
- [x] T030 [US1] Fill in section 6 of `docs/introspection.md`, and in section 10 the Inform change-of-player limit as research R3 states it

### Inventory

- [x] T031 [US1] Failing tests in `tests/zork1.rs` and `tests/advent.rs` (I§7; SC-005): at every command prompt, the clone's reply to `inventory` is compared with the reported inventory by the two rules of plan.md "How the evidence is gathered" (every reported printed name occurs in the reply; the reply's count of item lines equals the count of reported objects including contents the game shows), and an empty report meets the game's empty reply; the answer is `exact`; in Advent the inventory is never unknown; in Zork I it is unknown at no more than 3% of command prompts, only ever with `ambiguous-player`, and the test prints the steps where it is; after the walkthrough's `take` commands the taken object is in the next report; a carried container's contents are nested under it, not listed beside it
- [x] T032 [P] [US1] Failing unit tests in `src/inform_tests.rs` and `src/zil_tests.rs` (I§7; research R4, R8): Inform: children of `(self object)` in tree order with nested contents; `worn` is `yes` for attribute 26 and `no` otherwise; empty when it has no children. ZIL: with one unplaced-as-shipped object in the room whose number a global holds, that object is the player; with two such, `ambiguous-player`; with none, `no-player`; an unplaced-as-shipped object carried by the player does not tie with it; one inside a container in the room is considered; a global 0 that happens to hold a candidate's number does not count; `worn` is `unknown`
- [x] T033 [US1] Implement `inventory`, and the ZIL player rule with `holder`, in `src/inform.rs` and `src/zil.rs` until T031 and T032 pass. If Zork I's unknowns exceed 3% or any comparison fails, stop for the project owner
- [x] T034 [US1] Fill in section 7 of `docs/introspection.md`, including the ZIL player rule, that a tie is never broken, and the measured rate of unknowns in Zork I

**Checkpoint**: Both games report where the player is and what they hold, checked against
the games' own text at every prompt

---

## Phase 5: User Story 2 - See the exits (Priority: P2)

**Goal**: The directions out of the room, with destinations where they are stored facts

**Independent Test**: `cargo test -p zmachine-introspect --test zork1 --test advent exits`

- [x] T035 [US2] Failing tests in `tests/zork1.rs` and `tests/advent.rs` (I§8; SC-003, SC-004): a parser for `exits.txt` and, at each step it names, the reported exits equal the expected ones direction for direction, with kinds and destination names, and no direction in both lists; at every command prompt, when the command cut to the dictionary's word length is a word of a reported fixed exit and the room then changed, the new room is the exit's destination, with at least 100 such moves in each game; every reported exit has at least one word except where the specification allows none; `in` and `out` in Advent carry the words `in` and `out`; the exits answer is `exact`, and unknown only when the location is
- [x] T036 [P] [US2] Failing unit tests in `src/zil_tests.rs` (I§8; research R5): each of the five lengths gives what the table of R5 says; a flag exit is a fixed exit when its global is non-zero and absent when zero, the global being the byte minus 16; a flag byte below 16 gives a conditional exit; a 1-byte value that is not a room, and a length of 0 or above 5 (in a state reached after recognition by damaging memory), give that direction as unknown; the words are every dictionary word with that direction property, shortest first
- [x] T037 [P] [US2] Failing unit tests in `src/inform_tests.rs` (I§8; research R6, R7): each row of R6's table in its order, including a parentless door (conditional, not fixed), an object that is neither room nor door (conditional), and a property of one or four bytes (direction unknown); the routine and string bounds: with properties 1, 2 and 3 holding values that would mislead, S and R are found as R7 defines; a value between R and S is a direction unknown; S not above R makes every non-object value unknown; words come from the compass child's `name`, and for direction properties 17 and 18 without `name` are `in` and `out` only when the dictionary marks those words as verbs
- [x] T038 [US2] Implement `exits` for both families, with the direction table and the routine and string bounds worked out once in `Introspector::new`, until T035 to T037 pass. If an expectation in `exits.txt` and the game disagree, read the source again before changing either; record any correction to the expectation file in its README
- [x] T039 [US2] Fill in section 8 of `docs/introspection.md`: the exit forms per family, flag exits, the routine and string rule and its limits, words, and that a game's code may still refuse or redirect a listed exit

**Checkpoint**: The data a tappable compass needs

---

## Phase 6: User Story 3 - See what is within reach (Priority: P3)

**Goal**: The approximate list of objects the player could act on

**Independent Test**: `cargo test -p zmachine-introspect --test zork1 --test advent scope`

- [x] T040 [US3] Failing tests in `tests/zork1.rs` and `tests/advent.rs` (I§9; SC-006): a parser for `scope.txt` and, at each step it names, every object the file lists is in the reported scope and none of its must-not-list objects is; at every command prompt the answer is `approximate`, no object appears twice, the player and the room are not entries, every entry with `reach` `inside` or `on` names a `within` that is itself an entry, and every carried entry is in the reported inventory and the reverse; in Advent no entry is inside a closed opaque container, `light` is `lit` or `dark` and never `unknown`, and in the dark only carried and worn entries are listed; in Zork I `light` is `unknown`, every entry inside another has `visibility` `unknown`, `scenery` and `concealed` are `unknown`, and at the steps where the inventory is unknown the answer's `player` says `ambiguous-player`, nothing is marked carried, and the tied objects are absent
- [x] T041 [P] [US3] Failing unit tests in `src/inform_tests.rs` and `src/zil_tests.rs` (I§9; research R9): Inform: a closed opaque container hides its contents; open, or transparent, shows them as `inside`; a supporter's as `on`; the same rules for what is carried; `scenery` and `concealed` from attributes 17 and 3; `worn` reach from 26; an object whose `found_in` is a plain list including the room is listed `in-room`, one whose list does not include it is not, and one whose `found_in` is a routine is not; dark lists only what is carried. ZIL: everything under the room is listed, nested entries marked `visibility` `unknown`. Both: a tree with a loop below the room gives `damaged` and does not hang; 250 objects in one room are all listed
- [x] T042 [US3] Implement `scope` for both families until T040 and T041 pass
- [x] T043 [US3] Fill in section 9 of `docs/introspection.md`: what is listed and what is not, per family; each field; every reason the answer is approximate; that the interface marks and does not filter

**Checkpoint**: All six queries answer for both games

---

## Phase 7: User Story 4, completed - Parts of speech (Priority: P4)

- [x] T044 [US4] Failing tests in `tests/machine_level.rs`, `src/inform_tests.rs` and `src/zil_tests.rs` (I§5; research R10): in Zork I `north` is a direction with its direction number, `take` a verb, `lamp` a noun, `with` a preposition, and a word with several bits carries each part; in Advent `take` is a verb, `lamp` a noun, `n` and `north` directions with the direction number the exits query uses, and a word with bit 0x08 is a `grammar-word`; for czech and praxix `parts` is unknown; unit tests of each bit in turn for each family
- [x] T045 [US4] Implement `parts` and `direction` in `dictionary` for both families until T044 passes; fill in the rest of section 5 of `docs/introspection.md`

---

## Phase 8: User Story 5 - Inspection changes nothing (Priority: P5)

**Goal**: Proof that asking alters nothing, and that no file can break the queries

**Independent Test**: `cargo test -p zmachine-introspect --test read_only --test robustness`

- [x] T046 [P] [US5] Tests in `crates/zmachine-introspect/tests/read_only.rs` (I§1.1; SC-001, FR-036), written to fail if any query were to write: each walkthrough (Zork I, Advent in both builds), replayed with all six queries asked at every stop of every kind, gives the committed reference transcript byte for byte; each committed save script (`saves/make_n.txt`: five for Zork I, three for each Advent build), run with and without the queries, writes equal save files; at every stop the whole of memory read through `view` is equal before and after the six queries, and so are the next event and the next 50 results of a cloned machine's `random`. Confirm the memory comparison can fail by a deliberate mutation in a scratch copy of the test, and say so in the commit message
- [x] T047 [P] [US5] Tests in `crates/zmachine-introspect/tests/robustness.rs` (I§1.6; SC-009): with the damage patterns of `crates/zmachine-core/tests/robustness.rs` (truncation, each header byte, random bytes, the object table, the dictionary, the globals, noise with a valid version byte) applied to Zork I, both Advent builds, czech and praxix, at least 3,000 damaged files in all, every one the core loads is given a new introspector and all six queries at load and at up to 20 stops; none panics and each finishes within a bound of work proportional to the file's size; a file whose family is recognised and whose tables are then damaged in memory by the game's own writes (simulated by damaging the shipped file after the rules' data but before play) answers or says unknown; an introspector given a view of a different story file answers `inconsistent` for every query (rust-api.md guarantee 5)
- [x] T048 [US5] Fix whatever T046 and T047 show, each defect with a failing unit test in the matching `_tests.rs` file first

**Checkpoint**: The constitution's guarantee is measured, not assumed

---

## Phase 9: The command-line host (FR-032 to FR-035)

**Goal**: `zm` prints the answers while a game is played

**Independent Test**: `cargo test -p zmachine-cli --test inspect`

- [x] T049 [P] Failing unit tests in `crates/zmachine-introspect/src/json_tests.rs` (I§2; research R13): each answer type renders as `{"exactness":…,"value":…}`; an unknown whole answer and an unknown field each render as `{"unknown":"<reason>"}`; three-valued fields render as the data model's words; text is escaped (quote, backslash, control characters, non-ASCII passed through as UTF-8); output is one line with no trailing newline and fields in the data model's order; a small strict JSON reader written in the test parses every rendering
- [x] T050 Implement `crates/zmachine-introspect/src/json.rs` until T049 passes
- [x] T051 Failing tests in `crates/zmachine-cli/tests/inspect.rs` (contracts/cli.md): `--inspect FILE` on Zork I and on Advent writes one line per stop, each parsing as JSON with `spec`, `stop` counting from 1, `wants` of `line` or `key`, `family`, and the four answers; the first line names "West of House" / "At End Of Road"; with a script, the number of lines equals the number of stops; FILE is emptied at start, so two runs leave one run's lines; `--inspect -` writes the lines to standard error, each beginning `{"spec"`, and nothing to standard output that is not the game's; standard output and the transcript are byte for byte the same with and without `--inspect`; nothing of the inspection is in the transcript; FILE is not resolved inside `--dir`; an unwritable FILE is a usage error with status 2; `--tree` prints the object tree, `--dictionary` the dictionary, both together one object with both, each exiting 0 without playing, for a recognised and an unrecognised story file; a file that cannot be loaded gives status 2; the new options are in `--help`
- [x] T052 Implement `--inspect`, `--tree` and `--dictionary` in `crates/zmachine-cli/src/lib.rs` and `main.rs` until T051 passes; the CLI crate gains a dependency on `zmachine-introspect`; run the guard

---

## Phase 10: User Story 6 - Another interpreter could implement it (Priority: P6)

**Goal**: The published specification, complete and checked

**Independent Test**: `cargo test -p zmachine-introspect --test spec_agreement`, and the blind reading

- [x] T053 [US6] Failing tests in `crates/zmachine-introspect/tests/spec_agreement.rs` (SC-011, FR-030, FR-031): the query names in `docs/introspection.md`'s section headings equal `QUERIES`, in order; the document's version equals `SPEC_VERSION` and the `spec` field `zm` prints; every unknown reason and every field name of the JSON renderings appears in the document, and every reason and field the document lists is one the implementation can produce; every "I§" citation in the crate's tests names a section that exists; the document contains no Rust type name and no path into `crates/` (FR-029)
- [x] T054 [US6] Complete `docs/introspection.md` until T053 passes: sections 1 and 2 in full, with the JSON form of every answer and one worked example per query from each game; section 10, every known limit (change of player in Inform; ties for the player in ZIL; a translated library's compass; library and ZIL versions not tested; what makes scope approximate; that exits are the room's table and the game has the last word); section 11, what kind of change moves which part of the version
- [ ] T055 [US6] The blind reading (SC-012): give an independent reviewer, with no access to `crates/zmachine-introspect/src`, the document and the story files, and ask for the six answers at five named walkthrough steps in each of Zork I and Advent, worked out from the bytes of saved states; compare with `zm --inspect`. For every difference decide from the document's text which is right; correct the document where it was unclear and the implementation, test first, where it was wrong. Record the outcome, and each difference found, at the end of this file

---

## Phase 11: Polish & Cross-Cutting Concerns

- [x] T056 [P] Test in `crates/zmachine-introspect/tests/speed.rs`, marked `#[ignore = "timing; run with --release"]` (SC-010): all six queries at every command prompt of the Advent version 8 walkthrough, the slowest under 5 ms; making an introspector under 50 ms
- [x] T057 [P] Update `README.md`: the introspection interface in the status and the phases, `docs/introspection.md` and the new crate in the layout, `zm --inspect`, `--tree` and `--dictionary`, and the known limits in two or three lines
- [x] T058 [P] Update `.github/workflows/ci.yml` only if a new ignored test needs something the workflow lacks; confirm `cargo test --workspace --release -- --ignored` passes locally with dfrotz installed
- [x] T059 Run every step of `specs/003-introspection/quickstart.md` and correct the document or the code wherever they disagree
- [x] T060 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` and fix what they report; confirm with `git diff main -- crates/zmachine-core/tests crates/zmachine-cli/tests crates/zmachine-core/src` that no test from features 001 or 002 changed except the one addition to `coverage.rs` (T008); list at the end of this file what was done differently from the plan, including any test that was not seen to fail before its code was written

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies. T002 before any test that cites the document
- **Foundational (Phase 2)**: blocks everything. T004 → T005; T006 → T007; T010 → T011; T012 → T013 (needs T005); T015 needs T005 and T007. T016, T017 and T018 need nothing but the sources and must be finished before Phase 5 and Phase 6 begin, and before any adapter code exists
- **Phase 3 (tree and dictionary)**: needs T013
- **Phase 4 (US1)**: needs Phase 3, T014 and T015. Recognition (T023 to T026) before location; location before inventory
- **Phase 5 (US2)**: needs Phase 4 and T016, T017
- **Phase 6 (US3)**: needs Phase 4 and T018; uses the inventory from T033
- **Phase 7**: needs Phase 4's recognition and, for Inform directions, T038
- **Phase 8 (US5)**: T046 and T047 can be written once T015 exists and run red or trivially until the queries exist; they are complete only after Phase 7
- **Phase 9 (CLI)**: needs all six queries
- **Phase 10 (US6)**: needs everything before it
- **Polish**: last

### Within Each Phase

- A test task is committed, run and seen to fail before the implementation task after it is begun
- A documentation task (T022, T026, T030, T034, T039, T043) follows the code it describes, in the same phase, so the document never runs ahead of what is built or behind it

### Parallel Opportunities

- T003, T009, T010 and T014 alongside the core's additions
- T016, T017 and T018 alongside each other and alongside all of Phase 2 and Phase 3: they touch only `stories/`
- Within a story, the unit-test task marked [P] alongside the walkthrough-test task: different files
- T046 and T047 alongside each other; T056, T057 and T058 alongside each other

---

## Parallel Example: Phase 2

```text
T004 → T005 (the view)            ‖  T016 Zork I expected exits, from the ZIL source
T006 → T007 (clone)               ‖  T017 Advent expected exits, from advent.inf
T010 → T011, T012 → T013          ‖  T018 expected scope positions for both games
T014 testkit builders             ‖
```

---

## Implementation Strategy

### MVP First

Phases 1 to 4. At the end of Phase 4, `Introspector` recognises both games and reports
location and inventory, checked against the games' own text at every prompt of both
walkthroughs. That is the smallest thing that proves the approach: family recognition,
finding the player, and reading state without touching it.

### Incremental Delivery

1. Phases 1 and 2: nothing visible, everything measurable
2. Phase 3: any story file's tree and words
3. Phase 4: location and inventory (MVP)
4. Phase 5: exits. With Phase 9 this is all the first demo's compass needs
5. Phase 6: scope
6. Phases 7 to 11: parts of speech, the read-only proof, the host, the document

### What was learned from feature 002

- Tests there were written after code that already passed by hand. Here each test task
  says "failing", and T060 asks for a list of any that was not seen to fail.
- The plan's research was wrong in three places until re-tested. T016 to T018 are made
  from the games' sources, not from the interpreter's output, so the expectations are
  independent of the code they judge.

---

## What was done differently

Recorded for T060.

**Test-first: where it held and where it did not**

Every query's tests were written, run and seen to fail before the code that passes them,
and each failing run is in the commit message of its test commit. The exceptions:

- **Tests that could only pass, by construction.** The read-only tests (T046) and the
  guard against interior mutability (T008) check a property the compiler already
  enforces, so they passed on first run. Each was shown able to fail: T008 by planting a
  `Cell` in the core and seeing it caught, T046 by a control inside the test that gives a
  copy of the machine a turn and requires its memory to differ.
- **The robustness tests (T047)** passed on first run: no damaged file made a query
  panic, so T048 had nothing to fix. They were not seen to fail.
- **Rule-breaking recognition tests (T024).** Against the stub that recognises nothing,
  the ten tests that expect "unrecognised" passed trivially; only the five positive
  cases failed. Each rule's test is meaningful only together with the positive case,
  which shows the fixture is otherwise valid.
- **The JSON rendering (T049, T050).** The rendering code had to exist for its tests to
  compile, so it was written with the tests and switched off; the tests failed by
  assertion, then the switch was removed.
- **The testkit's new builders (T014)** and **the walkthrough driver (T015)** are test
  infrastructure, written with their own tests in the same step.
- **The agreement tests (T053)** were committed one commit early, with the CLI work,
  while still failing four of five.

**Expectations and checks that were corrected after their first run**

In each case the answer stood and the test's own data or rule was wrong. Each is in its
commit message.

- T024: a ZIL rule 3 case expected "unrecognised" for four direction words; the rule
  asks for four words, not four directions.
- T019: the size of Zork I's dictionary, 684 words, not the 697 first written.
- T015, T051: prompt and line counts, measured and recorded (332 command prompts in
  Zork I, 172 in each Advent build).
- T027: the location check's rules. A room heading ends ", in the <vehicle>" when the
  player is in one; a move between two rooms that share a printed name is not announced
  (Zork I's two Mirror Rooms). Both are now in the specification, section 6.3.
- T031: the inventory check. A copy of the machine asked for `inventory` goes on to print
  whatever else happens on its turn; only the listing itself is compared.
- T037, T049: three fixture mistakes in unit tests.

**Findings that changed the specification or a success criterion**

- **SC-003 reworded.** Advent's walkthrough makes only 67 moves through fixed exits, not
  the 100 the criterion asked for. A tour that follows reported exits was added (400
  moves in Zork I, 450 in each Advent build). It found the caveat the specification
  already stated, in practice: arriving in Advent's first forest room, the game's own
  code moves the player to the second half the time. The exit is reported as the room's
  table has it; the criterion now names this case.
- **One of Zork I's 38 hand-made scope positions falls on a tie for the player**
  (walkthrough line 245). The answer there says the player is unknown and leaves out the
  tied objects and what they hold, as specified; what that entry lists as carried is not
  required at that position, and the check records the line.
- **The walkthrough never enters the dark in Advent**, so darkness is tested by a short
  route of its own (lamp off in the Debris Room).
- **`container` is not consulted** by the Inform scope rule: contents show if the holder
  is a supporter, open, or transparent. The attribute was dropped from the
  specification's table.

**Measured results**

| Criterion | Result |
| --- | --- |
| SC-001 | Transcripts and all 11 save files identical with every query asked at every stop |
| SC-002 | Location known and agreeing at 332 + 172 + 172 prompts |
| SC-003 | 171 (Zork I) and 67 (Advent) walkthrough moves arrived; tours of 400 and 450 more; none astray but the forest |
| SC-004 | 59 hand-made entries in 49 rooms (Zork I), 50 in 50 (Advent, both builds): all match |
| SC-005 | Inventory agrees wherever known; unknown at 6 of 332 Zork I prompts (1.8%), none in Advent |
| SC-006 | 38 positions (Zork I, one on a tie), 39 (Advent, both builds): all match |
| SC-007 | Tree and dictionary equal an independent reading for all seven story files |
| SC-008 | czech (three versions) and praxix unrecognised |
| SC-009 | Over 3,000 damaged files, no panic |
| SC-010 | Slowest prompt 0.36 ms against 5 ms |
| SC-011 | Document and implementation agree, by test |
| SC-012 | See the blind reading, below |

**Not done, or done less than written**

- **T019's independent check** is a second reader written in the test from the Standard,
  with the default alphabet only. None of the committed story files has its own
  alphabet; the custom tables are covered by unit tests on built files.
- **T047** uses the damage patterns of the core's robustness tests but not its exact
  code, and does not separately simulate "tables damaged in memory after recognition":
  random damage to recognised files covers the same ground (over 100 damaged files were
  still recognised and then queried).
- **T059, the quickstart**, was run command by command in the working tree, not on a
  fresh clone.
- **The expected exits and scope (T016 to T018) were written by independent agents**,
  from the games' sources and Frotz's transcripts, without sight of the crate.
- **An example program** (`examples/answers.rs`) was added to print all six answers at
  one moment, for the blind reading. It is not part of the library.

