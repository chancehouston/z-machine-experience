# Data Model: An Authored Map for Zork I

- **Presentation** (layer): `sheets: BTreeMap<u16, Option<String>>`,
  `places: BTreeMap<u16, (u16, Cell)>` (place → file's sheet, position).
- **Map** gains the presentation, and for each sheet of its own which file's sheet
  it stands for, if any.
- **PlaceView.authored**: `bool`. **MapView**: `label` uses the file's name for the
  sheet where it has one.
- **Snapshot**: each of `map.places` gains `authored`.
- **Opening**: the session's constructors take `presentation: Option<&str>`.
