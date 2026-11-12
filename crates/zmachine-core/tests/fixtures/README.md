# Fixtures recorded from the reference interpreter

Everything here is output of dfrotz from Frotz 2.55 (`dfrotz -v`: git commit
`acf205585a9472d27c07c0fe62da4b8bc89d1ec7`), running small programs built by
`zmachine-testkit`. None of it is text from a published game.

## `random/seed_<n>.txt`

The transcript of `programs::random_probe()` at seed `n`: 200 results of `random 100`,
then 20 after each of `random -5`, `random -1000` and `random 0`.

```text
cargo run -p zmachine-testkit --example random_probe -- probe.z3
printf 'out\n' | dfrotz -m -q -S 0 -s <n> probe.z3        # writes out.scr
```

## `transcript/<name>.scr`

The transcript of each program in `programs::transcript_probes()`.

```text
cargo run -p zmachine-testkit --example transcript_probes -- probes
cd probes
dfrotz -m -q -S 0 -s 1 <name>.z3 < <name>.in              # writes <name>.scr
```

The ignored test `probes_match_dfrotz` in `crates/zmachine-cli/tests/script.rs` reruns
dfrotz and checks that it still produces these files.

## `transcript5/<name>.scr`

The transcript of each program in `programs::transcript5_probes()`, the version 5 probes
for rules 13 to 21 of `specs/002-v5-v8/contracts/transcript-format.md`.

```text
cargo run -p zmachine-testkit --example transcript5_probes -- probes
cd probes
dfrotz -m -q -w 80 -S 0 -s 1 <name>.z5 < <name>.in        # writes <name>.scr
```

Four probes start transcription twice. Frotz asks a version 5 game for the file name
each time and appends to the file, so their `.in` files give the name twice; this
interpreter asks once (§7.1.1.2).

## `keys/<name>.in`, `keys/<name>.scr`

Scripts for `programs::key_probe()`, which reads twelve keys, a line, and three more
keys, printing each code, and the transcripts dfrotz wrote. They record how dfrotz takes
keys from a script (`specs/002-v5-v8/contracts/cli.md`, "Keypresses").

```text
cargo run -p zmachine-testkit --example key_probe -- keys.z5
dfrotz -m -q -w 80 -S 0 -s 1 keys.z5 < <name>.in          # writes <name>.scr
```

There is no fixture for the arrow and function-key escapes: dfrotz 2.55 documents them
but gives the game a question mark for each run of them.

## `screen/<name>.txt`

What dfrotz printed for the three programs in `programs::screen_programs()` when asked to
show its whole 80 by 24 screen, rows numbered, each time the program waits for input.
`tests/screen.rs` compares the rows of the upper window with this interpreter's canvas.

```text
cargo run -p zmachine-testkit --example screen_programs -- screens
cd screens
dfrotz -m -r cn -r ln -r lt -h 24 -w 80 <name>.z5 < <name>.in > <name>.txt
```

The ignored tests `version_5_probes_match_dfrotz` in `crates/zmachine-cli/tests/script.rs`
and `dfrotz_still_writes_the_key_fixtures` in `crates/zmachine-cli/tests/play5.rs` rerun
dfrotz and check that it still produces these files.
