# Game packs

Each file here is a game pack: a few facts about exactly one story file, in the format
that `docs/introspection.md` defines in section 12. A pack is named by its story's id in
`stories/corpus/manifest.txt`, holds no text of any game, and applies only to the story
file whose SHA-256 digest, release and serial it states.

Every entry of every pack is proven against its story by trials (`corpus run`); an entry
that a trial contradicts fails the build.
