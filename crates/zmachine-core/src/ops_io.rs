//! Output, the status line, reading a line, and the instructions that reset or
//! check the game (§7, §8, §10, §15).

use crate::decode::{self, Instruction};
use crate::exec::{Frame, MAX_CALL_DEPTH, State, X};
use crate::{Event, Fault, Machine, StatusKind, StatusLine, dictionary, header, quetzal, text};

/// Flags 2, a word at 0x10 (§11.1). Bit 0: transcribing is on. Bit 1: the game
/// wants a fixed-pitch font. Both survive a restart (§6.1.3).
const FLAGS2: u32 = 0x10;
const FLAGS2_TRANSCRIPT: u16 = 0x0001;
const FLAGS2_SURVIVES_RESTART: u16 = 0x0003;

/// Flags 1 bit 1: the status line shows hours and minutes (§8.2.3.2).
const FLAGS1_TIME_GAME: u8 = 0x02;

impl Machine {
    /// Sends text to the selected output streams (§7.1). Only the lower window is
    /// the game's text (§8.5.2); what a game draws in the upper window is not.
    ///
    /// The screen gets text at once. The transcript gets it a word at a time: a
    /// word, with the space before it, is sent when it is complete, and is recorded
    /// only if transcription is on at that moment. A word is complete at the next
    /// space or new-line, after a hyphen, and whenever the game does something
    /// other than print. This is how the reference interpreter behaves
    /// (research.md R1), and it decides what is recorded when a game switches
    /// transcription on or off in the middle of a line.
    pub(crate) fn emit(&mut self, text: &str) {
        if self.screen_selected && self.window == 0 {
            self.output.push_str(text);
        }
        for ch in text.chars() {
            if ch == '\n' {
                self.flush_chunk();
                if self.transcribing() {
                    self.transcript.push('\n');
                }
                self.line_open = false;
                continue;
            }
            self.line_open = true;
            let after_hyphen = self.chunk.ends_with('-') && ch != '-';
            if ch == ' ' || after_hyphen {
                self.flush_chunk();
            }
            self.chunk.push(ch);
        }
    }

    /// Sends the word in progress to the transcript, if transcription is on.
    pub(crate) fn flush_chunk(&mut self) {
        if self.transcribing() {
            self.transcript.push_str(&self.chunk);
        }
        self.chunk.clear();
    }

    fn transcript_bit(&self) -> bool {
        self.memory
            .read_word(FLAGS2)
            .is_ok_and(|flags| flags & FLAGS2_TRANSCRIPT != 0)
    }

    /// Text printed now would be recorded: the game has transcription on, the host
    /// agreed to it, and the lower window is selected.
    fn transcribing(&self) -> bool {
        self.transcript_accepted && self.window == 0 && self.transcript_bit()
    }

    /// The game has turned transcription on and the host has not yet agreed.
    pub(crate) fn transcript_wanted(&self) -> bool {
        !self.transcript_accepted && self.transcript_bit()
    }

    pub(crate) fn answer_transcript_request(&mut self, accepted: bool) {
        if accepted {
            self.transcript_accepted = true;
        } else if let Ok(flags) = self.memory.read_word(FLAGS2) {
            // The header is always in dynamic memory, so this write cannot fail.
            let _ = self.memory.write_word(FLAGS2, flags & !FLAGS2_TRANSCRIPT);
        }
    }

    /// Ends the last line if the game quits leaving one unfinished, as the
    /// reference interpreter does on the screen and in the transcript.
    pub(crate) fn end_last_line(&mut self) {
        if self.line_open {
            self.emit("\n");
        }
        self.flush_chunk();
    }

    /// Prints the Z-string at a byte address.
    pub(crate) fn print_at(&mut self, address: u32) -> X<()> {
        let (text, _) = text::decode_string(&self.memory, address, self.header.abbreviations)?;
        self.emit(&text);
        Ok(())
    }

    /// Prints one ZSCII character (§15 print_char).
    pub(crate) fn print_zscii(&mut self, code: u16) {
        // The reference counts a print of ZSCII 0 as output for this purpose.
        self.line_open = true;
        if let Some(ch) = text::zscii_to_char(code) {
            let mut buffer = [0u8; 4];
            self.emit(ch.encode_utf8(&mut buffer));
        }
    }

    /// The short name of an object (§12.4), or nothing if it has none.
    pub(crate) fn object_name(&self, object: u16) -> X<String> {
        match self.objects().name_address(&self.memory, object)? {
            Some(address) => {
                Ok(text::decode_string(&self.memory, address, self.header.abbreviations)?.0)
            }
            None => Ok(String::new()),
        }
    }

    /// Works out the status line (§8.2) and notes whether it changed. The first
    /// global names the location; the next two are score and turns, or hours and
    /// minutes.
    pub(crate) fn update_status(&mut self) -> X<()> {
        let location = self.read_global(0)?;
        let (first, second) = (self.read_global(1)?, self.read_global(2)?);
        let location = if location == 0 {
            self.object_zero();
            String::new()
        } else {
            self.object_name(location)?
        };
        let kind = if self.memory.read_byte(1)? & FLAGS1_TIME_GAME != 0 {
            StatusKind::Time {
                hours: first,
                minutes: second,
            }
        } else {
            StatusKind::Score {
                score: first as i16,
                turns: second,
            }
        };
        let status = Some(StatusLine { location, kind });
        if status != self.status {
            self.status = status;
            self.status_changed = true;
        }
        Ok(())
    }

    fn read_global(&self, index: u32) -> X<u16> {
        Ok(self
            .memory
            .read_word(self.header.globals as u32 + 2 * index)?)
    }

    /// §15 output_stream: a positive number selects a stream, a negative one
    /// deselects it, and 0 does nothing. Streams 3 and 4 need operands and a
    /// meaning that version 3 does not have, and are ignored.
    pub(crate) fn output_stream(&mut self, stream: i16) -> X<()> {
        match stream {
            1 => self.screen_selected = true,
            -1 => self.screen_selected = false,
            2 | -2 => {
                // §7.1.1.2: stream 2 is selected by Flags 2 bit 0.
                let flags = self.memory.read_word(FLAGS2)?;
                let flags = if stream > 0 {
                    flags | FLAGS2_TRANSCRIPT
                } else {
                    flags & !FLAGS2_TRANSCRIPT
                };
                self.memory.write_word(FLAGS2, flags)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// §15 sread: redraws the status line (§8.2.1) and stops for a line of input.
    pub(crate) fn begin_read(&mut self, text: u16, parse: u16) -> X<Event> {
        self.flush_chunk();
        self.update_status()?;
        // §15 read: in versions 1 to 4, byte 0 holds the maximum number of
        // letters, plus one for the terminating zero.
        let max_len = (self.memory.read_byte(text as u32)? as usize).saturating_sub(1);
        self.state = State::AwaitingLine {
            text,
            parse,
            max_len,
        };
        Ok(Event::LineWanted { max_len })
    }

    /// Completes an sread with the player's line. A buffer the game placed outside
    /// dynamic memory is its error, reported as a fault by the next `run`.
    pub(crate) fn accept_line(&mut self, line: &str, text: u16, parse: u16, max_len: usize) {
        // contracts/transcript-format.md rules 4 and 8: the line is recorded as the
        // player gave it, whatever the game's buffer goes on to keep of it.
        if self.transcribing() {
            self.transcript.push_str(line);
            self.transcript.push('\n');
        }
        let zscii: Vec<u8> = line
            .chars()
            .take(max_len)
            .map(text::char_to_zscii)
            .collect();
        let result = dictionary::store_line(&mut self.memory, text, &zscii).and_then(|()| {
            dictionary::tokenise(&mut self.memory, self.header.dictionary, text, parse)
        });
        if let Err(kind) = result {
            self.pending_fault = Some(Fault {
                kind,
                address: self.current,
            });
        }
    }

    /// §15 verify: sums the bytes of the story file as shipped, from 0x40 to the
    /// length the header states, modulo 0x10000, and compares with the header
    /// checksum.
    pub(crate) fn verify(&self) -> bool {
        let end = self.header.file_length as usize;
        if end > self.story.len() {
            return false;
        }
        let sum = self
            .story
            .get(header::HEADER_LEN..end)
            .map_or(0u32, |bytes| {
                bytes
                    .iter()
                    .fold(0u32, |sum, &b| sum.wrapping_add(b as u32))
            });
        sum & 0xFFFF == self.header.checksum as u32
    }

    /// §6.1.3, §15 restart: back to the state at the start, except for the two
    /// bits of Flags 2 that survive.
    pub(crate) fn restart(&mut self) {
        let static_base = self.memory.static_base();
        let dynamic = self.memory.dynamic_mut();
        let flags = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        dynamic.copy_from_slice(&self.story[..static_base]);
        let original = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        let kept = original & !FLAGS2_SURVIVES_RESTART | flags & FLAGS2_SURVIVES_RESTART;
        dynamic[FLAGS2 as usize..FLAGS2 as usize + 2].copy_from_slice(&kept.to_be_bytes());
        header::apply_interpreter_fields(dynamic);

        self.frames = vec![Frame::main()];
        self.pc = self.header.initial_pc as u32;
        self.random.reseed_from_host();
        self.screen_selected = true;
        self.window = 0;
    }

    /// Records a fault met while acting on the host's answer, for the next `run`.
    fn fault_later(&mut self, result: X<()>) {
        if let Err(kind) = result {
            self.pending_fault = Some(Fault {
                kind,
                address: self.current,
            });
        }
    }

    /// §15 save: builds the save file and stops for the host to store it. In
    /// version 3 the instruction branches on the outcome, and the save records the
    /// address of that branch data so that a restore can resume there (Quetzal
    /// §5.8.1).
    pub(crate) fn begin_save(&mut self, ins: Instruction) -> Event {
        self.flush_chunk();
        let pc = ins.branch_data.unwrap_or(ins.next);
        let original = &self.story[..self.memory.static_base()];
        let bytes = quetzal::write(
            &self.header,
            original,
            self.memory.dynamic(),
            &self.frames,
            pc,
        );
        self.state = State::AwaitingSaveResult;
        self.pending = Some(ins);
        Event::SaveRequested(bytes)
    }

    pub(crate) fn finish_save(&mut self, stored: bool) {
        if let Some(ins) = self.pending.take() {
            let result = self.branch(&ins, stored);
            self.fault_later(result);
        }
    }

    /// §15 restore. With no usable save the instruction's failure branch is taken
    /// and nothing else changes (FR-017).
    pub(crate) fn finish_restore(&mut self, bytes: Option<&[u8]>) {
        let Some(ins) = self.pending.take() else {
            return;
        };
        let original = &self.story[..self.memory.static_base()];
        let saved = bytes
            .and_then(|bytes| quetzal::read(bytes, &self.header, original).ok())
            // The saved program counter must point at readable branch data, and
            // the call stack must be one this machine would itself allow.
            .filter(|saved| saved.frames.len() <= MAX_CALL_DEPTH + 1)
            .and_then(|saved| {
                let (branch, next) = decode::decode_branch(&self.memory, saved.pc).ok()?;
                Some((saved, branch, next))
            });
        let Some((saved, branch, next)) = saved else {
            let result = self.branch(&ins, false);
            self.fault_later(result);
            return;
        };

        // §6.1.2: the game state is replaced, except that Flags 2 keeps the bits it
        // has now, and the interpreter writes its own header fields again.
        let dynamic = self.memory.dynamic_mut();
        let now = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        dynamic.copy_from_slice(&saved.dynamic);
        let then = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        let kept = then & !FLAGS2_SURVIVES_RESTART | now & FLAGS2_SURVIVES_RESTART;
        dynamic[FLAGS2 as usize..FLAGS2 as usize + 2].copy_from_slice(&kept.to_be_bytes());
        header::apply_interpreter_fields(dynamic);
        self.frames = saved.frames;

        // Resume inside the `save` that made the file, reporting success: its
        // branch is evaluated with the condition true (§15 restore).
        self.current = saved.pc;
        self.pc = next;
        let resumed = Instruction {
            address: saved.pc,
            branch: Some(branch),
            branch_data: Some(saved.pc),
            ..ins
        };
        let result = self.branch(&resumed, true);
        self.fault_later(result);
    }
}
