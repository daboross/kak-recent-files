// from https://github.com/ul/kak-lsp/blob/53b0ca4a4363402fceffa7f672fc2c96cad67c44/src/util.rs#L109-L117

/// Escape Kakoune string wrapped into single quote
pub fn editor_escape(s: &str) -> String {
    s.replace("'", "''")
}

/// Convert to Kakoune string by wrapping into quotes and escaping
pub fn editor_quote(s: &str) -> String {
    format!("'{}'", editor_escape(s))
}
