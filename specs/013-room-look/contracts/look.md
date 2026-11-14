# Contract: The Look

Three parts: the layer (L), the snapshot (S), the screen (A). Each clause is a test.

## L. The layer

- **L§1** The word for looking is the first of `look`, `l` that the story's
  dictionary has, each cut to the length the dictionary keeps. With neither, there
  is no word.
- **L§2** The map's view has a look when, and only when: the game waits for a
  command; the player's place is known; there is a word.
- **L§3** `Session::tap_place(at)` sends the look's command as a tapped line. It is
  refused as stale for a stamp that is not the current turn's, and as not offered
  where the view has no look.
- **L§4** A look adds no place, line, stub or refusal to the map, and moves nothing.
- **L§5** A kept game holding such taps comes back the same, with the same map.
- **L§6** In the dark, and in a place with no name, the look is offered as anywhere.

## S. The snapshot

- **S§1** `map.look` follows `map.ways`. It is `null`, or an object with `command`
  and `spoken`.
- **S§2** `spoken` is "Look around". Nothing else in the snapshot changes.

## A. The screen

- **A§1** With `map.look` present, the player's place (`map.here`) is a button: a
  touch on it calls the layer's entry for the place (`zmx_session_tap_place`) and
  nothing else. Its label is the place's name; its hint is `spoken`.
- **A§2** With `map.look` absent, or while a command is being worked out, a touch on
  it sends nothing, and it is not a button.
- **A§3** A touch on any other place sends nothing.
- **A§4** The touch area is at least 44 points tall, and no way's control overlaps
  it.
- **A§5** A drag beginning on the player's place moves the opened sheet, and pulls
  the band down; no look is sent.
- **A§6** The app's source holds no word for looking and writes no text for it.
