# Specification Quality Checklist: The Map, First Slice

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

- The owner opened the map's design on 2026-11-13. Eleven of their decisions that day
  are listed in the spec under "Decided by the owner", the last two (the band plus
  the opened-out sheet; the map of the game in progress only) answered just before
  it was written. No marker remains.
- **Seven matters were settled by me and are listed as such** for the owner to confirm
  or change: the mark for a way into the dark is kept; a refused way is drawn once
  tried; names are shown where light is not known; the status line stays; a group of
  four moves to its own sheet; no mark for a new way and no arrowhead; the bars of
  SC-003 and SC-004.
- **This is the largest specification in the project** (eight user stories, 46
  requirements). It replaces the compass, changes how the keyboard behaves and adds
  the map in one slice, because the owner's decisions tie them: with the keyboard
  always up there is no room for a compass beside a map. The plan should consider
  building it in stages behind one pull request, or as more than one.
- **FR-006 needs something the core does not give today**: the layer cannot tell that
  a game undid a turn or restarted. The review found this. The plan must say how.
- **The figures in SC-003 and SC-004 come from a spike**, a throwaway program
  (`crates/zmachine-corpus/examples/map_spike.rs` on branch `spike-map-layout`, not
  committed), run on the two walkthroughs only. FR-043 asks for a second order of
  exploring because the spike did not try one.
- **Four things are listed as not known**, the first being whether a third of the
  screen is enough. They are for the owner's phone to answer (SC-009).
- "Written for non-technical stakeholders" holds loosely, as for the earlier specs.
- "No implementation details": the spec names no code. It does fix rules of placement
  (FR-009 to FR-014) in detail, because those rules are what the owner chose from
  pictures, and are the behaviour, not the means.
