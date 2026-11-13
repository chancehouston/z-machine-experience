# Quickstart: Validating Screen Reader Announcements

**Plan**: [plan.md](plan.md) | **Contracts**: [contracts/](contracts/)

Run from the repository's root. Step 4 needs Xcode and a simulator; step 5 needs a
phone and a person.

## 1. Everything in Rust

```bash
cargo test --workspace
```

## 2. What the layer gives to be spoken (User Stories 1, 2 and 4)

```bash
cargo test -p zmachine-experience announcement
```

Expected: pass. On built stories and on Zork I: a plain turn, the opening, a turn into
the dark, one that stays in it, one out of it, a question, an ending, a fault; and over
the whole of Zork I's walkthrough, each turn's text without its prompt.

## 3. The limits hold

```bash
cargo test --release -p zmachine-experience --test speed -- --ignored
```

Expected: pass, as before this feature.

## 4. What the app asks to be spoken (User Stories 1 to 3)

```bash
scripts/test-ios.sh 27
```

```bash
scripts/test-ios.sh 26
```

Expected: pass, including a test that walks into the attic and reads, from the element
a launch argument adds, the reply followed by "It is dark"; that a second turn's text
replaces the first; and that without the argument nothing is posted.

## 5. By ear (FR-017)

On a phone with VoiceOver on, play Zork I: north, east, open window, west, up, down.
Expected: each reply is heard without moving to the transcript; "It is dark" is heard
once, in the attic; a command sent while a reply is being spoken cuts it off. Record
what was heard in `tasks.md`.
