# Data Model: Untangling the Map

- **MapView.redraw**: `Option<&'static str>`: the layer's words for the control,
  when a redraw can be asked for.
- **Session.redraws**: `Vec<usize>`: for each redraw made, the count of inputs
  answered before it.
- **Kept game, form 2**: as form 1, with lines `redraw` among the input lines.
  `inputs N` still counts inputs only.
- **Refusals**: `Session::redraw_map(at)` is `Stale` for another turn and
  `NotOffered` where the view offers no redraw.
