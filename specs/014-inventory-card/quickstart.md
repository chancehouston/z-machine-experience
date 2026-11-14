# Quickstart: The Inventory Card

```sh
cargo test --workspace
cargo test --release -p zmachine-experience --test speed -- --ignored --nocapture
scripts/build-ios.sh
scripts/test-ios.sh 27
scripts/test-ios.sh 26
```

Run the two app suites one after the other, and do not build for a phone while
either runs.

Stage 1, by hand on the owner's phone (SC-006): take the leaflet from the mailbox,
tap it in the strip, read the card, close it; tap it again and drop it from the
card. Check the moves did not change for the look and did for the drop. Is this
what was wanted?
