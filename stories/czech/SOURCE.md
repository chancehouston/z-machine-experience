# czech: source and license record

CZECH, the Comprehensive Z-machine Emulation CHecker, version 0.8, by Amir Karger.

| | |
| --- | --- |
| Source | <https://ifarchive.org/if-archive/infocom/interpreters/tools/czech_0_8.zip> |
| Archive SHA-256 | `d0795723e98563050575c4d3c08fdd29ade2543e132d8190255763e3bf6fef6a` |
| License | In `README.txt` beside this file, under LICENSE: usage, distribution and modification granted to everyone for any purpose, provided the license and copyright notice are preserved. Copyright (C) 2003, Amir Karger |

## Files

| File | Origin | SHA-256 |
| --- | --- | --- |
| `czech.inf` | From the archive, unmodified | `783ca6ed532b985b62b8c28ffed5de5634b5e7c18698f671b0b09083c4de9d4b` |
| `README.txt` | From the archive, unmodified | `01219007c87a3f9bb8790a31049b258b206e74743fc31891f4fdd352e4439b6d` |
| `czech.out3` | From the archive, unmodified: the author's expected output for version 3 | `4a2c1880e5ac8fa264c6362d2fdea4b9edece35b956a758ae3e319ed6ca96c61` |
| `czech.z3` | Compiled by this project; the archive ships only a version 5 build | `1cfba855bdc17326e3327432f0e22e6721b83b69dffda6f51fd59531636f6695` |

## How `czech.z3` was built

Inform 6.44 for MacOS (11th September 2025), installed with `brew install inform6`:

```text
inform -v3 czech.inf czech.z3
```

The compiler reports one warning, that the source's `Switches` directive is deprecated.

Under dfrotz 2.55 this build reports "Passed: 349, Failed: 0, Print tests: 19", the same
totals as `czech.out3`.
