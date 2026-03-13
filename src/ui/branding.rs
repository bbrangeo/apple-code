//! ASCII art branding and theme-aware colors

use owo_colors::OwoColorize;

const LOGO: &str = r#"
 █████╗ ██████╗ ██████╗ ██╗     ███████╗     ██████╗ ██████╗ ██████╗ ███████╗
██╔══██╗██╔══██╗██╔══██╗██║     ██╔════╝    ██╔════╝██╔═══██╗██╔══██╗██╔════╝
███████║██████╔╝██████╔╝██║     █████╗      ██║     ██║   ██║██║  ██║█████╗  
██╔══██║██╔═══╝ ██╔═══╝ ██║     ██╔══╝      ██║     ██║   ██║██║  ██║██╔══╝  
██║  ██║██║     ██║     ███████╗███████╗    ╚██████╗╚██████╔╝██████╔╝███████╗
╚═╝  ╚═╝╚═╝     ╚═╝     ╚══════╝╚══════╝     ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝
"#;

/// Theme: light or dark terminal colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            _ => Theme::Dark,
        }
    }
}

/// Print the APPLE CODE logo with theme-aware colors.
pub fn print_logo(theme: Theme) {
    let (primary, secondary) = match theme {
        Theme::Dark => (
            owo_colors::Style::new().bright_cyan().bold(),
            owo_colors::Style::new().bright_blue(),
        ),
        Theme::Light => (
            owo_colors::Style::new().cyan().bold(),
            owo_colors::Style::new().blue(),
        ),
    };
    for (i, line) in LOGO.trim_start_matches('\n').lines().enumerate() {
        if i % 2 == 0 {
            println!("{}", line.style(primary));
        } else {
            println!("{}", line.style(secondary));
        }
    }
}

/// Print code preview without borders. Theme-aware. Uses syntax highlighting when title (language) is set.
pub fn print_code_preview(code: &str, title: Option<&str>, theme: Theme) {
    let code_color = match theme {
        Theme::Dark => owo_colors::Style::new().white(),
        Theme::Light => owo_colors::Style::new().black(),
    };

    let dark = matches!(theme, Theme::Dark);
    if let Some(lang) = title {
        let highlighted = crate::utils::highlight_to_ansi(code, Some(lang), dark);
        print!("{}", highlighted);
    } else {
        for line in code.lines() {
            println!("{}", line.style(code_color));
        }
    }
}

fn truncate_pad(s: &str, width: usize) -> String {
    let s = s.replace('\t', "  ");
    if s.len() > width {
        format!("{}…", &s[..width.saturating_sub(1)])
    } else {
        format!("{:<width$}", s, width = width)
    }
}
