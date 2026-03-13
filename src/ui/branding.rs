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

/// Print a bordered code block (for preview). Theme-aware. Uses syntax highlighting when title (language) is set.
pub fn print_code_preview(code: &str, title: Option<&str>, theme: Theme) {
    let border_color = match theme {
        Theme::Dark => owo_colors::Style::new().bright_black(),
        Theme::Light => owo_colors::Style::new().black(),
    };
    let title_color = match theme {
        Theme::Dark => owo_colors::Style::new().bright_white().bold(),
        Theme::Light => owo_colors::Style::new().black().bold(),
    };
    let code_color = match theme {
        Theme::Dark => owo_colors::Style::new().white(),
        Theme::Light => owo_colors::Style::new().black(),
    };

    let width = code.lines().map(str::len).max().unwrap_or(40).min(80);
    let border = "─".repeat(width + 2);
    let top = format!("┌{}┐", border);
    let bottom = format!("└{}┘", border);
    println!("{}", top.style(border_color));
    if let Some(t) = title {
        let title_line = format!("│ {} │", truncate_pad(t, width));
        println!("{}", title_line.style(title_color));
        println!("{}", format!("├{}┤", border).style(border_color));
    }
    let dark = matches!(theme, Theme::Dark);
    if let Some(lang) = title {
        let highlighted = crate::utils::highlight_to_ansi(code, Some(lang), dark);
        let ansi_lines: Vec<&str> = highlighted.trim_end_matches('\n').split('\n').collect();
        for (i, code_line) in code.lines().enumerate() {
            let ansi_line = ansi_lines.get(i).unwrap_or(&"");
            let pad_len = width.saturating_sub(code_line.len().min(width));
            let padding = " ".repeat(pad_len);
            println!("│ {}{} │", ansi_line, padding);
        }
    } else {
        for line in code.lines() {
            let padded = truncate_pad(line, width);
            println!("{}", format!("│ {} │", padded).style(code_color));
        }
    }
    println!("{}", bottom.style(border_color));
}

fn truncate_pad(s: &str, width: usize) -> String {
    let s = s.replace('\t', "  ");
    if s.len() > width {
        format!("{}…", &s[..width.saturating_sub(1)])
    } else {
        format!("{:<width$}", s, width = width)
    }
}
