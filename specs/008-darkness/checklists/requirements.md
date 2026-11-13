# Specification Quality Checklist: Darkness

**Purpose**: Validate specification completeness and quality before proceeding to planning

**Created**: 2026-11-12

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

- Two questions were put to the project owner before the spec was written (what the
  app shows in the dark; whether to mark a way into darkness) and answered on
  2026-11-12; no marker remains.
- Measured on Zork I on 2026-11-12 with temporary tests, since removed. Along the
  walkthrough: 330 prompts, none dark. On a walk of 24 commands into the attic and the
  cellar: 6 prompts dark; lit rooms carry attribute 19 and dark rooms do not; the lamp
  gains attribute 19 when switched on and loses it when switched off; the game was dark
  exactly when neither the room nor the lamp (carried or put down there) had it; and
  global variable 67 was not zero exactly when the game was lit.
- That attribute 19 means lit is a reading of eight rooms and one lamp, not yet a
  proven fact: User Story 2 is its proof.
- The ledger's 87 darkness entries were counted by their cause's wording: Zork I 39,
  Zork II 23, Zork III 12, Mini-Zork 12, the sampler 1. Only stories that get the entry
  lose theirs.
- "Written for non-technical stakeholders" holds loosely, as for the earlier specs.
- Points for the plan to settle with the owner: the words and drawing of the note and
  of the dark-ahead mark; whether the corpus's "made in the dark" verdict applies to
  every unconfirmed report at a dark place or only to some; whether Mini-Zork can be
  proven.
