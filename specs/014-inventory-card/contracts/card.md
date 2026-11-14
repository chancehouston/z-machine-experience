# Contract: The Inventory Card (stage 1)

Parts: introspection (I), the layer (L), the snapshot and the card (S), the screen
(A). Each clause is a test.

## I. Introspection

- **I§1** Each carried thing has `words`: the dictionary words of its naming
  property, in the property's order.
- **I§2** In an Inform story the naming property is `name`. In an Infocom story it
  is found by research R1, or given by a pack's `noun-property`.
- **I§3** Where the naming property is not known, `words` is unknown with a reason.
  A thing that does not provide the property has no words.

## L. The layer

- **L§1** The words for examining (`examine`, `x`) and dropping (`drop`) are found
  as the word for looking is.
- **L§2** A shown item can be tapped when: the game waits for a command; there is a
  word for examining; the item has at least one word.
- **L§3** `Session::card(at, index)` is refused as stale for another turn and as
  not offered for an item that cannot be tapped.
- **L§4** The card's description is what a copy of the game printed in reply to
  the examining word and the item's first word, trimmed as a spoken reply is; and
  only if the copy stopped at the command prompt. Otherwise there is no
  description and the note is the layer's words.
- **L§5** After `card`, the session's kept game, snapshot, turns and map equal
  what they were before it.
- **L§6** Two cards for one item at one turn are equal.
- **L§7** A card for an item not inside another has the action Drop when there is
  a word for dropping; no other card has it.
- **L§8** `Session::tap_action(at, index, action)` sends the action's command as a
  tapped line; refused as L§3, and for an action the card does not have.
- **L§9** No copy is run but by `card`.

## S. The snapshot and the card

- **S§1** Each of `inventory.items`, at every depth, has `index` and `card`.
- **S§2** A card is JSON: `name`, `state`, `description` (or `null`), `note` (or
  `null`), `ahead`, `spoken`, `actions` (each `label`, `spoken`, `command`).
- **S§3** Nothing else in the snapshot changes.

## A. The screen

- **A§1** An item with `card` true is a button `item.<index>`; one without is as it
  was.
- **A§2** A touch on it shows the card (`card`) over the map's band with the name
  (`card.name`), the description or the note (`card.text`), the layer's words that
  it is a look ahead (`card.ahead`), a button for each action (`card.action.<n>`)
  and a close control (`card.close`). The turn count is unchanged.
- **A§3** The card goes on its close control, on a touch on the band outside it,
  and at any new turn.
- **A§4** An action's button sends it by `zmx_session_tap_action`; the command is
  in the transcript as a tapped command and the card goes.
- **A§5** With the card open the keyboard, the transcript and the command field
  are where they were.
- **A§6** When the card opens its `spoken` is posted to the screen reader.
- **A§7** The app's source holds no command word and writes no text for the card.
