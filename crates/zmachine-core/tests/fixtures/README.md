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
