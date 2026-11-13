# Contract: Announcements in the Layer and the App

**Plan**: [../plan.md](../plan.md) | **Research**: [../research.md](../research.md) |
**Data**: [../data-model.md](../data-model.md)

Each numbered statement becomes a test that cites it, as "N§".

## The layer

1. The reply of a turn is its text with the white space at its end removed, then one
   final `>` removed if the game is waiting for a command or an answer, then the white
   space at both ends removed. No other character of the game's text is changed.
2. A session's announcement after a turn begins with that turn's reply, if the reply is
   not empty.
3. If the layer's light says dark after the turn and did not at the stop before, the
   note of darkness follows, on a line of its own. The opening turn counts if it is
   dark.
4. The note does not follow on a turn that leaves the player still in the dark, on one
   that leaves them in the light, or where light is unknown.
5. A stop that is not a command prompt counts as not dark for statement 3.
6. If the game ended on the turn, the layer's words for an ended game follow, on a line
   of their own.
7. If the game has stopped with a fault, the announcement is the fault's message and
   nothing else.
8. With nothing to say the announcement is none.
9. A new game's opening turn has its announcement as any opening has, and the dark
   before the new game is forgotten.
10. A command the session refuses leaves the announcement as it was.
11. The player's command is in no announcement.

## The snapshot

12. The snapshot's JSON ends with the key `announcement`, whose value is the text or
    `null`. Every earlier key is where it was.
13. When the game has ended, `message` is the layer's words for an ended game; when it
    has faulted, the fault's message, as before; otherwise `null`, as before.
14. The C interface is unchanged.

## The app

15. The app shows `message` beneath the transcript when the game has ended as when it
    has faulted. Its source holds no sentence about the game's ending.
16. When a snapshot it shows brought a new turn or a new game, `announcement` is not
    null, and a screen reader is running, the app posts the announcement to the
    system, marked to interrupt speech in progress.
17. It posts exactly the field's text.
18. It posts nothing when `announcement` is null, when the snapshot brought no new
    turn, or when no screen reader is running.
19. Posting moves no focus: after it the command field has the focus it had, and the
    screen reader's place is not set by the app.
20. With the launch argument `-speakAlways` the app behaves as if a screen reader were
    running, and an element with the identifier `announcement.last` has as its label
    the text last posted and as its value the number of posts. Without the argument
    there is no such element.
21. No label, value or identifier that an earlier feature gave an element changes.

## The owner's ear

22. On a phone with the screen reader running, on Zork I: each reply is heard on the
    walk from the opening to the kitchen, up to the attic and down; "It is dark" is
    heard once, after the attic's reply; a command sent during a reply cuts it off and
    its own reply is heard; the screen reader's place and the command field are where
    they were. The owner records what was heard.

## As built

- **Statement 20.** The element's label and value are kept by `Game`, which posts
  through `Announcer`; `GameView` shows the element only with the argument.
- **Statement 16.** "A new turn or a new game" is told by the count of turns the app
  holds growing, or the snapshot's generation changing.
- **Statement 19** has no test of its own for the screen reader's place, which a test
  cannot read; the test shows that typing still reaches the command field after a
  post. The owner's ear confirmed the rest.
- **Statement 22** was checked by the owner on an iPhone 15 Pro, iOS 27.2.1, on
  2026-11-13; what they said is in `tasks.md`.
