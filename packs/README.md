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

On Zork I the attribute that means lit is singled out: no other of the 32 passes. The
attribute for a hidden thing is not: 7 passes and so does 30. Attribute 30 is set on the
character who takes the player's light and never changes; the game sets 7 on the taken
thing itself and clears it again, so the pack names 7. The corpus run cannot tell the two
apart, and a test records that exactly those two pass.
