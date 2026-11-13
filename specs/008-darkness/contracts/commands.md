# Contract: Programs, Functions, Files and the Screen

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

## The introspection crate

`Location` and `Scope` gain `source: Source`. `Exit` gains `dark_ahead: Tri`. `Pack`
gains `lit_attribute: Option<u8>`. No function is added.

## The experience layer

The snapshot's JSON gains, after `inventory`:

```text
"light":{"dark":true,"note":"It is dark"}
```

`dark` is `true`, `false` or `null`; `note` is `"It is dark"` or `null`. Each compass
place and extra gains `"dark_ahead":true|false` after `leads_to`, and its `spoken` ends
", into darkness" when true. Nothing else of the compass changes. The C interface is
unchanged.

## The app

When `light.note` is there, it is shown on a line of its own above the compass, with
the identifier `light.note`, and read by a screen reader. A place that is dark ahead
is drawn as its state is, with a small dark badge at its corner; its accessibility
label is the layer's `spoken`. Touching it does what touching that place did.

## The corpus

**Manifest**: one new key, `look`.

**Ledger**: the third and fourth forms (data-model.md).

**Report**, only for a story with `look`, after the inventory's line (the figures show
the form and are not measured):

```text
light compared 380 agreed 380 differed-known 0 differed-unexplained 0 unjudged 6 dark 24 rooms-both 5 in-the-dark 40
ahead compared 900 agreed 899 differed-known 1 differed-unexplained 0 unjudged 12
```

Detail lines: `light ROOM lit|dark described|undescribed COUNT` and `ahead ROOM
DIRECTION dark|lit described|undescribed COUNT`, after the inventory's.

**Commands**: `corpus run` prints `ID light wrong ROOM lit|dark described|undescribed`
and `ID ahead wrong ROOM DIRECTION dark|lit described|undescribed` for unexplained
differences and fails; `lit-attribute` is proven or not as research R9 says.

## Tests that change because a form changes (SC-010)

Expected: the introspection crate's tests of the location, scope and exits JSON; `zm`'s
version string; the experience layer's tests that fix a whole snapshot or a whole
place; the app's hand-made snapshots; the corpus's format tests, no-text word list and
Zork I's two kept reports; the pack tests that say version 3 is refused.
