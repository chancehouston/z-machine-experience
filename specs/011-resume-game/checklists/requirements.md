# Specification Quality Checklist: Resume the Game

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

- The project owner chose this feature on 2026-11-13, to come before the map, after a
  review of the map's design found that the app starts a new game at every launch.
- Two questions were put to the owner before the spec was written and answered on
  2026-11-13: automatic resume only (the game's SAVE and RESTORE stay as they are),
  and on opening the player is carried straight on. No marker remains.
- **This feature replaces FR-005 of feature 004** ("Each launch of the app MUST start a
  new game"), which the owner decided then and has now decided otherwise.
- **User Story 4 was not asked for and was found while writing.** New game is offered
  today only when a game has ended or faulted. Until now a player started over by
  closing the app. With resume that no longer works, so a way to start over during
  play is needed or the player is held in their first game. The spec requires one that
  asks first, and leaves where it sits on the screen to the plan and the owner.
- **Five matters were settled by assumption, not by the owner**, and are listed as such
  in the spec: a fault is not kept; an ended game is kept; starting over asks and New
  game after an ending does not; the text in the command field is not kept; the newest
  turn is announced to a screen reader on reopening.
- **The figure 2,000 turns in FR-020 to FR-022 and SC-004 is mine.** It is about six
  times Zork I's walkthrough. It is there to stop a long game being slower to open
  than a short one, and it bears on how the plan may bring a game back. The owner may
  change it.
- **Not known**: how long bringing back a long game takes by any method. Nothing has
  been measured. FR-020 requires the measurement.
- SC-003 and SC-006 give counts (100 moments, 1,000 damaged games) so that they can be
  tested; the counts are mine.
- SC-008 needs the owner and their phone.
- "Written for non-technical stakeholders" holds loosely, as for the earlier specs.
