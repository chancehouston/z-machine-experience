# Feature Specification: Corpus Testing of Introspection

**Feature Branch**: `005-corpus-testing`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "Corpus testing of introspection. A way to measure, across
many story files, how often the introspection interface's reports about exits are right,
wrong or undetermined. Ground truth comes from look-ahead: at each place reached, every
direction is tried on a throwaway copy of the running game, and what the copy does is
compared with what introspection reported." Decisions made by the project owner on
2026-11-12: introspection is the feature to push; testing on many games is wanted; the
owner holds no story files, so every story comes from a recorded public source; look-ahead
in the compass, per-game metadata and a graphical UI are later features.

## User Scenarios & Testing *(mandatory)*

The users of this feature are the project owner and the people who work on the
introspection interface. Nothing here is seen by a player. The purpose is to replace
"introspection seems right on one walkthrough" with figures across many games, so that
the next decisions (look-ahead in the compass, per-game metadata, a graphical UI) rest on
evidence.

Three words are used throughout:

- **Trial**: trying one direction on a throwaway copy of the game at one place, and
  noting what the copy did. The game being played is not touched.
- **Place**: a state of a game at a command prompt, in some room.
- **Verdict**: what a trial says about one of introspection's reports: confirmed, wrong,
  or, where introspection claimed nothing, what the game in fact did.

### User Story 1 - See how right introspection is on one game (Priority: P1)

The owner runs the corpus test on a story already in the repository and gets a report:
for every exit introspection reported at every place visited, whether a trial confirmed
it.

**Why this priority**: It is the whole idea in its smallest form. With one game and a
walkthrough it already yields figures nobody has today, and it proves the trial is a
sound instrument before more games are added.

**Independent Test**: Run the test on Zork I with the project's walkthrough. A report is
produced with a count for each verdict, and the counts add up to the number of
directions tried.

**Acceptance Scenarios**:

1. **Given** a story with a walkthrough script, **When** the corpus test runs, **Then**
   at every command prompt along the walkthrough every direction the game has is tried
   once on a copy, and each trial is given exactly one verdict.
2. **Given** introspection reported that a direction leads to a room, **When** the trial
   ends in that room, **Then** the verdict is "confirmed"; **When** it ends anywhere
   else, or does not move, **Then** the verdict is "wrong", with what happened.
3. **Given** introspection reported that a direction is not an exit, **When** the trial
   leaves the player in the same room, **Then** the verdict is "confirmed"; **When** the
   player is somewhere else, **Then** the verdict is "wrong".
4. **Given** introspection reported a direction as conditional or unknown, **When** the
   trial runs, **Then** the verdict records what the game did (moved, to which room; or
   stayed), and is never "wrong": introspection claimed nothing.
5. **Given** the walkthrough is replayed with trials and without, **When** the two
   transcripts are compared, **Then** they are identical byte for byte.

---

### User Story 2 - A wrong report stops the build until it is explained (Priority: P2)

When a trial shows introspection was wrong, the test fails, naming the story, the place
and the direction. The owner either fixes the cause or records the case in a ledger with
its cause, after which the test passes and the case is counted as a known divergence.

**Why this priority**: Figures alone are easy to ignore. A failing test makes each wrong
answer a decision, and the ledger becomes the list of what introspection must learn.

**Independent Test**: Run the test with an empty ledger on a story known to have a wrong
report; it fails and names the case. Add the case to the ledger; it passes and the
report counts the case as known.

**Acceptance Scenarios**:

1. **Given** a "wrong" verdict that is not in the ledger, **When** the test finishes,
   **Then** it fails, and the failure names the story, the room, the direction, what was
   reported and what happened, by number and identifier.
2. **Given** the same case recorded in the ledger with a cause, **When** the test runs,
   **Then** it passes and the report counts the case under "known divergences".
3. **Given** a ledger entry that no trial matches any longer, **When** the test runs,
   **Then** it fails and names the stale entry, so the ledger cannot outlive its cases.
4. **Given** a ledger entry with no cause written, **When** the test runs, **Then** it
   fails.

---

### User Story 3 - Reach places no walkthrough visits (Priority: P3)

For a story with no walkthrough, or to go beyond one, the test crawls: from each place
it has reached it follows every direction that a trial showed to move the player, and
runs trials at each new room, within a stated budget.

**Why this priority**: A walkthrough exists for two stories today. Without crawling,
every other story contributes one room.

**Independent Test**: Run the crawl on a story with no walkthrough. The report lists
more than one room visited, states the budget and whether it was reached, and running it
twice gives the same report.

**Acceptance Scenarios**:

1. **Given** a story and no walkthrough, **When** the crawl runs, **Then** it starts at
   the opening, visits every room it can reach by directions alone within the budget,
   and runs trials in each.
2. **Given** the same story, seed and budget, **When** the crawl runs twice, **Then**
   the two reports are identical.
3. **Given** a move on which the game ends, asks a question, or stops with a fault,
   **When** the crawl meets it, **Then** that branch is recorded and not followed, and
   the crawl goes on elsewhere.
4. **Given** a story with a walkthrough, **When** the test runs, **Then** it crawls
   outward from the places along the walkthrough as well, within the same budget.
5. **Given** the budget is reached, **When** the test finishes, **Then** the report says
   so and gives the number of rooms visited and the number of rooms that a reported
   exit leads to and that were never reached.

---

### User Story 4 - Add a game without committing it (Priority: P4)

The owner adds a story to the corpus by adding one entry to a manifest: where it comes
from, its digest, and whether its license lets it be committed. A story that may not be
committed is fetched into a folder the repository ignores; tests that need it report as
skipped where it is absent.

**Why this priority**: The games that may be committed are few. Breadth depends on
games that are free to download but not to redistribute, and the constitution forbids
committing those or their text.

**Independent Test**: Add a manifest entry for a story that is not committed. Without
the file, the test for that story is reported as skipped. After fetching, it runs. No
story file and no game text appears in what the repository tracks.

**Acceptance Scenarios**:

1. **Given** a manifest entry and no file, **When** the tests run, **Then** that story's
   test is reported as skipped, not passed, and the run as a whole does not fail.
2. **Given** a manifest entry, **When** the fetch is run with the owner's approval,
   **Then** the file is downloaded from the recorded address into the ignored folder and
   its digest is checked; **When** the digest differs, **Then** the file is refused and
   removed.
3. **Given** a file in the ignored folder whose digest matches no manifest entry,
   **When** the tests run, **Then** it is not used.
4. **Given** a story marked as committed, **When** the manifest is checked, **Then** its
   source, revision, digest and license text are recorded beside the file, as the
   constitution requires; a story marked committed without them fails the check.
5. **Given** any story not cleared for redistribution, **When** its report is written,
   **Then** the report holds numbers and identifiers only: no room name, no object name,
   no line of the game's text.

---

### User Story 5 - One table across the corpus (Priority: P5)

The owner reads one summary: for each story and for each family, how many reports were
confirmed, wrong, conditional and unknown, and for the conditional and unknown ones, how
often the game in fact let the player through.

**Why this priority**: This is the evidence the later decisions need: how often the
compass would have said "may be blocked" when the way was open, and how often
introspection could not say at all.

**Independent Test**: Run the corpus. The summary has a row for each story that ran, a
row for each family, and totals; skipped stories are listed as skipped; the figures in
each row equal those in that story's own report.

**Acceptance Scenarios**:

1. **Given** the corpus has run, **When** the summary is written, **Then** it has for
   each story the counts of each verdict, the rooms visited, and whether the budget was
   reached.
2. **Given** some stories were skipped, **When** the summary is written, **Then** they
   are listed as skipped and are not counted in any total.
3. **Given** the summary in the repository and a fresh run on the same stories,
   **When** they are compared, **Then** they are identical; a difference fails the test,
   so a change in introspection's accuracy is seen in review.

---

### Edge Cases

- A trial in which the game ends (the player dies and the game stops), asks a question,
  or faults: recorded as its own outcome; it is "wrong" only against a report of a fixed
  exit or of no exit, as the rules above say for "anywhere else" and "the same room".
- A trial that moves the player and prints a refusal, or stays and prints a room
  description: only where the player is counts; the text is not read.
- A trial whose result depends on chance: the copy carries the game's random state, so
  the trial shows what the same move would do next in the game itself.
- A direction introspection reports as unknown for the whole room: every direction the
  family has is still tried.
- A place where introspection cannot say where the player is, before or after the
  trial: the trial's outcome is "location unknown" and gives no verdict on the exit.
- A room reached twice in different states (a door now open): each is a place; the
  report counts places and rooms separately.
- The player moved, but to a room that is the same room object (a maze that returns the
  player to where they stood): counted as stayed, and noted as a limit of the instrument.
- A game whose moves take game time: a trial advances the copy only.
- A story whose family introspection does not recognise: it is listed, with every
  report "unknown", and no trials are run.
- A walkthrough that reaches a prompt that is not a command prompt (a question): no
  trials are run there.
- A source address that no longer answers, or answers with a different file: the fetch
  fails and says which; nothing is used.
- Two manifest entries with the same digest, or one entry marked both committed and
  fetched: the manifest check fails.

## Requirements *(mandatory)*

### Functional Requirements

**The trial**

- **FR-001**: A trial MUST run on a copy of the game that shares nothing with the game
  it was copied from. After any number of trials, the game copied from MUST be
  unchanged: the same memory, the same position, the same random state.
- **FR-002**: A run with trials and a run without MUST produce identical transcripts of
  the game being played (constitution, Principle IV).
- **FR-003**: A trial MUST send the copy one command: a word the game's own dictionary
  holds for that direction, as the introspection interface reports it. No other text is
  sent (Principle III).
- **FR-004**: The outcome of a trial MUST be one of: moved to a room (which one);
  stayed; the game ended; the game asked a question; the game faulted; location
  unknown. Where the player is, before and after, is as the introspection interface
  reports it.
- **FR-005**: Trials MUST exist only in the project's testing tools. No result of a
  trial may reach the introspection interface's answers, the experience layer's
  snapshots, the C interface or the app.

**Verdicts**

- **FR-006**: Each trial MUST be given exactly one verdict from what introspection
  reported for that direction at that place:

  | Reported | Trial outcome | Verdict |
  | --- | --- | --- |
  | Leads to room X | moved to X | confirmed |
  | Leads to room X | anything else | wrong |
  | Not an exit | stayed | confirmed |
  | Not an exit | moved to a room | wrong |
  | Not an exit | ended, question or fault | not an exit, with that outcome noted |
  | Conditional | any | conditional, with the outcome |
  | Unknown | any | unknown, with the outcome |
  | any | location unknown | no verdict, counted separately |
  | Leads to the room the player is in | stayed | no verdict: a move to the same room cannot be told from staying |

- **FR-007**: The counts of all verdicts for a story MUST add up to the number of trials
  run on it.

**Reaching places**

- **FR-008**: Where the project holds a walkthrough for a story, trials MUST be run at
  every command prompt along it.
- **FR-009**: The test MUST crawl: from each place reached, follow each direction whose
  trial moved the player to a room not yet visited, and run trials there. A room is
  visited once by the crawl.
- **FR-010**: The crawl MUST have a stated budget, the same for every run of a story,
  and MUST stop when it is reached. The report MUST say whether it was.
- **FR-011**: The same story, seed, walkthrough and budget MUST give the same report on
  every run and on every platform the project tests on.
- **FR-012**: A branch on which the game ends, asks a question or faults MUST be
  recorded and not followed.

**The ledger**

- **FR-013**: A "wrong" verdict MUST fail the test unless the ledger holds that case:
  the story's digest, the room, the direction, what was reported, what happened, and a
  written cause.
- **FR-014**: A ledger entry that matches no trial, or has no cause, MUST fail the test.
- **FR-015**: This feature MUST NOT change the introspection interface's rules to make
  a case pass. What the corpus finds is recorded; changing the rules is later work.

**The corpus**

- **FR-016**: A manifest in the repository MUST list every story in the corpus with:
  SHA-256 digest, size, source address, Z-machine version, family as introspection
  detects it, license standing, and whether it is committed or fetched.
- **FR-017**: A story is identified by its digest. A file whose digest matches no entry
  MUST NOT be used.
- **FR-018**: A story marked committed MUST have its license text, source, revision and
  digest recorded beside it (Principle VI). The manifest check MUST fail otherwise.
- **FR-019**: A story marked fetched MUST NOT be tracked by the repository, and neither
  may any transcript of its text.
- **FR-020**: Fetching MUST download only from the address the manifest records, check
  the digest, and refuse a file that does not match. Fetching MUST be a step a person
  runs; no test and no automated check may download a story.
- **FR-021**: A test that needs a story that is absent MUST be reported as skipped, not
  passed (Principle VI).
- **FR-022**: The first corpus MUST hold Zork I and Advent, already in the repository,
  and Zork II and Zork III, each added only with the project owner's approval for that
  file, recorded as Zork I's is, with the same caveat about what the license covers.
- **FR-023**: The first corpus MUST also hold at least six fetched stories, so that the
  fetched path is exercised and the figures cover about ten games. They are found on
  public archives in planning: mostly games of the Inform family, and any of the ZIL
  family beyond the three Zorks whose terms are clear. Each is added only with the
  project owner's approval for that file (decided by the project owner, 2026-11-12).

**Reports**

- **FR-024**: Each story that runs MUST have a report: places and rooms visited, the
  rooms that a reported fixed exit leads to and that were never reached, the count of each
  verdict, the count of each trial outcome under "conditional" and under "unknown", the
  known divergences, the budget and whether it was reached.
- **FR-025**: A summary MUST give the same figures for each story, for each family, and
  in total, and list the stories skipped.
- **FR-026**: Reports and the summary MUST hold numbers and identifiers only (object
  numbers, direction names, digests). They MUST NOT hold room names, object names or any
  of a game's text, for any story.
- **FR-027**: The reports and summary for the committed stories MUST be kept in the
  repository, and the test MUST fail when a fresh run differs from them.
- **FR-028**: The reports for fetched stories MUST be kept in the repository as well,
  and checked the same way wherever the story is present.

**Room for more**

- **FR-029**: Verdicts are about exits only. The manifest, the ledger and the reports
  MUST be shaped so that another kind of check (what is in reach, what is carried,
  light) can be added as a new kind of verdict without changing what exists.

### Key Entities

- **Story**: one story file, known by its digest; has a version, a family, a source and
  a license standing; is either committed or fetched.
- **Manifest**: the list of stories in the corpus.
- **Place**: a game at a command prompt: the room, and how it was reached (the step of
  the walkthrough, or the path of the crawl).
- **Trial**: one direction tried on a copy at one place; has an outcome.
- **Verdict**: what a trial says of one report: confirmed, wrong, conditional, unknown,
  or none.
- **Ledger entry**: one known wrong report, with its cause.
- **Report**: the figures for one story. **Summary**: the figures across the corpus.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For Zork I along its walkthrough, every direction at every command prompt
  has a verdict, and the counts add up to the number of trials.
- **SC-002**: The transcript of every walkthrough the project holds is byte for byte the
  same with trials as without.
- **SC-003**: Every "wrong" verdict in the first corpus is either absent or in the
  ledger with a cause; the test passes with no unexplained case.
- **SC-004**: Running the corpus twice gives identical reports, on each platform the
  project tests on.
- **SC-005**: With every fetched story absent, the tests pass, each such story is
  reported as skipped, and the committed stories still run.
- **SC-006**: Nothing the repository tracks under the corpus's reports contains a room
  name, an object name or a line of text from any story; a check proves it for the
  committed stories by searching the reports for every name the story holds.
- **SC-007**: For each story with no walkthrough, the crawl visits more than one room,
  and the report states how many rooms were visited and how many were named by a
  reported exit and never reached.
- **SC-008**: The summary answers, with a figure for each family: of the exits reported
  conditional, how many let the player through; and of all directions tried, how many
  introspection could not answer.
- **SC-009**: The corpus test on the committed stories finishes within ten minutes on
  the project's CI; the whole first corpus within thirty minutes on the owner's Mac.
- **SC-010**: Every test from the earlier features passes unchanged.
- **SC-011**: The first corpus holds at least ten stories, at least six of them fetched,
  and every one has a report.

## Assumptions

- **Where the player is, before and after a trial, is taken from the introspection
  interface.** The instrument therefore trusts introspection's answer to "which room",
  which feature 003 tested on its own. A wrong room would show as wrong verdicts, and
  would be found by reading them.
- **"Moved" means the room object changed.** A game that prints a new description but
  keeps the same room object counts as stayed. This is recorded as a limit, not solved.
- **The crawl moves by directions only.** It opens nothing, takes nothing and solves
  nothing, so it reaches what can be walked to from the opening and from the places
  along a walkthrough. The report says how much of the story that is.
- **The budget is a number of rooms and a number of trials for each story**, chosen in
  planning so that SC-009 holds, and written in the manifest.
- **Walkthroughs are the ones the project already holds** (Zork I, Advent). Writing new
  ones is not part of this feature.
- **A copy of a game carries its random state**, so a trial shows what the next move in
  the game itself would do. The crawl uses a fixed seed.
- **Zork II and Zork III stand as Zork I does**: MIT License from Microsoft, dated
  2025-11-20, with the recorded caveat that whether it covers the compiled story file is
  a legal reading. The owner approves each file when it is fetched.
- **Advent stays a test fixture only**, as its record says; it is committed already and
  nothing here bundles it in an app.
- **Fetched stories come from public archives that offer them for download.** Which
  ones, and each one's license standing, is found in planning and approved by the owner
  file by file. Downloading for testing is not redistribution; nothing fetched is
  committed.
- **No total of a story's rooms is claimed.** The introspection interface has no query
  for it, and this feature does not add one. The report gives the rooms visited and the
  rooms named and not reached (plan, research R10).
- **Reports name rooms and objects by number.** Numbers are facts about a file, not its
  text. For committed stories, a person can look a number up in the story itself.
- **Out of scope**: look-ahead anywhere a player sees it; per-game metadata and the
  amendment to the constitution it needs; a graphical UI; changing introspection's
  rules; checks on anything but exits.
