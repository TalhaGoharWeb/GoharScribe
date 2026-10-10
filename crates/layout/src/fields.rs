//! Field results computed at layout time (page numbers) and note numbering.

use std::collections::HashMap;
use std::sync::Arc;

use goharscribe_doc::section::NumFormat;

/// Context for fields that depend on where text lands.
#[derive(Clone, Debug, Default)]
pub struct FieldCtx {
    /// Current page number (as numbered) and its format.
    pub page: u32,
    pub page_format: NumFormat,
    /// Total pages in the document.
    pub pages: u32,
    /// Pages in the current section.
    pub section_pages: u32,
    pub section: u32,
    /// Note part id → number, in document order.
    pub notes: Arc<HashMap<u32, u32>>,
    /// Document title / author for TITLE / AUTHOR fields.
    pub title: Arc<str>,
    pub author: Arc<str>,
    pub filename: Arc<str>,
}

impl FieldCtx {
    pub fn note_number(&self, id: u32) -> u32 {
        self.notes.get(&id).copied().unwrap_or(1)
    }
    /// A key for caches: the parts of the context a page-dependent paragraph depends on.
    pub fn page_key(&self) -> (u32, u32, u32, u32) {
        (self.page, self.pages, self.section_pages, self.section)
    }
}

/// The field's name (first word of the instruction, upper-cased).
pub fn field_name(instr: &str) -> String {
    instr.split_whitespace().next().unwrap_or("").trim_start_matches('=').to_ascii_uppercase()
}

/// `\* FORMAT` switch value (Arabic, roman, ROMAN, alphabetic, ALPHABETIC, CardText, Ordinal…).
fn format_switch(instr: &str) -> Option<NumFormat> {
    let mut it = instr.split_whitespace();
    while let Some(w) = it.next() {
        if w == "\\*" {
            let v = it.next()?;
            return Some(match v {
                "roman" => NumFormat::LowerRoman,
                "ROMAN" | "Roman" => NumFormat::UpperRoman,
                "alphabetic" => NumFormat::LowerLetter,
                "ALPHABETIC" | "Alphabetic" => NumFormat::UpperLetter,
                "CardText" | "cardtext" => NumFormat::CardinalText,
                "OrdText" | "ordtext" => NumFormat::OrdinalText,
                "Ordinal" | "ordinal" => NumFormat::Ordinal,
                "Arabic" | "arabic" => NumFormat::Decimal,
                _ => continue,
            });
        }
    }
    None
}

/// Display text for a field and whether it depends on the page it lands on.
pub fn field_text(instr: &str, result: &str, ctx: &FieldCtx) -> (String, bool) {
    let name = field_name(instr);
    let fmt = format_switch(instr);
    match name.as_str() {
        "PAGE" => (fmt.unwrap_or(ctx.page_format).format(ctx.page), true),
        "NUMPAGES" => (fmt.unwrap_or(NumFormat::Decimal).format(ctx.pages), true),
        "SECTIONPAGES" => (fmt.unwrap_or(NumFormat::Decimal).format(ctx.section_pages), true),
        "SECTION" => (fmt.unwrap_or(NumFormat::Decimal).format(ctx.section), true),
        "TITLE" if result.is_empty() => (ctx.title.to_string(), false),
        "AUTHOR" if result.is_empty() => (ctx.author.to_string(), false),
        "FILENAME" if result.is_empty() => (ctx.filename.to_string(), false),
        "DATE" | "TIME" | "CREATEDATE" | "SAVEDATE" => {
            // Evaluate to current date/time; format switch \@ "fmt" supported.
            let date_fmt = date_format_switch(instr).unwrap_or_else(|| {
                if name.as_str() == "TIME" { "h:mm AM/PM".to_string() } else { "M/d/yyyy".to_string() }
            });
            (format_current_date(&date_fmt), false)
        }
        _ => (result.to_string(), false),
    }
}

/// Extract the \@ "format" switch from a field instruction.
fn date_format_switch(instr: &str) -> Option<String> {
    // Look for \@ "format" or \@ 'format'
    let mut chars = instr.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c == '\\' {
            if let Some((_, '@')) = chars.peek() {
                chars.next();
                // Skip whitespace, expect quoted string.
                for (_, c2) in chars.by_ref() {
                    if c2 == '"' || c2 == '\'' {
                        let quote = c2;
                        let mut fmt = String::new();
                        for (_, c3) in chars.by_ref() {
                            if c3 == quote {
                                return Some(fmt);
                            }
                            fmt.push(c3);
                        }
                        break;
                    } else if !c2.is_whitespace() {
                        break;
                    }
                }
            }
        }
    }
    None
}

/// Format the current date/time according to a Word-style format string.
fn format_current_date(fmt: &str) -> String {
    // Get current date/time components.
    let (y, m, d, hh, mm) = current_ymdhm();
    const MONTHS: [&str; 12] =
        ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let month = MONTHS.get(m.saturating_sub(1)).copied().unwrap_or("January");
    let mut out = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut n = 1;
        while chars.get(i + n) == Some(&c) {
            n += 1;
        }
        match (c, n) {
            ('y', _) => out.push_str(&y.to_string()),
            ('M', 1) => out.push_str(&m.to_string()),
            ('M', 2) => out.push_str(&format!("{:02}", m)),
            ('M', _) => out.push_str(month),
            ('d', 1) => out.push_str(&d.to_string()),
            ('d', _) => out.push_str(&format!("{:02}", d)),
            ('h', _) => {
                let h12 = if hh % 12 == 0 { 12 } else { hh % 12 };
                if n == 1 { out.push_str(&h12.to_string()) } else { out.push_str(&format!("{:02}", h12)) }
            }
            ('H', 1) => out.push_str(&hh.to_string()),
            ('H', _) => out.push_str(&format!("{:02}", hh)),
            ('m', 1) => out.push_str(&mm.to_string()),
            ('m', _) => out.push_str(&format!("{:02}", mm)),
            ('A', _) | ('a', _) => {
                // AM/PM marker (Word uses AM/PM in format string).
                if fmt[i..].starts_with("AM/PM") || fmt[i..].starts_with("am/pm") {
                    out.push_str(if hh < 12 { "AM" } else { "PM" });
                    i += 4; // skip "AM/PM", n was counted
                    // Adjust n to avoid double-counting.
                    let skip = 5usize.saturating_sub(n);
                    i += skip;
                    continue;
                } else {
                    for _ in 0..n { out.push(c); }
                }
            }
            _ => {
                for _ in 0..n { out.push(c); }
            }
        }
        i += n;
    }
    out
}

/// Current (year, month, day, hour, minute) in local time (approximate via UTC).
fn current_ymdhm() -> (i32, usize, u32, u32, u32) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        // Days since epoch -> civil date (Howard Hinnant's algorithm).
        let days = (secs / 86_400) as i64;
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = (z - era * 146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        let secs_of_day = (secs % 86_400) as u32;
        let hh = secs_of_day / 3600;
        let mm = (secs_of_day % 3600) / 60;
        (y as i32, m as usize, d, hh, mm)
    }
    #[cfg(target_arch = "wasm32")]
    {
        (2026, 1, 1, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_fields() {
        let ctx = FieldCtx { page: 4, pages: 12, ..Default::default() };
        assert_eq!(field_text("PAGE", "", &ctx), ("4".into(), true));
        assert_eq!(field_text(" PAGE \\* ROMAN ", "", &ctx).0, "IV");
        assert_eq!(field_text("NUMPAGES \\* CardText", "", &ctx).0, "Twelve");
        // DATE/TIME now evaluate to current date/time (not static result).
        let (date_text, _) = field_text("DATE \\@ \"M/d/yyyy\"", "1/2/2026", &ctx);
        assert!(date_text.contains('/'), "DATE should evaluate, got {date_text:?}");
        assert_ne!(date_text, "1/2/2026", "DATE should not return static result");
        let (time_text, _) = field_text("TIME \\@ \"h:mm AM/PM\"", "", &ctx);
        assert!(time_text.contains("M"), "TIME should evaluate, got {time_text:?}");
        assert_eq!(field_name("=SUM(ABOVE)"), "SUM(ABOVE)");
    }

    #[test]
    fn date_format_switch_extraction() {
        assert_eq!(date_format_switch("DATE \\@ \"MMMM d, yyyy\""), Some("MMMM d, yyyy".to_string()));
        assert_eq!(date_format_switch("TIME \\@ 'h:mm'"), Some("h:mm".to_string()));
        assert_eq!(date_format_switch("DATE"), None);
    }
}
