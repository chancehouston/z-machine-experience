# Quickstart: Touch the Room to Look

```sh
cargo test --workspace
scripts/build-ios.sh
scripts/test-ios.sh 27 && scripts/test-ios.sh 26
```

Expect the new tests: the word found in a dictionary; the look in the map's view and
the snapshot; `tap_place` sent, refused when stale or not offered; Zork I and
Adventure described in full after one touch; a kept game with such taps brought back
the same; every story of the corpus with the word or with no touch. In the app:
touching the place looks; it is not a button at a question; a drag from the place
still drags.

By hand, on the owner's phone (SC-005): walk into a room twice so that the game
names it in a line, touch the room, and read the description. Try it on the opened
sheet and in the dark. Is this the easy way that was asked for?
