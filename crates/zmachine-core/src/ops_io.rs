//! Output, the status line, reading a line, and the instructions that reset or
//! check the game (§7, §8, §10, §15).

use crate::decode::{self, Instruction};
use crate::exec::{Frame, MAX_CALL_DEPTH, State, X};
use crate::{Event, Fault, FaultKind, Key, Machine, StatusKind, StatusLine, Style, dictionary, header, quetzal, undo};

/// Flags 2, a word at 0x10 (§11.1). Bit 0: transcribing is on. Bit 1: the game
/// wants a fixed-pitch font. Both survive a restart (§6.1.3).
const FLAGS2: u32 = 0x10;
const FLAGS2_TRANSCRIPT: u16 = 0x0001;
const FLAGS2_FIXED_PITCH: u16 = 0x0002;

/// §7.1.2.1.1: how deep memory streams may nest.
const MAX_MEMORY_STREAMS: usize = 16;
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
        // §7.1.2.2: while a memory stream is open, it alone receives text.
        if let Some((_, captured)) = self.memory_streams.last_mut() {
            for ch in text.chars() {
                captured.push(self.charset.zscii_for(ch).unwrap_or(b'?'));
            }
            return;
        }
        if self.screen_selected {
            let style = self.style_in_force();
            if self.window == 0 {
                self.output.push(text, style);
            } else if !self.header.v.is_v3() {
                self.screen.write(text, style);
            }
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
            // With buffering off the reference passes each character on at once.
            if !self.buffering {
                self.flush_chunk();
            }
        }
    }

    /// The style text is printed in: what the game set, and fixed pitch as well
    /// while Flags 2 bit 1 is set (§8.1).
    fn style_in_force(&self) -> Style {
        let forced = self
            .memory
            .read_word(FLAGS2)
            .is_ok_and(|flags| flags & FLAGS2_FIXED_PITCH != 0);
        Style { fixed_pitch: self.style.fixed_pitch || forced, ..self.style }
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
        // The line feed is for the main text; it is never drawn on the canvas.
        if self.line_open && self.window == 0 {
            self.emit("\n");
        }
        self.flush_chunk();
    }

    /// Prints the Z-string at a byte address.
    pub(crate) fn print_at(&mut self, address: u32) -> X<()> {
        let (text, _) = self.charset.decode(&self.memory, address, self.header.abbreviations)?;
        self.emit(&text);
        Ok(())
    }

    /// Prints one ZSCII character (§15 print_char).
    pub(crate) fn print_zscii(&mut self, code: u16) {
        // The reference counts a print of ZSCII 0 as output for this purpose.
        if self.memory_streams.is_empty() {
            self.line_open = true;
        }
        if let Some(ch) = self.charset.zscii_to_char(code) {
            let mut buffer = [0u8; 4];
            self.emit(ch.encode_utf8(&mut buffer));
        }
    }

    /// The short name of an object (§12.4), or nothing if it has none.
    pub(crate) fn object_name(&self, object: u16) -> X<String> {
        match self.objects().name_address(&self.memory, object)? {
            Some(address) => {
                Ok(self.charset.decode(&self.memory, address, self.header.abbreviations)?.0)
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
    /// deselects it, and 0 does nothing. Stream 3 writes into a table, from
    /// version 5 here; stream 4, a record of the player's commands, is not offered.
    pub(crate) fn output_stream(&mut self, stream: i16, table: u16) -> X<()> {
        match stream {
            3 if !self.header.v.is_v3() => {
                if self.memory_streams.len() >= MAX_MEMORY_STREAMS {
                    return Err(FaultKind::StreamNesting);
                }
                self.memory_streams.push((table, Vec::new()));
            }
            -3 => {
                // §7.1.2.1: a word holding the number of characters, then the
                // characters. Closing a stream that is not open does nothing.
                if let Some((table, captured)) = self.memory_streams.pop() {
                    let table = table as u32;
                    self.memory.write_word(table, captured.len() as u16)?;
                    for (i, &byte) in captured.iter().enumerate() {
                        self.memory.write_byte(table + 2 + i as u32, byte)?;
                    }
                }
            }
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

    /// §15 read, version 5: stops for a line of input. No status line is drawn;
    /// the buffer's first byte is its capacity and its second the number of
    /// characters already there, which the player's line follows.
    pub(crate) fn begin_aread(&mut self, ins: Instruction, text: u16, parse: u16) -> X<Event> {
        self.flush_chunk();
        let capacity = self.memory.read_byte(text as u32)? as usize;
        let present = self.memory.read_byte(text as u32 + 1)? as usize;
        self.preloaded.clear();
        for i in 0..present {
            let code = self.memory.read_byte(text as u32 + 2 + i as u32)? as u16;
            self.preloaded.extend(self.charset.zscii_to_char(code));
        }
        let max_len = capacity.saturating_sub(present);
        self.state = State::AwaitingLine { text, parse, max_len };
        self.pending = Some(ins);
        Ok(Event::LineWanted { max_len })
    }

    /// Completes a read with the player's line and the ZSCII code of the key that
    /// ended it. A buffer the game placed outside dynamic memory is its error,
    /// reported as a fault by the next `run`.
    pub(crate) fn accept_line(&mut self, line: &str, text: u16, parse: u16, max_len: usize, terminator: u16) {
        // contracts/transcript-format.md rules 4 and 8: the line is recorded as the
        // player gave it, whatever the game's buffer goes on to keep of it.
        if self.transcribing() {
            self.transcript.push_str(line);
            self.transcript.push('\n');
        }
        self.screen.clear_overlay();
        let zscii: Vec<u8> = line
            .chars()
            .take(max_len)
            .map(|ch| self.charset.input_zscii(ch))
            .collect();
        let lexer = dictionary::Lexer {
            version: self.header.v,
            charset: &self.charset,
            dictionary: self.header.dictionary,
            skip_unknown: false,
        };
        let result = if self.header.v.is_v3() {
            dictionary::store_line(&mut self.memory, text, &zscii)
                .and_then(|()| lexer.tokenise(&mut self.memory, text, parse))
        } else {
            // §15 read: from version 5 the parse buffer may be absent.
            let stored = dictionary::append_line(&mut self.memory, text, &zscii);
            let parsed = match parse {
                0 => stored,
                _ => stored.and_then(|()| lexer.tokenise(&mut self.memory, text, parse)),
            };
            self.preloaded.clear();
            match self.pending.take() {
                Some(ins) => parsed.and_then(|()| self.store(&ins, terminator)),
                None => parsed,
            }
        };
        self.fault_later(result);
    }

    /// Completes a read_char with the key's ZSCII code. The key is not echoed.
    pub(crate) fn finish_read_char(&mut self, code: u16) {
        self.screen.clear_overlay();
        if let Some(ins) = self.pending.take() {
            let result = self.store(&ins, code);
            self.fault_later(result);
        }
    }

    /// The ZSCII code a key is read as (§3.8), if it has one.
    pub(crate) fn key_code(&self, key: Key) -> Option<u16> {
        match key {
            Key::Char(ch) if ch != '\n' => self.charset.zscii_for(ch).map(u16::from),
            Key::Char(_) => None,
            Key::Return => Some(13),
            Key::Delete => Some(8),
            Key::Escape => Some(27),
            Key::Up => Some(129),
            Key::Down => Some(130),
            Key::Left => Some(131),
            Key::Right => Some(132),
            Key::Function(n @ 1..=12) => Some(132 + n as u16),
            Key::Keypad(n @ 0..=9) => Some(145 + n as u16),
            Key::Function(_) | Key::Keypad(_) => None,
        }
    }

    /// Whether the game's terminating characters table lists this code
    /// (§10.5.2.1). Only function-key codes may be listed; 255 lists them all.
    pub(crate) fn ends_a_line(&self, code: u16) -> bool {
        let table = self.header.terminating_characters as u32;
        if table == 0 || !(129..=154).contains(&code) {
            return false;
        }
        (0u32..256)
            .map_while(|i| self.memory.read_byte(table + i).ok().filter(|&b| b != 0))
            .any(|b| b == 255 || b as u16 == code)
    }

    /// §15 erase_window: -1 unsplits the screen and clears it, -2 clears it, and
    /// 0 or 1 clears that window.
    pub(crate) fn erase_window(&mut self, which: i16) {
        self.flush_chunk();
        if matches!(which, 0 | -1 | -2) {
            self.output.mark_cleared();
        }
        self.screen.erase_window(which);
        if which == -1 {
            self.window = 0;
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
        header::apply_interpreter_fields(dynamic, &self.header, &self.settings);

        self.frames = vec![Frame::main()];
        self.pc = self.header.initial_pc as u32;
        self.random.reseed_from_host();
        self.screen_selected = true;
        self.window = 0;
        self.style = Style::default();
        self.buffering = true;
        self.memory_streams.clear();
        self.screen.reset();
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
        // From version 5 the instruction stores its outcome, and the save records
        // the address of the store byte instead.
        let pc = if self.header.v.is_v3() { ins.branch_data } else { ins.store_data };
        let pc = pc.unwrap_or(ins.next);
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
            let result = self.outcome(&ins, stored as u16);
            self.fault_later(result);
        }
    }

    /// Reports the outcome of save, restore or undo: a branch in version 3, a
    /// stored value from version 5.
    fn outcome(&mut self, ins: &Instruction, value: u16) -> X<()> {
        if self.header.v.is_v3() { self.branch(ins, value != 0) } else { self.store(ins, value) }
    }

    /// Replaces the game state with a saved one (§6.1.2): Flags 2 keeps the bits it
    /// has now, and the interpreter writes its own header fields again.
    fn replace_state(&mut self, saved_dynamic: &[u8], frames: Vec<Frame>) {
        let dynamic = self.memory.dynamic_mut();
        let now = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        dynamic.copy_from_slice(saved_dynamic);
        let then = u16::from_be_bytes([dynamic[FLAGS2 as usize], dynamic[FLAGS2 as usize + 1]]);
        let kept = then & !FLAGS2_SURVIVES_RESTART | now & FLAGS2_SURVIVES_RESTART;
        dynamic[FLAGS2 as usize..FLAGS2 as usize + 2].copy_from_slice(&kept.to_be_bytes());
        header::apply_interpreter_fields(dynamic, &self.header, &self.settings);
        self.frames = frames;
    }

    /// Resumes at a version 5 `save` or `save_undo` whose store byte is at
    /// `store_byte`, storing 2 there: "the game was just restored" (§15).
    fn resume_storing(&mut self, store_byte: u32) -> X<()> {
        let variable = self.memory.read_byte(store_byte)?;
        self.current = store_byte;
        self.pc = store_byte + 1;
        self.write_variable(variable, 2)
    }

    /// §15 save_undo: records a position; stores 1, or -1 when undo is not offered.
    pub(crate) fn save_undo(&mut self, ins: &Instruction) -> X<()> {
        if self.settings.undo_slots == 0 {
            return self.store(ins, 0xFFFF);
        }
        let position = undo::Position {
            dynamic: self.memory.dynamic().to_vec(),
            frames: self.frames.clone(),
            pc: ins.store_data.unwrap_or(ins.next),
        };
        self.undo.push(position, self.settings.undo_slots);
        self.store(ins, 1)
    }

    /// §15 restore_undo: returns to the latest recorded position, where the
    /// `save_undo` then stores 2. With none recorded, stores 0.
    pub(crate) fn restore_undo(&mut self, ins: &Instruction) -> X<()> {
        if self.settings.undo_slots == 0 {
            return self.store(ins, 0xFFFF);
        }
        match self.undo.pop() {
            None => self.store(ins, 0),
            Some(position) => {
                self.replace_state(&position.dynamic, position.frames);
                self.resume_storing(position.pc)
            }
        }
    }

    /// §15 restore. With no usable save the instruction reports failure and
    /// nothing else changes (FR-017).
    pub(crate) fn finish_restore(&mut self, bytes: Option<&[u8]>) {
        let Some(ins) = self.pending.take() else {
            return;
        };
        let v3 = self.header.v.is_v3();
        let original = &self.story[..self.memory.static_base()];
        let saved = bytes
            .and_then(|bytes| quetzal::read(bytes, &self.header, original).ok())
            // The call stack must be one this machine would itself allow, and the
            // saved program counter must point at readable branch data (version 3)
            // or a readable store byte.
            .filter(|saved| saved.frames.len() <= MAX_CALL_DEPTH + 1)
            .filter(|saved| match v3 {
                true => decode::decode_branch(&self.memory, saved.pc).is_ok(),
                false => self.memory.read_byte(saved.pc).is_ok(),
            });
        let Some(saved) = saved else {
            let result = self.outcome(&ins, 0);
            self.fault_later(result);
            return;
        };

        self.replace_state(&saved.dynamic, saved.frames);
        if !v3 {
            let result = self.resume_storing(saved.pc);
            self.fault_later(result);
            return;
        }
        // Resume inside the `save` that made the file, reporting success: its
        // branch is evaluated with the condition true (§15 restore).
        let Ok((branch, next)) = decode::decode_branch(&self.memory, saved.pc) else {
            return;
        };
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
