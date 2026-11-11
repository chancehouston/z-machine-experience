//! The divergence ledger (`docs/divergences.md`): reading its rules, applying them
//! to a reference transcript, and saying where two transcripts differ.
//!
//! The ledger is the one place where a difference from the reference interpreter
//! may be excused, and each excuse must cite the Z-Machine Standard (constitution
//! Principle I). So the reading here is strict: anything that cannot be understood
//! is an error, and a rule that excuses nothing is an error too.

use std::ops::Range;

/// One rule from the ledger.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    /// Excuses one difference in a transcript comparison.
    Transcript {
        id: String,
        /// SHA-256 of the story file the rule is about, in lower-case hex.
        story: String,
        /// The command, counting from 1, after which the difference appears; 0 is
        /// the text before the first command.
        after_command: usize,
        /// The text in the reference transcript.
        reference: String,
        /// The text in ours.
        ours: String,
    },
    /// Excuses one czech test whose expectation conflicts with the Standard.
    Czech { id: String, test: String },
}

impl Rule {
    pub fn id(&self) -> &str {
        match self {
            Rule::Transcript { id, .. } | Rule::Czech { id, .. } => id,
        }
    }
}

/// Where two transcripts first differ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Difference {
    /// Byte offset of the first difference.
    pub offset: usize,
    /// How many commands had been entered by then; 0 if none.
    pub after_command: usize,
    /// The last of those commands, or empty if none.
    pub command: String,
    /// A few lines of the reference around the difference.
    pub reference_context: String,
    /// The same lines of ours.
    pub ours_context: String,
}

impl std::fmt::Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.after_command == 0 {
            writeln!(
                f,
                "first difference at byte {}, before the first command",
                self.offset
            )?;
        } else {
            writeln!(
                f,
                "first difference at byte {}, after command {} ({:?})",
                self.offset, self.after_command, self.command
            )?;
        }
        writeln!(f, "--- reference ---\n{}", self.reference_context)?;
        write!(f, "--- ours ---\n{}", self.ours_context)
    }
}

/// Turns `\n` into a line feed and `\\` into a backslash.
fn unescape(text: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            other => {
                return Err(format!(
                    "unknown escape \\{}",
                    other.map_or(String::new(), String::from)
                ));
            }
        }
    }
    Ok(out)
}

/// The entry being read.
struct Entry {
    id: String,
    cites_standard: bool,
    has_rule: bool,
}

fn build_rule(id: &str, lines: &[&str]) -> Result<Rule, String> {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    for line in lines {
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("{line:?} is not `key: value`"))?;
        // One space after the colon separates; anything further is part of the value.
        fields.push((key.trim(), value.strip_prefix(' ').unwrap_or(value)));
    }
    let take = |name: &str, fields: &mut Vec<(&str, &str)>| -> Result<String, String> {
        let at = fields
            .iter()
            .position(|(k, _)| *k == name)
            .ok_or(format!("no `{name}`"))?;
        Ok(fields.remove(at).1.to_string())
    };
    let rule = match take("kind", &mut fields)?.as_str() {
        "transcript" => {
            let story = take("story", &mut fields)?;
            if story.len() != 64
                || !story
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(format!(
                    "`story` must be a SHA-256 digest in lower-case hex, not {story:?}"
                ));
            }
            let after = take("after-command", &mut fields)?;
            let after_command = after
                .trim()
                .parse()
                .map_err(|_| format!("`after-command` must be a number, not {after:?}"))?;
            let reference = unescape(&take("reference", &mut fields)?)?;
            if reference.is_empty() {
                return Err("`reference` is empty, so the rule points at nothing".to_string());
            }
            let ours = unescape(&take("ours", &mut fields)?)?;
            Rule::Transcript {
                id: id.to_string(),
                story,
                after_command,
                reference,
                ours,
            }
        }
        "czech" => {
            let test = take("test", &mut fields)?.trim().to_string();
            if test.is_empty() {
                return Err("`test` is empty".to_string());
            }
            Rule::Czech {
                id: id.to_string(),
                test,
            }
        }
        other => return Err(format!("unknown kind {other:?}")),
    };
    match fields.first() {
        Some((key, _)) => Err(format!("unexpected `{key}`")),
        None => Ok(rule),
    }
}

/// Reads the rules from the text of the ledger. Only what follows the
/// `## Entries` heading is read; the format description above it is not.
pub fn parse(ledger: &str) -> Result<Vec<Rule>, String> {
    let Some((_, entries)) = ledger.split_once("\n## Entries") else {
        return Err("no `## Entries` section".to_string());
    };

    let finish = |entry: &Option<Entry>| match entry {
        Some(e) if !e.has_rule => Err(format!("{}: the entry has no rule", e.id)),
        _ => Ok(()),
    };

    let mut rules: Vec<Rule> = Vec::new();
    let mut entry: Option<Entry> = None;
    let mut lines = entries.lines().skip(1);
    while let Some(line) = lines.next() {
        if let Some(heading) = line.strip_prefix("### ") {
            finish(&entry)?;
            let id = heading
                .split_whitespace()
                .next()
                .ok_or("an entry heading with no identifier")?;
            if rules.iter().any(|r| r.id() == id) {
                return Err(format!("{id}: the identifier is used twice"));
            }
            entry = Some(Entry {
                id: id.to_string(),
                cites_standard: false,
                has_rule: false,
            });
        } else if let Some(citation) = line.strip_prefix("Standard:") {
            if let Some(e) = entry.as_mut() {
                e.cites_standard = citation.contains('\u{a7}');
            }
        } else if line.trim_end() == "```rule" {
            let e = entry
                .as_mut()
                .ok_or("a rule appears before any entry heading")?;
            if e.has_rule {
                return Err(format!("{}: more than one rule in the entry", e.id));
            }
            if !e.cites_standard {
                return Err(format!(
                    "{}: the entry must cite the Standard on a line `Standard: \u{a7}...` before its rule",
                    e.id
                ));
            }
            let mut body = Vec::new();
            loop {
                match lines.next() {
                    Some(l) if l.trim_end() == "```" => break,
                    Some(l) => body.push(l),
                    None => return Err(format!("{}: the rule block is not closed", e.id)),
                }
            }
            rules.push(build_rule(&e.id, &body).map_err(|reason| format!("{}: {reason}", e.id))?);
            e.has_rule = true;
        }
    }
    finish(&entry)?;
    Ok(rules)
}

/// Divides a transcript by its commands. Region 0 is the text before the first
/// command's echo; region n is the text after the echo of command n, up to the
/// echo of the next.
///
/// A command is echoed as `>` followed by the command and a line feed
/// (contracts/transcript-format.md rule 4). `None` if the commands are not all
/// there in order.
fn regions(transcript: &str, commands: &[&str]) -> Option<Vec<Range<usize>>> {
    let mut regions = Vec::with_capacity(commands.len() + 1);
    let mut start = 0;
    for command in commands {
        let echo = format!(">{command}\n");
        let at = start + transcript[start..].find(&echo)?;
        regions.push(start..at);
        start = at + echo.len();
    }
    regions.push(start..transcript.len());
    Some(regions)
}

/// Applies the transcript rules for one story file to a reference transcript,
/// giving the transcript this interpreter is expected to produce.
///
/// `commands` are the input lines echoed in the transcript, in order. Every rule
/// for this story must match exactly once in the text after its command; otherwise
/// it is an error naming the rule.
pub fn apply(
    reference: &str,
    rules: &[Rule],
    story: &str,
    commands: &[&str],
) -> Result<String, String> {
    let regions = regions(reference, commands)
        .ok_or("the reference transcript does not contain the commands in order")?;

    // (range to replace, replacement)
    let mut edits: Vec<(Range<usize>, &str)> = Vec::new();
    for rule in rules {
        let Rule::Transcript {
            id,
            story: rule_story,
            after_command,
            reference: theirs,
            ours,
        } = rule
        else {
            continue;
        };
        if rule_story != story {
            continue;
        }
        let region = regions.get(*after_command).ok_or_else(|| {
            format!("{id}: there is no command {after_command} in the transcript")
        })?;
        let text = &reference[region.clone()];
        let first = text
            .find(theirs.as_str())
            .ok_or_else(|| format!("{id}: matches nothing after command {after_command}"))?;
        if text[first + theirs.len()..].contains(theirs.as_str()) {
            return Err(format!(
                "{id}: matches more than once after command {after_command}"
            ));
        }
        let at = region.start + first;
        edits.push((at..at + theirs.len(), ours));
    }

    edits.sort_by_key(|(range, _)| range.start);
    let mut out = String::with_capacity(reference.len());
    let mut copied = 0;
    for (range, replacement) in edits {
        if range.start < copied {
            return Err("two rules overlap in the transcript".to_string());
        }
        out.push_str(&reference[copied..range.start]);
        out.push_str(replacement);
        copied = range.end;
    }
    out.push_str(&reference[copied..]);
    Ok(out)
}

/// A few whole lines of `text` around byte `offset`: two before and two after.
fn context(text: &str, offset: usize) -> String {
    let offset = offset.min(text.len());
    let bytes = text.as_bytes();
    let mut start = offset;
    let mut seen = 0;
    while start > 0 {
        if bytes[start - 1] == b'\n' {
            seen += 1;
            if seen == 3 {
                break;
            }
        }
        start -= 1;
    }
    let mut end = offset;
    let mut seen = 0;
    while end < bytes.len() {
        if bytes[end] == b'\n' {
            seen += 1;
            if seen == 3 {
                break;
            }
        }
        end += 1;
    }
    // Line feeds are single bytes, so both ends fall between characters.
    text[start..end].to_string()
}

/// Finds where two transcripts first differ, or `None` if they are the same.
pub fn first_difference(ours: &str, reference: &str, commands: &[&str]) -> Option<Difference> {
    if ours == reference {
        return None;
    }
    let offset = ours
        .bytes()
        .zip(reference.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(ours.len().min(reference.len()));

    // Count the commands whose echo is complete before the difference.
    let mut after_command = 0;
    let mut search_from = 0;
    for command in commands {
        let echo = format!(">{command}\n");
        match reference[search_from..].find(&echo) {
            Some(at) if search_from + at + echo.len() <= offset => {
                after_command += 1;
                search_from += at + echo.len();
            }
            _ => break,
        }
    }
    let command = if after_command == 0 {
        ""
    } else {
        commands[after_command - 1]
    };

    Some(Difference {
        offset,
        after_command,
        command: command.to_string(),
        reference_context: context(reference, offset),
        ours_context: context(ours, offset),
    })
}
