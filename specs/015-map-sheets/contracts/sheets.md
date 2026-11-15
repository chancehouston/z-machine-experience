# Contract: The Other Sheets of the Map

Parts: the layer (L), the snapshot (S), the screen (A). Each clause is a test.

## L. The layer

- **L§1** `Map::sheet(number)` gives the view of that sheet, from 1 to the count of
  sheets, and none for any other number. For the player's sheet it equals `view()`.
- **L§2** The places of all the sheets together are every place of the map, each
  once.
- **L§3** A sheet that is not the player's has no `here`, no ways, no look, and
  `own` false; its stubs are the unwalked ways of every place on it.
- **L§4** `label` is "Sheet N of M · P", P being the name of the first place found
  on it that has one; or "Sheet N of M" where none has. `spoken` for a sheet not the
  player's is "Map. Sheet N of M, P. You are not on this sheet." (without ", P"
  where none has a name).
- **L§5** `before` and `after` are the sheets before and after in the order found,
  going round; none where there is one sheet. `back` is the player's sheet where
  this is not it.
- **L§6** Each lead has `place`, the id of the place it leads to, which is on the
  sheet it names.
- **L§7** `Session::sheet(at, number)` is refused as stale for another turn and as
  not offered for a number with no sheet. It changes nothing: the kept game, the
  snapshot and the map's page are as they were.

## S. The snapshot

- **S§1** `map` gains `own`, `label`, `first`, `before`, `after`, `back` after
  `look`; a lead gains `place`. A sheet given on request has the same form.

## A. The screen

- **A§1** The band shows the player's own sheet, whatever sheet the opened-out map
  shows.
- **A§2** Opened out, the map shows `label` (`map.sheet.label`), and where there is
  more than one sheet the controls `button.sheet.before` and `button.sheet.after`,
  whose spoken words are the labels of the sheets they turn to.
- **A§3** Opened out, each lead is a button `map.lead.<n>` at least 44 points
  square; a touch shows the sheet it leads to with the place it leads to in the
  middle. In the band a lead is not a button.
- **A§4** On a sheet not the player's there is no `map.lamp`, no `compass.*` and no
  touchable place; there is `button.sheet.return`, which shows the player's sheet
  with their place in the middle. On their own sheet it is absent.
- **A§5** Closing the map, and any new turn, returns to the player's sheet.
- **A§6** Turning a sheet adds no turn.
- **A§7** The app writes no word for any of this, and only `Engine.swift` calls
  `zmx_session_sheet`.
