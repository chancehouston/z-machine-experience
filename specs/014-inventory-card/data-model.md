# Data Model: The Inventory Card (stage 1)

- **Carried.words** (introspection): `Part<Vec<String>>`. The dictionary words that
  name the thing, in the order its property lists them; unknown where the story's
  naming property is not known.
- **Verbs** (session, found once): `look`, `examine`, `drop`: each `Option<String>`.
- **StripItem** gains `index: usize` (its place among shown items, R5) and
  `card: bool` (can be tapped now).
- **Card**: `name: String`, `state: ItemState`, `description: Option<String>`,
  `note: Option<&'static str>` (in place of a description),
  `spoken: String` (what a screen reader is given when it opens),
  `actions: Vec<Action>`.
- **Action**: `label: String`, `spoken: String`, `command: String`.
- **Refusals**: `Stale` for another turn; `NotOffered` for an index with no card or
  an action that is not on it.

## Stage 2

- **Carried.uses** (introspection): each `yes`, `no` or `unknown`: `holds` (a
  container that opens and closes), `open`, `switch` (turned on and off), `on`,
  `read`, `eat`, `drink`, `wear`; and `light` (giving light now). `worn` is as
  before. In a ZIL story each is unknown unless the pack names its attribute.
- **Kind** (layer): `plain`, `lamp`, `container`, `page`, `food`, `clothing`.
- **StripItem** and **Card** gain `kind` and `lit` (`true`, `false` or none).
- **Card.actions** in order: Open or Close; Turn on or Turn off; Read; Wear or Take
  off; Drop; Eat; Drink.
