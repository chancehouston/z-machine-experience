//! Z-character text encoding (§3), for version 3.

const A2: &[u8] = b"0123456789.,!?_#'\"/\\-:()";

/// The Z-characters for `text`, before packing into words (§3.2 to §3.5).
fn zchars(text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for ch in text.chars() {
        match ch {
            ' ' => out.push(0),
            'a'..='z' => out.push(ch as u8 - b'a' + 6),
            'A'..='Z' => out.extend([4, ch as u8 - b'A' + 6]),
            '\n' => out.extend([5, 7]),
            _ => {
                if let Some(i) = A2.iter().position(|&c| c as char == ch) {
                    out.extend([5, i as u8 + 8]);
                } else {
                    // §3.4: A2 character 6, then the ZSCII code as two 5-bit halves.
                    let code = ch as u32;
                    assert!(code < 1024, "character {ch:?} has no ZSCII code here");
                    out.extend([5, 6, (code >> 5) as u8, (code & 31) as u8]);
                }
            }
        }
    }
    out
}

/// Packs Z-characters three to a word, padding with 5s and setting the end bit on
/// the last word (§3.2).
fn pack(mut z: Vec<u8>) -> Vec<u8> {
    if z.is_empty() {
        z.push(5);
    }
    while !z.len().is_multiple_of(3) {
        z.push(5);
    }
    let words = z.len() / 3;
    let mut out = Vec::with_capacity(words * 2);
    for (i, c) in z.chunks(3).enumerate() {
        let mut w = (c[0] as u16) << 10 | (c[1] as u16) << 5 | c[2] as u16;
        if i == words - 1 {
            w |= 0x8000;
        }
        out.extend(w.to_be_bytes());
    }
    out
}

/// Encodes `text` as a Z-string.
pub fn zencode(text: &str) -> Vec<u8> {
    pack(zchars(text))
}

/// Encodes `word` as a version 3 dictionary key: exactly six Z-characters in four
/// bytes, truncated or padded with 5s (§13.3, §13.4).
pub fn zencode_dict(word: &str) -> [u8; 4] {
    let mut z = zchars(word);
    z.resize(6, 5);
    let packed = pack(z);
    [packed[0], packed[1], packed[2], packed[3]]
}
