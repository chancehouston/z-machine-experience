//! The table instructions of version 5 (§15): copy_table, scan_table,
//! print_table, encode_text and tokenise.

use crate::decode::Instruction;
use crate::dictionary::Lexer;
use crate::exec::X;
use crate::Machine;

impl Machine {
    /// §15 copy_table. With `second` 0 the first table is zeroed. A positive size
    /// copies so that overlapping tables come out right; a negative size copies
    /// forwards byte by byte whatever the overlap.
    pub(crate) fn copy_table(&mut self, first: u16, second: u16, size: u16) -> X<()> {
        let signed = size as i16;
        let len = signed.unsigned_abs() as u32;
        let (first, second) = (first as u32, second as u32);
        if second == 0 {
            for i in 0..len {
                self.memory.write_byte(first + i, 0)?;
            }
            return Ok(());
        }
        // Copying backwards is the safe direction only when the destination starts
        // inside the source.
        let backwards = signed > 0 && second > first && second < first + len;
        for n in 0..len {
            let i = if backwards { len - 1 - n } else { n };
            let byte = self.memory.read_byte(first + i)?;
            self.memory.write_byte(second + i, byte)?;
        }
        Ok(())
    }

    /// §15 scan_table: looks for `x` among `len` fields. `form` bit 7 set means the
    /// fields are words, clear means bytes; its low seven bits are the distance
    /// from one field to the next. Returns the address of the match, or 0.
    pub(crate) fn scan_table(&mut self, x: u16, table: u16, len: u16, form: u16) -> X<u16> {
        let words = form & 0x80 != 0;
        let step = (form & 0x7F) as u32;
        let mut at = table as u32;
        for _ in 0..len {
            let value = if words { self.memory.read_word(at)? } else { self.memory.read_byte(at)? as u16 };
            if value == x {
                return Ok(at as u16);
            }
            if step == 0 {
                break;
            }
            at += step;
        }
        Ok(0)
    }

    /// §15 print_table: a rectangle of ZSCII text, `width` characters to a row. Each
    /// row after the first starts on a new line, under the start of the first.
    pub(crate) fn print_table(&mut self, table: u16, width: u16, height: u16, skip: u16) -> X<()> {
        let (start_row, start_column) = self.screen.cursor();
        let mut at = table as u32;
        for row in 0..height {
            if row > 0 {
                if self.window == 0 {
                    self.emit("\n");
                } else {
                    self.screen.set_cursor(start_row + row, start_column);
                }
            }
            for _ in 0..width {
                let code = self.memory.read_byte(at)? as u16;
                self.print_zscii(code);
                at += 1;
            }
            at += skip as u32;
        }
        Ok(())
    }

    /// §15 encode_text: encodes `length` characters starting `from` bytes into
    /// `text` as a dictionary word, written to `coded`.
    pub(crate) fn encode_text(&mut self, text: u16, length: u16, from: u16, coded: u16) -> X<()> {
        let mut word = Vec::with_capacity(length as usize);
        for i in 0..length as u32 {
            word.push(self.memory.read_byte(text as u32 + from as u32 + i)?);
        }
        let key = self.charset.encode(&word, self.header.v.dictionary_zchars());
        for (i, byte) in key.iter().enumerate() {
            self.memory.write_byte(coded as u32 + i as u32, *byte)?;
        }
        Ok(())
    }

    /// §15 tokenise: lexical analysis on request, against the main dictionary or
    /// one the game supplies.
    pub(crate) fn tokenise_instruction(&mut self, ins: &Instruction, operands: &[u16]) -> X<()> {
        let _ = ins;
        let get = |i: usize| operands.get(i).copied().unwrap_or(0);
        let dictionary = if get(2) != 0 { get(2) } else { self.header.dictionary };
        let lexer = Lexer {
            version: self.header.v,
            charset: &self.charset,
            dictionary,
            skip_unknown: get(3) != 0,
        };
        lexer.tokenise(&mut self.memory, get(0), get(1))
    }
}
