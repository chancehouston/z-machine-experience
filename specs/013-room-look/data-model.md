# Data Model: Touch the Room to Look

- **Look word** (session): `Option<String>`. The story's word for looking, found
  once at opening (R1, R2). None where the dictionary has neither candidate.
- **LookView** (map view): `command: String`, `spoken: String`. Present when the
  game waits for a command, `here` is known and the session has a look word.
  `spoken` is the layer's constant words, "Look around".
- **MapView.look**: `Option<LookView>`.
- **Snapshot**: `map.look` is `null` or `{"command": "look", "spoken": "Look around"}`.
- **Refusals**: `Session::tap_place(at)` gives `Refused::Stale` for another turn, as
  for a way, and `Refused::NotOffered` where `look` is absent.
