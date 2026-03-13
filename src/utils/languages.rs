//! Supported languages and language detection

/// Supported languages: (display name, aliases for CLI)
pub const SUPPORTED_LANGUAGES: &[(&str, &[&str])] = &[
    ("typescript", &["ts"]),
    ("javascript", &["js"]),
    ("python", &["py"]),
    ("rust", &["rs"]),
    ("go", &["golang"]),
    ("java", &[]),
    ("c", &[]),
    ("cpp", &["c++"]),
    ("csharp", &["c#", "cs"]),
    ("swift", &[]),
    ("kotlin", &["kt"]),
    ("ruby", &["rb"]),
    ("php", &[]),
    ("html", &[]),
    ("css", &[]),
    ("sql", &[]),
    ("shell", &["bash", "sh"]),
    ("json", &[]),
    ("yaml", &["yml"]),
    ("markdown", &["md"]),
];

/// Return true if `lang` is a supported language or alias.
pub fn is_supported(lang: &str) -> bool {
    let lang = lang.to_lowercase();
    for (name, aliases) in SUPPORTED_LANGUAGES {
        if *name == lang {
            return true;
        }
        if aliases.contains(&lang.as_str()) {
            return true;
        }
    }
    false
}

/// Normalize language (alias -> canonical name).
pub fn normalize(lang: &str) -> Option<String> {
    let lang = lang.to_lowercase();
    for (name, aliases) in SUPPORTED_LANGUAGES {
        if *name == lang {
            return Some(name.to_string());
        }
        if aliases.contains(&lang.as_str()) {
            return Some(name.to_string());
        }
    }
    None
}

/// Suggest language from file extension (e.g. ".rs" -> "rust").
pub fn from_extension(ext: &str) -> Option<String> {
    let ext = ext.trim_start_matches('.').to_lowercase();
    for (name, aliases) in SUPPORTED_LANGUAGES {
        if *name == ext || aliases.contains(&ext.as_str()) {
            return Some(name.to_string());
        }
    }
    None
}

/// Suggest language from prompt (simple keyword heuristic).
pub fn from_prompt(prompt: &str) -> Option<String> {
    let p = prompt.to_lowercase();
    let keywords = [
        ("typescript", "typescript"),
        ("python", "python"),
        ("rust", "rust"),
        ("react", "typescript"),
        ("express", "javascript"),
        ("django", "python"),
        ("flask", "python"),
    ];
    for (keyword, lang) in keywords {
        if p.contains(keyword) {
            return Some(lang.to_string());
        }
    }
    None
}
