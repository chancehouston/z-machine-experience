# Quickstart: Checking Resume

What to run to see that the feature works. The contracts are in
`contracts/kept-game.md`; the entities in `data-model.md`.

## 1. The layer

```bash
cargo test -p zmachine-experience
```

Expect, among the rest:

- a session kept and brought back after every turn of Zork I's walkthrough gives the
  same replies and snapshots as one never closed (SC-002);
- the same on a built story that decides something by chance (FR-028);
- every damaged kept game gives a new game and the notice, and none panics (SC-006).

## 2. The timings

```bash
cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture
```

Expect printed, and within their limits: bringing back 331 turns and 2,000 turns
(FR-020); making the kept bytes at the first turn and the 2,000th (FR-021); their size
(FR-022); and every command of the walkthrough still within the earlier limit
(SC-005).

## 3. The app, in the simulator

```bash
scripts/build-ios.sh
```

```bash
scripts/test-ios.sh 27
```

```bash
scripts/test-ios.sh 26
```

Expect every earlier test to pass, and the new ones: the game comes back after the
app is ended abruptly; starting over asks, and is not undone by a relaunch; an ended
game comes back ended; a damaged kept file gives a new game and the notice once.

## 4. By hand, on the owner's phone (SC-008)

1. Install and open the app. Play a few turns.
2. Close it from the app switcher. Open it. The turns are there; type a command.
3. Restart the phone. Open the app. The turns are there.
4. Open About, choose Start over, decline. Nothing has changed.
5. Choose Start over and confirm. The opening text is alone on the screen.
6. Close and open the app. It is the new game.
7. With VoiceOver on, close and open the app. The newest reply is spoken once.

Record what was seen in `tasks.md`.
