# Specification Quality Checklist: Compass Look-Ahead

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-11-13
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

- The project owner chose this feature on 2026-11-13 from four put to them (look-ahead,
  the map, the scene and graphical interface, Android).
- Two questions were put to the owner after the draft and answered on 2026-11-13: a way
  the game refuses is drawn as no way at all; a way on which the copy's game ends keeps
  its question mark. No marker remains.
- **SC-002's "nine in ten" is not measured.** It is a target written before any trial
  was run. How many of the 198 conditional ways end the copy's game or turn on chance
  is not known; both stay conditional. The plan's research must measure this and the
  figure be corrected to what is found, before tasks are written.
- **Since measured** (plan, research R1 and R2, 2026-11-13): 482 of 495 such ways
  settle with three copies, without the time limit in force. Whether nine in ten holds
  with it is still to be shown (research R7).
- **FR-016 names no time.** The limit is left to the plan, to be set from measurement,
  as the earlier features' limits were.
- The 55%, 96.6% and 98.1% figures are those of `stories/corpus/README.md` as written
  on 2026-11-12. The living-room case was seen by hand on 2026-11-13
  (`specs/008-darkness/tasks.md`, T045).
- `docs/introspection.md` is named in FR-022 because the constitution makes that file
  the contract; this is the one place a file is named.
- "Written for non-technical stakeholders" holds loosely, as for the earlier specs.
