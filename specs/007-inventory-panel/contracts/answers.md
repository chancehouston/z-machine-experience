# Contract: The Published Interface and the Pack Format

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

What `docs/introspection.md` is to say, at version 0.3.0. Each numbered statement
becomes a sentence there with a test that cites it.

## The inventory answer (changes to I§7 and I§2.3)

1. Every entry has a field `seen`: `yes`, `no` or `unknown`.
2. An entry at the top level is seen `yes`.
3. An entry inside another is seen `yes` if the one it is in is seen `yes` and lets its
   contents be seen; `no` if the one it is in is seen `no`, or is known not to let them
   be seen; and `unknown` otherwise.
4. Every entry has a field `state`: `open`, `closed` or `unknown` if the object holds
   something at the moment of the query, and none if it holds nothing.
5. Every entry has a field `source`: `pack` if `state` or `seen` would be different
   were no pack in force, and `file` otherwise.
6. `contents` lists everything the object holds, whatever its marks. The answer marks;
   it does not filter.
7. In JSON an entry's keys are, in order: `object`, `name`, `worn`, `state`, `seen`,
   `source`, `contents`.

## By family (to be I§7.1 and I§7.2)

8. Inform 6: an object that holds something has no `state` if it has attribute 20
   (`supporter`); otherwise `open` if it has attribute 14 and `closed` if not. What it
   holds can be seen if it is a supporter, is open, or has attribute 23
   (`transparent`).
9. ZIL with no pack fact: `state` is `unknown`, and what the object holds is seen
   `unknown`.
10. ZIL with `open-attribute: A` in force: `state` is `open` if the object has
    attribute A and `closed` if not.
11. ZIL with `transparent-attribute: T` in force: what an object with attribute T holds
    can be seen, whatever its `state`.

## The pack (changes to I§12)

12. `transparent-attribute: T`, a decimal number 0 to 31, at most once, says that in
    this story attribute T is set on a thing whose contents can be seen while it is
    closed.
13. The format's version is 2. A reader of version 2 reads packs of versions 1 and 2. A
    pack that holds `transparent-attribute` states `pack: 2`.
14. A reader refuses a pack of a version it does not know, as before; so a reader of
    version 1 given a pack of version 2 behaves as with no pack.
15. `open-attribute` now also decides `state` (statement 10). Its effect on exits is
    unchanged.
16. A pack changes the answers to `exits` and `inventory`, and to no other query.

## Worked example, with no game's text

```text
[{"object":40,"name":"…","worn":"unknown","state":"closed","seen":"yes","source":"pack","contents":[
   {"object":41,"name":"…","worn":"unknown","state":null,"seen":"yes","source":"pack","contents":[]}]},
 {"object":50,"name":"…","worn":"unknown","state":"closed","seen":"yes","source":"pack","contents":[
   {"object":51,"name":"…","worn":"unknown","state":null,"seen":"no","source":"pack","contents":[]}]}]
```

The first is closed and see-through; the second is closed and not.

## What changed from 0.2.0

- An inventory entry has three more fields: `state`, `seen`, `source`.
- The pack format has a version 2, with one more key.
- A pack may now change the inventory answer.
