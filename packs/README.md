# Game packs

Each file here is a game pack: a few facts about exactly one story file, in the format
that `docs/introspection.md` defines in section 12. A pack is named by its story's id in
`stories/corpus/manifest.txt`, holds no text of any game, and applies only to the story
file whose SHA-256 digest, release and serial it states.

Every entry of every pack is proven against its story (`corpus run`): entries about exits
by trials of walking, entries about what can be seen of the inventory by the game's own
listing. An entry that either contradicts fails the build. A pack of version 2 may also
state which attribute means see-through. A pack of version 3 may state which attribute
means lit, and which is set on a thing the game hides; both are proven by asking the game
to describe the room and seeing whether it does.

A pack of version 4 may name the property that holds the words naming an object
(`noun-property`), for a story where it cannot be told by its shape. No pack here needs
it: in every story of the corpus the property is found from the file alone.

On Zork I the attribute that means lit is singled out: no other of the 32 passes. The
attribute for a hidden thing is not: 7 passes and so does 30. Attribute 30 is set on the
character who takes the player's light and never changes; the game sets 7 on the taken
thing itself and clears it again, so the pack names 7. The corpus run cannot tell the two
apart, and a test records that exactly those two pass.

A pack of version 5 may say which attribute marks what a thing is for: one that holds
other things, a surface, something to read, to eat, to drink, to wear, a light that is
turned on and off, and a flame (section 12.13 of the specification). Zork I's pack has
seven of the eight; nothing in that game is worn.

These entries are proven one way only. Along the walkthrough each action the card
offers from them is tried on a copy of the game and must do what it says
(`crates/zmachine-corpus/tests/card.rs`): reading, eating, turning on and off, closing
and dropping did every time. Opening did not: of five things with the attribute of
things that hold others, the game opened two and its own routines refused the other
three (one is a puzzle, one answers that it is open already, one has no lid). The
attribute marks a thing that holds others, which is not always a thing that opens; no
attribute of the file says which. The cases are written in that test, and a new one
fails it. Nothing proves that no other attribute would also pass.

