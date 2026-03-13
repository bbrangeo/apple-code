//! Syntax highlighting with syntect (ANSI output for terminal)

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

/// Map language name to syntect extension for syntax lookup.
fn lang_to_extension(lang: &str) -> &str {
    match lang.to_lowercase().as_str() {
        "rust" => "rs",
        "typescript" | "ts" => "ts",
        "javascript" | "js" => "js",
        "python" | "py" => "py",
        "go" => "go",
        "java" => "java",
        "c" => "c",
        "cpp" | "c++" => "cpp",
        "csharp" | "c#" => "cs",
        "swift" => "swift",
        "kotlin" => "kt",
        "ruby" => "rb",
        "php" => "php",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "shell" | "bash" | "sh" => "sh",
        "json" => "json",
        "yaml" | "yml" => "yml",
        "markdown" | "md" => "md",
        _ => "txt",
    }
}

/// Highlight code to ANSI-colored string for terminal. Returns original code if no syntax found.
pub fn highlight_to_ansi(code: &str, language: Option<&str>, dark: bool) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme_name = if dark {
        "base16-ocean.dark"
    } else {
        "base16-ocean.light"
    };
    let theme = match ts.themes.get(theme_name) {
        Some(t) => t,
        None => return code.to_string(),
    };
    let ext = language.map(lang_to_extension).unwrap_or("txt");
    let syntax = match ps.find_syntax_by_extension(ext) {
        Some(s) => s,
        None => return code.to_string(),
    };
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut out = String::new();
    for line in LinesWithEndings::from(code) {
        if let Ok(ranges) = highlighter.highlight_line(line, &ps) {
            out.push_str(&as_24_bit_terminal_escaped(&ranges[..], true));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}
