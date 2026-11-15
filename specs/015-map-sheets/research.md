# Research: The Other Sheets of the Map

- **R1. A view of any sheet.** `Map::view` draws the sheet `shown`. It becomes
  `view_of(sheet)`; the player's own is `view_of(shown)`. For another sheet the
  view has no `here`, no ways, no look and no lamp, and its stubs include every
  place's.
- **R2. The order of sheets** is the order they were made, which the map keeps; a
  sheet's number is its place in that order among sheets that hold a place. A group
  of four moving to its own sheet makes a new last sheet and empties none.
- **R3. Where a way leads.** A lead already gives the far sheet's number and the far
  place's name. It gains the far place's id, to centre the sheet on.
- **R4. How the app gets a sheet.** `Session::sheet(at, number)` returns the view;
  the binding returns it as JSON in the form of the snapshot's `map`. It is checked
  against the turn like a card, and takes `&self`: the session is not changed.
- **R5. What the app remembers.** The sheet being looked at and the place to centre
  it on, held by `Game` beside the open card and dropped at every new snapshot and
  when the map closes. The band is always given `showing.map`.
- **R6. A lead as a button.** In the opened-out map a lead's short line and label
  become a button (`map.lead.<n>`, at least 44 points). In the band it is drawing
  only, as now.
- **R7. Centring.** The app centres on, in order: the place it was told to (a
  lead's far place), the player's place, the sheet's first place.
