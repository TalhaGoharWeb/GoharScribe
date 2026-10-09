//! Urdu phonetic keyboard input and Roman-Urdu transliteration.
//!
//! The phonetic layout maps ASCII keys to Urdu characters (the de-facto
//! phonetic standard: `a`→`ا`, `x`→`ش`, `R`→`ڑ`, …). Transliteration converts
//! whole strings between Urdu script and Roman letters in both directions.
//!
//! Keyboard-map design adapted from Naseem Amjad's Urdu Nigar
//! (`setUrduPhoneticUnicodes`, `setUrduUnicode2Roman`); reimplemented here in
//! Rust. See ATTRIBUTION.md.

/// Map one ASCII key to its Urdu character (phonetic layout).
/// Unmapped keys pass through unchanged.
pub fn phonetic_key(key: char) -> char {
    match key {
        '\r' => '\n',
        ' ' => ' ',
        '!' => '!',
        '"' => '”',
        '\'' => '’',
        '(' => '(',
        ')' => ')',
        ',' => '؛',
        '.' => '۔',
        '0' => '۰',
        '1' => '۱',
        '2' => '۲',
        '3' => '۳',
        '4' => '۴',
        '5' => '۵',
        '6' => '۶',
        '7' => '۷',
        '8' => '۸',
        '9' => '۹',
        ':' => ':',
        '?' => '؟',
        'A' => 'آ',
        'C' => 'ث',
        'D' => 'ڈ',
        'G' => 'غ',
        'H' => 'ح',
        'J' => 'ض',
        'K' => 'خ',
        'N' => 'ں',
        'R' => 'ڑ',
        'S' => 'ص',
        'T' => 'ٹ',
        'V' => 'ظ',
        'X' => 'ژ',
        'Z' => 'ذ',
        'a' => 'ا',
        'b' => 'ب',
        'c' => 'چ',
        'd' => 'د',
        'e' => 'ع',
        'f' => 'ف',
        'g' => 'گ',
        'h' => 'ھ',
        'i' => 'ی',
        'j' => 'ج',
        'k' => 'ک',
        'l' => 'ل',
        'm' => 'م',
        'n' => 'ن',
        'o' => 'ہ',
        'p' => 'پ',
        'q' => 'ق',
        'r' => 'ر',
        's' => 'س',
        't' => 'ت',
        'u' => 'ء',
        'v' => 'ط',
        'w' => 'و',
        'x' => 'ش',
        'y' => 'ے',
        'z' => 'ز',
        '$' => 'ئ',
        other => other,
    }
}

/// Type a whole ASCII string through the phonetic layout.
pub fn phonetic_type(text: &str) -> String {
    text.chars().map(phonetic_key).collect()
}

/// Map one Urdu character to its Roman letters. Unmapped characters pass
/// through unchanged.
pub fn urdu_to_roman_char(c: char) -> &'static str {
    match c {
        'ا' => "A",
        'ٵ' => "A",
        'ٳ' => "A",
        'ذ' => "Z",
        'آ' => "AA",
        'ب' => "B",
        'پ' => "P",
        'ت' => "T",
        'ط' => "T",
        'ٹ' => "T",
        'ج' => "J",
        'س' => "S",
        'ث' => "S",
        'ص' => "S",
        'چ' => "CH",
        'ح' => "H",
        'ہ' => "H",
        'ۃ' => "H",
        '۟' => "H",
        'خ' => "KH",
        'د' => "D",
        'ڈ' => "D",
        'ز' => "Z",
        'ض' => "Z",
        'ظ' => "Z",
        'ژ' => "Z",
        'ر' => "R",
        'ڑ' => "R",
        'ش' => "SH",
        'غ' => "GH",
        'ف' => "F",
        'ک' => "K",
        'ق' => "K",
        'گ' => "G",
        'ل' => "L",
        'م' => "M",
        'ن' => "N",
        'ں' => "N",
        'و' => "O",
        'ى' => "Y",
        'ئ' => "Y",
        'ی' => "Y",
        'ے' => "E",
        'ھ' => "H",
        'ي' => "E",
        'ۂ' => "AH",
        'ع' => "A",
        'ك' => "K",
        'ء' => "A",
        'ؤ' => "O",
        '؟' => "?",
        '۱' => "1",
        '۲' => "2",
        '۳' => "3",
        '۴' => "4",
        '۵' => "5",
        '۶' => "6",
        '۷' => "7",
        '۸' => "8",
        '۹' => "9",
        '۰' => "0",
        _ => "",
    }
}

/// Transliterate Urdu-script text to Roman letters.
pub fn urdu_to_roman(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        let r = urdu_to_roman_char(c);
        if r.is_empty() {
            out.push(c);
        } else {
            out.push_str(r);
        }
    }
    out
}

/// Two-letter Roman sequences with dedicated Urdu letters.
fn roman_digraph(pair: &str) -> Option<char> {
    match pair {
        "aa" | "AA" => Some('آ'),
        "ch" | "CH" => Some('چ'),
        "sh" | "SH" => Some('ش'),
        "kh" | "KH" => Some('خ'),
        "gh" | "GH" => Some('غ'),
        _ => None,
    }
}

/// Transliterate Roman-Urdu text to Urdu script (longest match: digraphs first,
/// then the phonetic single-key map).
pub fn roman_to_urdu(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if let Some(&n) = it.peek() {
            let mut pair = String::with_capacity(2);
            pair.push(c);
            pair.push(n);
            if let Some(u) = roman_digraph(&pair) {
                it.next();
                out.push(u);
                continue;
            }
            // Case-insensitive digraph retry (e.g. "Sh").
            let lower = pair.to_lowercase();
            if let Some(u) = roman_digraph(&lower) {
                it.next();
                out.push(u);
                continue;
            }
        }
        out.push(phonetic_key(c));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phonetic_basics() {
        assert_eq!(phonetic_type("slam"), "سلام");
        assert_eq!(phonetic_type("assalam o alaikum"), "اسسالام ہ الایکءم");
        assert_eq!(phonetic_key('R'), 'ڑ');
        assert_eq!(phonetic_key('x'), 'ش');
        assert_eq!(phonetic_key('Z'), 'ذ');
        assert_eq!(phonetic_key('$'), 'ئ');
        assert_eq!(phonetic_key('?'), '؟');
        assert_eq!(phonetic_key('5'), '۵');
        // Unmapped keys pass through.
        assert_eq!(phonetic_key('%'), '%');
    }

    #[test]
    fn urdu_to_roman_basics() {
        assert_eq!(urdu_to_roman("اسلام"), "ASLAM");
        assert_eq!(urdu_to_roman("چائے"), "CHAYE");
        assert_eq!(urdu_to_roman("۱۲۳"), "123");
        assert_eq!(urdu_to_roman("؟"), "?");
        // ذ is Z (the source table's "A" was a decompiler artifact).
        assert_eq!(urdu_to_roman("ذ"), "Z");
    }

    #[test]
    fn roman_to_urdu_basics() {
        assert_eq!(roman_to_urdu("sh"), "ش");
        assert_eq!(roman_to_urdu("ch"), "چ");
        assert_eq!(roman_to_urdu("aa"), "آ");
        assert_eq!(roman_to_urdu("slam"), "سلام");
        // Round trip through the phonetic map.
        let urdu = phonetic_type("ktab");
        assert_eq!(urdu, "کتاب");
        assert_eq!(roman_to_urdu("ktab"), "کتاب");
    }
}
