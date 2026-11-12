# Divergence ledger

This is the record of every place where this interpreter intentionally differs from the
reference interpreter (Frotz 2.55) or from the czech test program, as required by
constitution Principle I. The Z-Machine Standard 1.1 defines correct behavior; an entry
here must cite the section of the Standard that justifies it.

The fidelity tests read this file. They apply exactly the rules below and nothing else,
and they fail if a rule matches nothing.

## Entry format

Each entry is a third-level heading with an identifier, followed by a description, the
Standard section, and one fenced rule block.

A transcript rule excuses one difference in a transcript comparison:

````text
### D-001 Short title

What differs and why, in plain words.

Standard: §x.y

```rule
kind: transcript
story: <SHA-256 of the story file>
after-command: <index of the command, counting from 1, after which the difference appears>
reference: <the exact text in the reference transcript>
ours: <the exact text in our transcript>
```
````

A czech rule excuses one czech test whose expectation conflicts with Standard 1.1:

````text
```rule
kind: czech
test: <the test's number or name as czech prints it>
```
````

In `reference` and `ours`, `\n` stands for a line feed and `\\` for a backslash.

## Entries

None.
