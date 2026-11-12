# Contract: The Screen Model

**Plan**: [../plan.md](../plan.md) | Section numbers: Z-Machine Standard 1.1. §8.7 is the
screen model for versions 4 and 5; §8.6 is version 3's and does not apply here.

Version 5 and 8 games see a screen of two windows. The core models what the game does to
them and hands the result to the host; it never draws anything itself. Where the Standard
leaves something open, this contract says what was chosen and marks it **(choice)**.

## Lower window (window 0)

- Receives the game's ordinary text, delivered to the host as styled runs in the order
  printed. The core does not wrap it (§7.2 concerns buffering; layout is the host's).
- Is the only window whose text is transcribed **(choice, following Frotz)**.
- `erase_window 0` and `erase_window -1` add a **clear marker** to the output: a run with
  no text and a flag saying the lower window was cleared at this point. In plain
  `take_output` text the marker is absent. Nothing is written to the transcript.

## Upper window (window 1)

The upper window is the top `height` rows of a canvas `rows` tall and `width` wide, the
screen size from the host.

- `split_window n` sets `height` to `n`, or to `rows` if `n` is larger (§8.7.2.1). Cells
  keep their contents (§15 `split_window`: only version 3 clears). If the cursor is now
  below the window it moves to the top left (§8.7.2.1.1). `split_window 0` unsplits.
- `set_window 1` selects the upper window and moves its cursor to the top left;
  `set_window 0` selects the lower window (§8.7.2).
- `set_cursor row column` moves the cursor, counting from 1 (§8.7.2.3). It has no effect
  while the lower window is selected. A row or column outside the canvas is brought to
  the nearest one **(choice)**. A row below `height` is allowed: see Overlay.
- A character printed while the upper window is selected is written at the cursor with
  the current style, and the cursor moves one column right. Printing in the last column
  is legal (§8.7.3.1) and leaves the cursor past the edge, where further characters on
  that row are discarded **(choice, following Frotz)**. A new-line moves to the start of
  the next row; on the last row of the canvas the cursor stays where it is **(choice)**.
- `get_cursor` reports the upper window's cursor when it is selected. For the lower
  window it reports row and column 1, the core having no layout to consult **(choice;
  Frotz reports its terminal's position)**.
- `erase_window 1` empties the top `height` rows and moves the cursor to the top left
  (§8.7.3.2.1). `erase_window -1` unsplits, empties the whole canvas and selects the
  lower window (§8.7.3.3). `erase_window -2` empties the whole canvas without unsplitting
  or changing the selected window (§15). `erase_line 1` empties from the cursor to the
  end of its row (§8.7.3.4); any other value does nothing.
- Upper-window text is never buffered, wrapped or transcribed, and is always fixed pitch
  (§8.7.2.4).

## Overlay

A version 5 game can draw on rows below its upper window. Inform's boxed quotations do:
the game enlarges the window, draws the box, and shrinks the window to one row again, all
before asking for input. On a real terminal the box stays on screen until text scrolls it
away.

- Cells in rows at or below `height` that hold something are the **overlay**.
- The overlay is readable, with the rest of the canvas, whenever the host has control.
- When the host answers a request for a line or a key, the overlay is emptied. Rows inside
  the upper window are not affected.

A host can therefore show a quotation box when it appears and remove it at the next
input, which is what a player sees in a terminal interpreter.

## Styles

- `set_text_style` with 0 clears all styles; with 1, 2, 4 or 8 adds reverse, bold, italic
  or fixed pitch to those in force (§15, as revised in Standard 1.1). There is one
  current style for all text, in whichever window **(choice, following Frotz)**.
- Fixed pitch is also in force whenever Flags 2 bit 1 is set (§8.1).
- `set_colour` and `set_true_colour` are accepted with any operands, including colour
  numbers this version does not define, and have no effect. The header tells the game
  colour is unavailable.
- `set_font` stores the number of the font that was current, or 0 if the font asked for
  is unavailable (§15). Only font 1, the normal font, is available. So `set_font 1`
  stores 1; `set_font 0` stores 1 and changes nothing; any other number stores 0. Fixed
  pitch is reached through `set_text_style`, as Frotz's dumb interface has it.
- `buffer_mode` changes nothing the host sees on screen. It changes how text reaches the
  transcript ([transcript-format.md](transcript-format.md)).

## What the host is told

- `take_screen_changed()` is true once after anything alters the canvas, `height` or the
  cursor.
- The canvas always has exactly `rows` rows of exactly `width` cells. An empty cell reads
  as a space with no styles, and can be told apart from a space the game printed.

## Test programs for this contract (SC-010)

Each is a small program built by the testkit, with the expected canvas written out cell
for cell:

1. A status line drawn the way the Inform library draws one: split to 1, reverse video,
   location on the left, score and turns on the right, back to the lower window.
2. A boxed quotation drawn the way the Inform veneer draws one: enlarge, draw, shrink to
   1. Read at the next input, then again after it.
3. A full-screen menu: `erase_window -1`, split, draw, wait for a key.
4. Enlarging and shrinking the window keeps contents.
5. Each `erase_window` operand and `erase_line`.
6. Printing in the last column, past it, and a new-line on the last row.
7. `set_cursor` to row 0, column 0, beyond the screen, and before any split.
8. Every style, alone and combined, in both windows, and with Flags 2 bit 1 set.
9. `split_window` with a height larger than the screen.
10. `get_cursor` in each window; `set_font` with 0, 1, 3 and 4.

## Cross-check against the reference interpreter (FR-033)

Run only where dfrotz is installed. `dfrotz -r cn -r ln -r lt -h 24 -w 80` prints its
whole screen with numbered rows before each input. The test runs programs 1 to 3 and the
first commands of Advent under it, takes the rows dfrotz shows for the upper window, and
compares their text with our canvas. Rows where this contract made a different choice
from Frotz (marked above) are not compared. Styles are not compared.
