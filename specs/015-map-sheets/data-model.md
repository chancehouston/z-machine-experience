# Data Model: The Other Sheets of the Map

- **MapView** gains: `own: bool` (the player is on this sheet); `label: String`;
  `first: Option<u16>` (the first place found on it); `before` and `after`:
  `Option<SheetRef>` (none with one sheet); `back: Option<SheetRef>` (the player's
  sheet, where this is not it and their place is known).
- **SheetRef**: `number: usize`, `label: String`.
- **LeadView** gains `place: u16`: the far place's id.
- For a sheet that is not the player's: `here` none, `lamp` unknown, `ways` empty,
  `look` none, `note` none; `stubs` for every place of the sheet.
