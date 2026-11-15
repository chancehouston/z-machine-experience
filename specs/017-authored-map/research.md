# Research: An Authored Map for Zork I

- **R1. The form.** Lines of `key: value`, as a game pack's: `presentation: 1`,
  `sha256`, `release`, `serial` first and in that order; then `sheet: N Name` (the
  name optional) and `place: ROOM SHEET X Y`, in any order. Lines that are empty or
  start with `#` are skipped. A comment may not hold a word of a game; a test holds
  the file to digits, the keys and the sheets' names.
- **R2. Refusing.** Anything out of form, a sheet used and not declared, a place
  given twice, two places in one position of a sheet, a place that is not a room of
  the story, or an identity that does not match: the whole file is refused and the
  map is as without one. The session says so in the snapshot's notice once.
- **R3. A room of the story.** The layer asks the interface for the object tree: a
  place is a room when it has the parent the player's first room has.
- **R4. Sheets.** The file's sheet numbers are its own. The map makes a sheet for a
  file's sheet when the first place of it is found, and numbers sheets, as today,
  in the order found. So "Sheet 2 of 3" is the second the player reached, whatever
  the file calls it.
- **R5. Putting a place.** Found for the first time: if the file has it, it goes to
  the map's sheet for its file's sheet, at its position, not inset and in no group.
  Otherwise today's rules, on the sheet of the place left, with every authored
  position of that sheet counted as taken whether or not its place is drawn yet.
- **R6. The first place of a game** starts sheet 1 as today; if the file has it, that
  sheet is the file's sheet for it and it sits at its authored position.
- **R7. Redraw.** Not offered, and not made, where the player's sheet has an authored
  place. Bringing back a kept game, a `redraw` line that meets such a sheet is
  passed over; one that cannot be made for any other reason still fails (feature
  016, K§4).
- **R8. Opening.** `Session::open_kept` and its kin take the file's text beside the
  pack's; `zmx_session_open` gains a pointer for it. A new game of the same session
  uses the same file.
- **R9. Drafting the trial file.** From the walkthrough and long wanders, the ways
  walked among the trial's rooms are gathered with their room numbers; a draft is
  made by the search of feature 016's experiment, run to its end over every
  starting place; it is then adjusted by hand. This is done with scripts outside
  the repository; the note beside the file says what was done and what was
  consulted. The file is the deliverable, not the scripts.
- **R10. What is counted.** On the walkthrough and a wander, with the file: joins
  straight and bent, crossings, and how many joins have a place of the file lying
  between their ends.
