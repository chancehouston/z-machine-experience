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
- **R2. Which word is sent.** The word of the thing's printed name whose form as
  the dictionary keeps it is one of the thing's words, given in full; where no
  word of the name is one of its words, the first of its words as the dictionary
  stores it. (Changed in building, 2026-11-14. The plan sent the first word of
  the property. On Zork I the leaflet's first word is `advert`, cut from
  "advertisement", so Drop would have put `drop advert` in the transcript. With
  this rule it is `drop leaflet`. Both rules read only the story file; this one
  reads as the player would have typed it.)
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

## Stage 2 (researched 2026-11-14, after stage 1's trial)

- **R10. What an item is for, in an Inform story.** The library numbers its flags
  alike in every story: clothing 2, container 4, edible 6, light 9, on 13, open 14,
  openable 15, switchable 21, worn 26. These agree with the numbers the interface
  already uses (3, 5, 9, 14, 16, 17, 20, 23, 26). The library has no flag for a
  thing to read or to drink, so an Inform item is never offered Read or Drink. It
  marks a thing as giving light only while it does: an unlit lamp cannot be told
  from any other switchable thing.
- **R11. In Zork I (release 119).** Read from the story file by which objects carry
  each flag: 16 is on the leaflet, the map, the label, the book, the guidebook, the
  matchbook, the manual and the prayer (readable); 18 on the sack, the egg, the
  bottle, the coffin, the buoy, the mailbox, the basket (holds things); 10 on the
  altar, the pedestal and the tables (a surface: holds things and is not opened);
  20 on the lunch and the garlic (food); 21 on the water (drink); 31 on the lantern,
  the torch and the candles (a light that is turned on and off); 25 on the torch
  and the candles only (a flame: lit with something burning, which is a pairing,
  and not offered). No carried thing is worn in Zork I. 11 (open), 12
  (see-through) and 19 (lit, which is "on") are in the pack already. These are
  read off the file, not yet proven: R13.
- **R12. The pack's new keys** (format version 5): `holds-attribute`,
  `surface-attribute`, `read-attribute`, `food-attribute`, `drink-attribute`,
  `switch-attribute`, `flame-attribute`, `wear-attribute`. Each is an attribute
  number, given once, and optional. Without a key the interface says "unknown" for
  what it would have told, and the card offers nothing for it.
- **R13. Proof of the pack's entries**, as every entry is proven: by the game.
  Along the walkthrough, at each stop, each action the card would offer is tried
  on a copy and what the interface then reports is compared with what the action
  should have done: Open, then open is yes; Close, then no; Turn on, then on is
  yes; Turn off, then no; Eat and Drink, then the thing is no longer carried;
  Read, then the copy came back to the prompt having printed something. The counts
  of agreed and differed are recorded for each action; a difference fails the
  build until its cause is written in the ledger. This proves that what is offered
  works. It does not prove that nothing else would: an action is never tried on a
  thing whose flags do not offer it.
- **R14. The verbs.** Zork I, Adventure and Anchorhead each have `open`, `close`,
  `turn`, `on`, `off`, `read`, `eat`, `drink`, `wear`, `remove` and `take` in their
  dictionaries. The commands are `open X`, `close X`, `turn on X`, `turn off X`,
  `read X`, `wear X`, `remove X`, `eat X`, `drink X`, each offered only where every
  word of it is in the story's dictionary.
- **R15. Items inside another** keep stage 1's rule: a card, and no action. The
  game may have to take the thing out first, and the card does not guess how.
- **R16. The token's kind** is the first that applies of: lamp (a switch-light in
  Zork I; in an Inform story a thing giving light now), container, page, food
  (food or drink), clothing; else plain. A lamp's token shows whether it is on. It
  is drawn by the app from the layer's word for the kind.
- **R17. Room for the buttons.** A card may have four or five actions. They are in
  one row that scrolls sideways, after the close control and its clear space.
