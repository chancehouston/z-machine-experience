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
  the examining word and the item's word (research R2), trimmed as a spoken reply is; and
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
  `null`), `spoken`, `actions` (each `label`, `spoken`, `command`).
- **S§3** Nothing else in the snapshot changes.

## A. The screen

- **A§1** An item with `card` true is a button `item.<index>`; one without is as it
  was.
- **A§2** A touch on it shows the card (`card`) over the map's band with the name
  (`card.name`), the description or the note (`card.text`), a button for each
  action (`card.action.<n>`) and a close control (`card.close`). The turn count is
  unchanged. A card with a description holds no other words of the layer's (the
  owner's ruling, 2026-11-14: the first build's note that it was a look ahead was
  unclear on a phone).
- **A§2a** The close control is at the card's bottom left and the actions follow it
  to its right in one row, with at least 28 points of clear space between the
  close control and the first action. A description taller than its room scrolls, and has
  the mark `card.more` while there is text below what is shown. Scrolling does
  not close the card. (The owner's trial, 2026-11-14.)
- **A§3** The card goes on its close control; on a touch anywhere on the card that
  is not an action; on a touch on the band beside it; on a touch on its own item
  again; and at any new turn. None of these sends anything. The card is set in
  from the band's edges and has a drawn edge of its own; an action is small to
  look at and as tall to touch as any control. (Changed twice in building. The
  first build filled the band and closed only on its control. On their phone the
  owner could not see the card's edge, and dropped an item by mistake reaching to
  close it: "it's going to be the default action most of the time".)
- **A§4** An action's button sends it by `zmx_session_tap_action`; the command is
  in the transcript as a tapped command and the card goes.
- **A§5** With the card open the keyboard, the transcript and the command field
  are where they were.
- **A§6** When the card opens its `spoken` is posted to the screen reader.
- **A§7** The app's source holds no command word and writes no text for the card.

# Stage 2: what an item is for

## I. Introspection

- **I§4** Each carried thing has `uses`, each part `yes`, `no` or `unknown`
  (data-model). In an Inform story they are the library's flags (research R10);
  `read` and `drink` are `unknown`.
- **I§5** In a ZIL story a part is `unknown` unless a pack names its attribute
  (R12). `holds` is `yes` for a thing with the holds attribute and without the
  surface attribute; `switch` is `yes` for a thing with the switch attribute and
  without the flame attribute; `open` and `on` are the pack's open and lit
  attributes.
- **I§6** A pack of version 5 may hold the keys of R12, each once.

## L. The layer

- **L§10** A card for an item not inside another offers, in this order and only
  where every word of the command is in the story's dictionary:
  Open (`holds` yes, `open` no) or Close (`holds` yes, `open` yes);
  Turn on (`switch` yes, `on` no) or Turn off (`switch` yes, `on` yes);
  Read (`read` yes); Wear (`wear` yes, `worn` no) or Take off (`worn` yes);
  Drop; Eat (`eat` yes); Drink (`drink` yes).
- **L§11** Where a part is `unknown` nothing is offered for it. Where `open` or
  `on` is `unknown`, neither of its pair is offered.
- **L§12** An action names the item and nothing else, and no action is tried on a
  copy to decide whether to offer it.
- **L§13** An item's kind is the first that applies of lamp, container, page, food,
  clothing, else plain (R16); a lamp's `lit` is its `on`, or for an Inform lamp
  true.
- **L§14** A card for an item inside another has no action (stage 1, L§7).

## S. The snapshot and the card

- **S§4** Each of `inventory.items` and each card has `kind` and `lit`.

## A. The screen

- **A§8** The actions are in one row after the close control and its clear space,
  in the order given, and the row scrolls sideways where it is longer than the
  card.
- **A§9** An item's token is drawn from its `kind`, `state` and `lit`, on its card
  and on its chip in the strip (`item.<index>` has the kind as its value).
- **A§10** The app draws no kind and writes no word the layer did not give.

## P. Proof

- **P§1** Along Zork I's walkthrough with its pack, and Adventure's, every action
  offered does what it should when tried on a copy (R13), or its cause is in the
  ledger. The counts are recorded.
