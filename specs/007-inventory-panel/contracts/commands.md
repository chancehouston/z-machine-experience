# Contract: Programs, Functions, Files and the Screen

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

## The introspection crate

`Carried` gains `state: Option<ContainerState>` (`Open`, `Closed`, `Unknown`), `seen:
Tri` and `source: Source`.
`Pack` gains `transparent_attribute: Option<u8>` and `version: u8`. No function is added
or changed in what it takes.

## The experience layer

`Session::inventory(&self) -> &Strip`. The snapshot's JSON gains, after `compass`:

```text
"inventory":{"known":true,"note":null,"spoken":"…","items":[
  {"name":"…","state":"closed","contents":[{"name":"…","state":"plain","contents":[]}]},
  {"name":"…","state":"plain","contents":[]}]}
```

`note` is `"Carrying nothing"` when the inventory is known and empty, `"Not known: type
to ask"` when it is not known, and `null` otherwise. `spoken` is, for example,
"Carrying: (name), closed, holding (name); (name)". The C interface is unchanged: the
snapshot is its one way out.

## The app

A row between the transcript and the compass, shown whenever the compass is. Each item
is a chip with the layer's name. A container's shown contents follow it as one group. A
closed container carries the outlined-panel mark. The row scrolls sideways; where it
runs past the edge a fade shows there is more. The note, when there is one, is shown in
place of chips. The whole row is one accessibility element whose label is the layer's
`spoken`; each chip has the identifier `inventory.N` for tests. Nothing in the row is a
button.

## The corpus program

| Command | Change |
| --- | --- |
| `corpus check` | Checks `inventory` and `aside` in the manifest, and ledger entries of the second form; prints those entries whose story has a pack |
| `corpus run` | For a story with `inventory`: compares at every place of its walk and of its asides; prints `ID inventory wrong ROOM OBJECT seen|kept listed|unlisted` for each unexplained difference and fails; `transparent-attribute` is proven or not as research R8 says |
| `corpus explain ID inventory ROOM` | Prints, to the terminal only, the game's listing at each place in that room and what the interface marked |

## The corpus's files

**Manifest**: keys `inventory` and `aside` (data-model.md).

**Ledger**: the second form of entry (data-model.md).

**Report**, only for a story with `inventory`, after the pack's lines (the figures here
show the form and are not measured):

```text
inventory compared 4230 agreed 4230 differed-known 0 differed-unexplained 0 no-listing 2 empty 61 unknown 6
```

`compared`, `agreed` and the two `differed` count objects; `no-listing`, `empty` and
`unknown` count places. Detail lines for differences: `inventory ROOM OBJECT seen|kept
listed|unlisted COUNT`, after the exit detail.

## Tests that change because a form changes (SC-011)

Expected: the introspection crate's tests of the inventory JSON (three more keys); the
experience layer's and the C interface's tests that fix a whole snapshot (the
`inventory` key); the app's hand-made snapshots in its UI tests; the corpus's format
tests and Zork I's two kept reports; the pack tests that say version 2 is refused.
