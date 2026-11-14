# Research: The Inventory Card

- **R1. The words that name a thing.** In Inform stories they are the `name`
  property (number 1), a list of dictionary entries. In Infocom stories the
  property's number is the story's own. It is found once: the property for which
  at least five objects provide it and at least nine in ten of them hold a
  non-empty list of whole dictionary entries; if none or more than one qualifies,
  the words are not known. Measured 2026-11-14 on the files held: one such property
  in each recognised file (Inform: 1; Infocom: 4, 17, 18, 19). `curses.z5` also had
  a second small one (9 objects of 502) and is not recognised as either family, so
  it is outside this; the nine-in-ten rule is judged against the larger. A pack
  may name the property (`noun-property`), which then is used without the search.
- **R2. Which word is sent.** The first of the thing's words in the order its
  property lists them, as the dictionary stores it. *Rejected*: the longest, or
  the one nearest the printed name: each is a guess at what reads best, and the
  word is seen only in a command sent to a copy, or in a Drop the player chose.
- **R3. The verbs.** As feature 013 found the word for looking: `examine` then
  `x`; `drop`. `look.rs` becomes `verbs.rs` and holds all three.
- **R4. The peek.** A clone of the machine is given the line and run, as
  look-ahead runs a copy, under look-ahead's limit for one copy and with the
  game's own chance (so that the same peek gives the same text, FR-008). Its
  output is taken as the session takes a turn's. It counts as described only if
  it stops wanting a line at the command prompt's address. The game's machine is
  never touched: FR-004 holds by construction and is tested by comparing the kept
  game and the snapshot before and after.
- **R5. Where the card lives.** Not in the snapshot: making it costs a run of the
  game, and FR-007 forbids examining what was not tapped. `Session::card(at, index)`
  returns it; the binding returns JSON. `index` counts the shown items in the
  order the strip lists them, contents after their holder.
- **R6. Actions.** `Session::tap_action(at, index, action)` makes the item's
  actions again from the item (no peek) and sends the chosen one's command as a
  tap. An item inside another has no Drop.
- **R7. The app.** The strip's items become buttons where the snapshot marks them.
  A touch asks the engine for the card off the main thread, as a command is sent.
  The card is held by the view with the stamp it was made at and dropped when the
  turn count or generation changes, on its close control, and on a touch outside.
  It is drawn over the band. With no band (a game with no map) it is drawn in the
  band's place at the band's height.
- **R8. Speaking.** The card's description is posted by the `Announcer`, as a
  reply's text is; `-speakAlways` shows it for tests.
- **R9. Stage 2, in outline.** Inform stories: the library's flags are numbered
  alike in every story (container 4, edible 6, light 9, on 13, open 14, openable
  15, switchable 21, clothing 2, worn 26). Infocom stories: a pack names each flag
  the story has. The interface reports a thing's kinds; the layer turns kinds and
  states into actions; the token shows the kind. To be researched: Zork I's flag
  numbers, and what "readable" is in each family.
