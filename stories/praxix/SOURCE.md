# praxix: source and license record

Praxix, a Z-machine unit test, by Zarf (Andrew Plotkin) and Dannii Willis.

| | |
| --- | --- |
| Source | <https://ifarchive.org/if-archive/infocom/interpreters/tools/praxix.zip> |
| Archive SHA-256 | `c654d656fb426ae1b386e5ab9c44cc5062eade4a8c3bce69c9e61191221501ad` |
| License | Stated at the top of `praxix.inf`: "Public domain, but please share your changes so we can improve the tests". The same header notes "The indirect test is Copyright (C) 2003, Amir Karger"; that test comes from czech, whose license (see `../czech/README.txt`) permits redistribution with its notice, which is retained in the file |

## Files

| File | Origin | SHA-256 |
| --- | --- | --- |
| `praxix.inf` | From the archive, unmodified | `de7d1202048fee386c8d6feef80ba971e3ab9338566bd12f85cb50fb8f22b210` |
| `praxix.z5` | From the archive, unmodified; Release 1, serial 180329 | `bef3bdc2543cc7161833062855aa9bb1682db9eca69d9d36e9101347c21b48ac` |

## Running it

praxix reads commands. The two lines `all` and `quit` run every test with no keypresses
and no file prompts. Under dfrotz 2.55 the output ends:

```text
All tests passed.

>Goodbye.
```

When a test fails, its group prints how many failed and the run ends with a line listing
the groups that failed, in place of "All tests passed."
