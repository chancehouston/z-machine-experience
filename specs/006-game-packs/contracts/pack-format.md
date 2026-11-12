# Contract: The Pack Format and the Interface's Answers

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md)

This is what `docs/introspection.md` is to say, as section 12 and as changes to
sections 2.3, 2.4, 8, 8.2 and 10.4. The published document is the contract; this file
is the draft the tasks write it from, and each numbered statement becomes a statement
there with a test that cites it.

## The file (to be I§12.1)

1. A pack is text in UTF-8. Lines end in a line feed; a carriage return before it is
   ignored.
2. A line that is empty, or holds only spaces, or starts with `#`, is skipped.
3. Every other line is a key, a colon, one space, and a value, with nothing after the
   value.
4. The first four such lines are, in this order: `pack`, `sha256`, `release`, `serial`.
5. The lines after them are entries, in any order.
6. A reader that finds anything else refuses the whole pack: an unknown key, a value
   out of range, a key repeated that may not be, a line it cannot read, a missing
   identity line.

```text
pack: 1
sha256: 0000000000000000000000000000000000000000000000000000000000000000
release: 1
serial: 000000
open-attribute: 7
untrusted-variable: 40
```

| Key | Value | Times |
| --- | --- | --- |
| `pack` | `1` | once |
| `sha256` | 64 of `0-9a-f` | once |
| `release` | decimal, 0 to 65535 | once |
| `serial` | six characters, each a letter or a digit | once |
| `open-attribute` | decimal, 0 to 31 | at most once |
| `untrusted-variable` | decimal, 16 to 255 | any; no value twice |

A decimal number has no sign and no leading zero, other than `0` itself.

## Identity (to be I§12.2)

7. A story file's identity is the SHA-256 of the file as shipped, the word at byte 2 of
   its header, and the six bytes at 0x12, read as characters.
8. A pack is in force for a story exactly when all three equal the pack's.
9. A reader given a pack reports one of `none`, `in-force`, `not-matching`, `refused`.
   A host that writes one line for each stop (I§2.4) writes this word in a field
   `pack`, after `family`.
10. Only a pack in force changes any answer. In every other state every answer is what
    this specification defines without a pack.
11. A pack with an entry is refused for a story whose family is not `zil`.
12. A pack in force stays in force through restore, restart and undo.

## Entries (to be I§12.3 and I§12.4)

**`open-attribute: A`** (changes I§8.2, the 5-byte exit):

13. A 5-byte exit's first byte is a room and its second a door.
14. If the second byte is not the number of an object, or the first is not the number
    of a room, the direction is `unknown`, `not-determinable`.
15. Otherwise the exit's `kind` is `door`, its `destination` the room, its `door` the
    object, and its `state` is `open` if the door has attribute A now and `closed` if
    not.
16. A closed door is in the `exits` list. It is not "not an exit".

**`untrusted-variable: N`** (changes I§8.2, the 4-byte exit):

17. A 4-byte exit whose second byte is N is `conditional`, whatever variable N holds.
18. A 4-byte exit whose second byte is any other number is read as without a pack.

## The mark (to be I§12.5, and changes to I§8 and I§2.3)

19. Every exit has a field `source`: `pack` if the entry would be different in any
    field with no pack in force, and `file` otherwise.
20. So: every `door` exit is `pack`; a `conditional` exit under statement 17 is `pack`;
    every other exit is `file`; and with no pack in force every exit is `file`.
21. Every exit has fields `door` and `state`, which are none unless `kind` is `door`.
22. In JSON an exit's keys are, in order: `direction`, `words`, `kind`, `destination`,
    `destination_name`, `door`, `state`, `source`.
23. Entries of the `unknown` list are not marked.

Examples, with no game's text:

```text
{"direction":22,"words":["e","east"],"kind":"door","destination":85,"destination_name":"…","door":243,"state":"closed","source":"pack"}
{"direction":20,"words":["n","north"],"kind":"conditional","destination":null,"destination_name":null,"door":null,"state":null,"source":"pack"}
{"direction":29,"words":["s","south"],"kind":"fixed","destination":12,"destination_name":"…","door":null,"state":null,"source":"file"}
```

## What a pack may not do (to be I§12.6)

24. A pack changes no query but `exits`.
25. An answer with a pack in force is still a read of the story's state at the moment
    of the query (I§1.1, I§1.2, I§1.7 hold unchanged).
26. "Open" is the door's stored state. The game has the last word (I§8.3): it may still
    refuse an open door.

## What changed from 0.1.0 (to be I§12.7)

- Version 0.2.0.
- An exit has three more fields: `door`, `state`, `source`.
- `kind` has a third word: `door`.
- A host's line for each stop has one more field: `pack`.
- Section 8.2's last paragraph on the 5-byte exit points to section 12.
- Section 10.4 says that what differs between games of a family is what packs are for.
