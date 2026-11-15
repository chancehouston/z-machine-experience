# Quickstart: Untangling the Map

```sh
cargo test --workspace
cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture --test-threads=1
scripts/build-ios.sh
scripts/test-ios.sh 27
scripts/test-ios.sh 26
```

By hand, on the owner's phone (SC-006): from a new game, wander the forest first,
then walk round the house until a room looks out of place. Press the redraw button.
Are the rooms round you where the directions say? Walk to another confusing room
and press again. Close the app and open it: is the sheet as you left it?
