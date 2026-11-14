# Research: Touch the Room to Look

- **R1. Where the word comes from.** `Introspector::dictionary` gives every word of
  the story's main dictionary as the story keeps it. The layer looks for `look`,
  then `l`, and takes the first found. Both are shorter than the six letters a
  version 3 dictionary keeps, so no cutting is needed; the search cuts all the
  same, so that a longer candidate added later is found. *Rejected*: reading the
  grammar tables to find the verb by what it does; no table says what a routine
  does. *Rejected*: a word in a game pack; that is authored content and needs an
  amendment for no gain here.
- **R2. When it is found.** Once, when the session is opened, and kept with the
  session. The dictionary does not change in play in any story met (a story may
  write to it; none does for this word). A new game of the same story keeps it.
- **R3. Where the map says so.** A new part of the map's view, `look`, present when
  the game waits for a command, the player's place is known and the story has the
  word. It is not one of `ways`: it is not a way, and the app places ways by name.
- **R4. The touch in the app.** A SwiftUI `Button` inside a view that has a drag
  gesture takes the drag. A tap gesture does not: it fails once the finger moves,
  and the drag goes to the band. The place gets a tap gesture, the button trait and
  an accessibility action. To be confirmed by the tests of dragging, which start on
  the place.
- **R5. The label.** UI tests and VoiceOver users know the place by its name. The
  name stays the label; the layer's words are the hint. A UI test cannot read a
  hint; a rule test of the source holds the app to using the layer's words, and the
  UI test checks that the element is a button exactly when the map offers the look.
- **R6. The kept game.** A tap is already kept as `tap <text>` and replayed as given.
  No change of form.
- **R7. The cost.** One more optional part in the map's JSON, about 40 bytes; one
  string comparison at each stop. Inside the limits.
