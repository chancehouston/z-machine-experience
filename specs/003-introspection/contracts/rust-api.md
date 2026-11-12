# Contract: Rust Interfaces

**Plan**: [../plan.md](../plan.md)

## Addition to `zmachine-core`

```text
impl Machine {
    pub fn view(&self) -> View<'_>;
}

impl View<'_> {
    pub fn version(&self) -> u8;                 // 3, 5 or 8
    pub fn len(&self) -> usize;                  // bytes in the story file
    pub fn byte(&self, address: usize) -> Option<u8>;      // memory as it is now
    pub fn word(&self, address: usize) -> Option<u16>;
    pub fn shipped_byte(&self, address: usize) -> Option<u8>;   // the file as loaded
    pub fn shipped_word(&self, address: usize) -> Option<u16>;
}
```

- `View` borrows the machine. It has no method that writes, and the core has no interior
  mutability, so holding a `View` cannot change the game.
- An address outside the file gives `None`. Nothing panics.
- Nothing else in the core's public interface changes. Guarantee 6 of feature 002's core
  contract (version 3 unchanged) and every other guarantee stand.

## `zmachine-introspect`

```text
pub struct Introspector { /* family, and facts worked out once from the shipped file */ }

impl Introspector {
    pub fn new(view: &View<'_>) -> Introspector;
    pub fn family(&self) -> Family;
    pub fn object_tree(&self, view: &View<'_>) -> Answer<Vec<Object>>;
    pub fn dictionary(&self, view: &View<'_>) -> Answer<Vec<Word>>;
    pub fn location(&self, view: &View<'_>) -> Answer<Location>;
    pub fn inventory(&self, view: &View<'_>) -> Answer<Vec<Carried>>;
    pub fn exits(&self, view: &View<'_>) -> Answer<Exits>;
    pub fn scope(&self, view: &View<'_>) -> Answer<Scope>;
}

pub struct Answer<T> { pub value: Result<T, Unknown>, pub exactness: Exactness }
pub enum Exactness { Exact, Approximate }
pub enum Unknown { UnrecognisedFamily, NoPlayer, AmbiguousPlayer, Inconsistent,
                   NotDeterminable, Damaged }
pub enum Family { Inform6, Zil, Unrecognised }

pub const QUERIES: [&str; 7];        // the names the specification must list
pub const SPEC_VERSION: &str;        // must equal the version in docs/introspection.md

pub mod json;                        // to_json for each answer, as the specification defines
```

The record types are those of [../data-model.md](../data-model.md), with fields of the
same names.

## Guarantees

1. **No mutation.** Every function takes `&View`. A test asserts the core has no
   interior mutability.
2. **No panics** on any story file the core loads, in any state, including after a
   fault.
3. **Bounded work.** Every walk is bounded by the number of objects or the size of the
   file. All six queries on Advent take under 5 ms.
4. **No game-specific knowledge.** No source file names a game, a serial number, a
   release or a checksum; a test reads the crate's source to confirm it.
5. **An introspector is tied to one story file.** Given a view of a different file it
   answers `damaged` or `inconsistent`; it does not panic. A host makes a new one when it
   loads a new game. After a restore, restart or undo the same introspector is still
   valid, because it holds nothing that changes during play.
6. **No dependencies** beyond `zmachine-core`, no `unsafe`, no I/O.
