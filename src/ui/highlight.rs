/// Apply simple case-insensitive substring highlighting using Pango markup.
/// - Escapes markup in the input text
/// - Highlights all non-overlapping occurrences of `query` with coral foreground
/// - If query is empty or starts with a command prefix (@ | $ | >), returns escaped text unchanged
/// - Keeps allocations minimal; O(n) over text length
pub fn apply_highlight(text: &str, query: &str) -> String {
    let query = query.trim();
    // Skip highlighting on empty/very short queries and command prefixes for performance and clarity
    if query.is_empty()
        || query.len() < 2
        || matches!(query.as_bytes().get(0), Some(b'@' | b'$' | b'>'))
    {
        return escape_markup(text);
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if query_lower.is_empty() || !text_lower.contains(&query_lower) {
        return escape_markup(text);
    }

    let mut out = String::with_capacity(text.len() + 16);
    let mut i = 0;

    while i < text.len() {
        // Find next match from i
        if let Some(pos) = text_lower[i..].find(&query_lower) {
            let start = i + pos;
            let end = start + query_lower.len();
            // Push preceding segment escaped
            out.push_str(&escape_markup(&text[i..start]));
            // Push highlighted segment
            out.push_str("<span foreground=\"#FF6363\">");
            out.push_str(&escape_markup(&text[start..end]));
            out.push_str("</span>");
            i = end;
        } else {
            // No more matches, push the rest
            out.push_str(&escape_markup(&text[i..]));
            break;
        }
    }

    out
}

/// Escape text for safe Pango markup usage
pub fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_query_returns_escaped() {
        assert_eq!(apply_highlight("a < b & c", ""), "a &lt; b &amp; c");
        assert_eq!(apply_highlight("hello", "@clip"), "hello");
    }

    #[test]
    fn highlights_case_insensitive() {
        let s = apply_highlight("Calculator", "calc");
        assert!(s.contains("<span"));
        assert!(s.contains("Cal"));
    }

    #[test]
    fn highlights_multiple_occurrences() {
        let s = apply_highlight("ababa", "aba");
        // Non-overlapping: highlights the first 'aba'
        assert!(s.starts_with("<span"));
        assert!(s.ends_with("ba"));
    }

    #[test]
    fn escapes_inside_and_outside() {
        let s = apply_highlight("a<b>a", "<b>");
        // escaped text and escaped highlight
        assert!(s.contains("&lt;b&gt;"));
        assert!(s.contains("<span"));
    }
}
