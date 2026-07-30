use std::env;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";

#[derive(Debug)]
struct Options {
    files: Vec<String>,
    numbers: bool,
    show_all: bool,
    squeeze_blank: bool,
    plain: bool,
    color: ColorChoice,
    style: String,
    language: Option<String>,
    file_name: Option<String>,
    line_range: Option<LineRange>,
    paging: Paging,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            numbers: false,
            show_all: false,
            squeeze_blank: false,
            plain: false,
            color: ColorChoice::Auto,
            style: "default".to_owned(),
            language: None,
            file_name: None,
            line_range: None,
            paging: Paging::Auto,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
enum ColorChoice {
    Always,
    #[default]
    Auto,
    Never,
}

#[derive(Debug, Default, PartialEq)]
enum Paging {
    Always,
    #[default]
    Auto,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineRange {
    start: usize,
    end: Option<usize>,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.contains("Broken pipe") {
                return ExitCode::SUCCESS;
            }
            eprintln!("ofat: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let options = parse_args(args)?;
    let color = !options.plain
        && options.style != "plain"
        && match options.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none(),
        };
    let files = if options.files.is_empty() {
        vec!["-".to_owned()]
    } else {
        options.files.clone()
    };
    let many_files = files.len() > 1;
    let decorations = !options.plain && decorations_enabled(&options, many_files, color);
    let mut output = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let content = read_input(file)?;
        if decorations {
            if index > 0 {
                writeln!(output).map_err(|error| error.to_string())?;
            }
            write_header(
                &mut output,
                options.file_name.as_deref().unwrap_or(file),
                color,
            )?;
        }
        render(&mut output, &content, file, &options, color)?;
    }
    write_output(&output, options.paging, io::stdout().is_terminal())
}

fn parse_args(args: Vec<String>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut positional_only = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if positional_only {
            options.files.push(arg);
            continue;
        }
        match arg.as_str() {
            "--" => positional_only = true,
            "-n" | "--number" => {
                options.numbers = true;
                options.style = "numbers".to_owned();
            }
            "-A" | "--show-all" => options.show_all = true,
            "-s" | "--squeeze-blank" => options.squeeze_blank = true,
            "-p" | "--plain" => options.plain = true,
            "-pp" => {
                options.plain = true;
                options.paging = Paging::Never;
            }
            "-l" | "--language" => {
                options.language = Some(args.next().ok_or("missing value after --language")?)
            }
            value if value.starts_with("--language=") => {
                options.language = Some(value[11..].to_owned())
            }
            "--file-name" => {
                options.file_name = Some(args.next().ok_or("missing value after --file-name")?)
            }
            value if value.starts_with("--file-name=") => {
                options.file_name = Some(value[12..].to_owned())
            }
            "-r" | "--line-range" => {
                options.line_range = Some(parse_range(
                    &args.next().ok_or("missing value after --line-range")?,
                )?)
            }
            value if value.starts_with("--line-range=") => {
                options.line_range = Some(parse_range(&value[13..])?)
            }
            "--style" => options.style = args.next().ok_or("missing value after --style")?,
            value if value.starts_with("--style=") => options.style = value[8..].to_owned(),
            "-P" => options.paging = Paging::Never,
            "--paging" => {
                options.paging = parse_paging(&args.next().ok_or("missing value after --paging")?)?
            }
            value if value.starts_with("--paging=") => options.paging = parse_paging(&value[9..])?,
            "-L" | "--list-languages" => {
                print_languages();
                std::process::exit(0);
            }
            "--list-themes" => {
                print_themes();
                std::process::exit(0);
            }
            "--color" => {
                options.color = parse_color(&args.next().ok_or("missing value after --color")?)?
            }
            value if value.starts_with("--color=") => options.color = parse_color(&value[8..])?,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("ofat {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option: {value}"));
            }
            file => options.files.push(file.to_owned()),
        }
    }
    Ok(options)
}

fn parse_color(value: &str) -> Result<ColorChoice, String> {
    match value {
        "always" => Ok(ColorChoice::Always),
        "auto" => Ok(ColorChoice::Auto),
        "never" => Ok(ColorChoice::Never),
        _ => Err(format!(
            "invalid color choice: {value} (use always, auto, or never)"
        )),
    }
}

fn parse_paging(value: &str) -> Result<Paging, String> {
    match value {
        "always" => Ok(Paging::Always),
        "auto" => Ok(Paging::Auto),
        "never" => Ok(Paging::Never),
        _ => Err(format!(
            "invalid paging choice: {value} (use always, auto, or never)"
        )),
    }
}

fn parse_range(value: &str) -> Result<LineRange, String> {
    let (start, end) = value.split_once(':').unwrap_or((value, value));
    let start = if start.is_empty() {
        1
    } else {
        start
            .parse()
            .map_err(|_| format!("invalid line range: {value}"))?
    };
    let end = if end.is_empty() {
        None
    } else {
        Some(
            end.parse()
                .map_err(|_| format!("invalid line range: {value}"))?,
        )
    };
    if end.is_some_and(|end| end < start) {
        return Err(format!("invalid line range: {value}"));
    }
    Ok(LineRange { start, end })
}

fn decorations_enabled(options: &Options, many_files: bool, color: bool) -> bool {
    match options.style.as_str() {
        "plain" | "numbers" => false,
        "full" | "header" | "default" => color || many_files,
        _ => color || many_files,
    }
}

fn show_numbers(options: &Options) -> bool {
    !options.plain
        && options.style != "plain"
        && (options.numbers || matches!(options.style.as_str(), "default" | "full" | "numbers"))
}

fn show_grid(options: &Options) -> bool {
    !matches!(options.style.as_str(), "plain" | "numbers")
}

fn write_output(output: &[u8], paging: Paging, is_terminal: bool) -> Result<(), String> {
    let page = matches!(paging, Paging::Always)
        || (matches!(paging, Paging::Auto)
            && is_terminal
            && output.iter().filter(|byte| **byte == b'\n').count() > terminal_height());
    if page {
        let mut pager = Command::new("less")
            .args(["-R", "-F", "-X"])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|error| format!("cannot start pager: {error}"))?;
        let mut input = pager.stdin.take().ok_or("cannot open pager input")?;
        input.write_all(output).map_err(|error| error.to_string())?;
        drop(input);
        pager.wait().map_err(|error| error.to_string())?;
        Ok(())
    } else {
        io::stdout()
            .lock()
            .write_all(output)
            .map_err(|error| error.to_string())
    }
}

fn terminal_height() -> usize {
    env::var("LINES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(24)
}

fn read_input(file: &str) -> Result<String, String> {
    if file == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("cannot read stdin: {error}"))?;
        Ok(input)
    } else {
        fs::read_to_string(file).map_err(|error| format!("cannot read {file}: {error}"))
    }
}

fn write_header(output: &mut impl Write, file: &str, color: bool) -> Result<(), String> {
    if color {
        writeln!(
            output,
            "{DIM}── {BOLD}{CYAN}{file}{RESET} {DIM}────────────────────────{RESET}"
        )
    } else {
        writeln!(output, "── {file} ────────────────────────")
    }
    .map_err(|error| error.to_string())
}

fn render(
    output: &mut impl Write,
    content: &str,
    file: &str,
    options: &Options,
    color: bool,
) -> Result<(), String> {
    let lines: Vec<&str> = content.lines().collect();
    let width = lines.len().max(1).to_string().len();
    let mut blank_run = false;
    let mut display_number = 0;
    for (source_number, raw_line) in lines.iter().enumerate() {
        let source_number = source_number + 1;
        if let Some(range) = options.line_range {
            if source_number < range.start || range.end.is_some_and(|end| source_number > end) {
                continue;
            }
        }
        let is_blank = raw_line.is_empty();
        if options.squeeze_blank && is_blank && blank_run {
            continue;
        }
        blank_run = is_blank;
        display_number += 1;
        let rendered_number = if options.line_range.is_some() {
            source_number
        } else {
            display_number
        };
        if show_numbers(options) {
            if color {
                if show_grid(options) {
                    write!(
                        output,
                        "{DIM}{:>width$} │ {RESET}",
                        rendered_number,
                        width = width
                    )
                } else {
                    write!(
                        output,
                        "{DIM}{:>width$} {RESET}",
                        rendered_number,
                        width = width
                    )
                }
            } else {
                if show_grid(options) {
                    write!(output, "{:>width$} │ ", rendered_number, width = width)
                } else {
                    write!(output, "{:>width$} ", rendered_number, width = width)
                }
            }
            .map_err(|error| error.to_string())?;
        }
        let line = if options.show_all {
            visible_whitespace(raw_line)
        } else {
            (*raw_line).to_owned()
        };
        if color {
            writeln!(
                output,
                "{}",
                highlight(
                    &line,
                    options
                        .language
                        .as_deref()
                        .or(options.file_name.as_deref())
                        .unwrap_or(file)
                )
            )
        } else {
            writeln!(output, "{line}")
        }
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn visible_whitespace(line: &str) -> String {
    let mut visible = String::with_capacity(line.len() + 1);
    for character in line.chars() {
        match character {
            '\t' => visible.push_str("→   "),
            ' ' => visible.push('·'),
            _ => visible.push(character),
        }
    }
    visible.push('¶');
    visible
}

fn highlight(line: &str, file: &str) -> String {
    match language(file) {
        Language::Rust => highlight_code(
            line,
            &[
                "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "match", "if",
                "else", "return", "const", "trait",
            ],
        ),
        Language::Toml | Language::Yaml => highlight_config(line),
        Language::Json => highlight_json(line),
        Language::Markdown => highlight_markdown(line),
        Language::Shell => highlight_code(
            line,
            &[
                "if", "then", "fi", "for", "in", "do", "done", "case", "esac", "function", "export",
            ],
        ),
        Language::Plain => line.to_owned(),
    }
}

#[derive(Clone, Copy)]
enum Language {
    Rust,
    Toml,
    Yaml,
    Json,
    Markdown,
    Shell,
    Plain,
}

fn language(file: &str) -> Language {
    let name = file.to_ascii_lowercase();
    match name.as_str() {
        "rust" => return Language::Rust,
        "toml" => return Language::Toml,
        "yaml" | "yml" => return Language::Yaml,
        "json" => return Language::Json,
        "markdown" | "md" => return Language::Markdown,
        "shell" | "sh" | "bash" | "zsh" | "fish" => return Language::Shell,
        "plain" | "text" => return Language::Plain,
        _ => {}
    }
    match Path::new(&name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "rs" => Language::Rust,
        "toml" => Language::Toml,
        "yaml" | "yml" => Language::Yaml,
        "json" => Language::Json,
        "md" | "markdown" => Language::Markdown,
        "sh" | "bash" | "zsh" | "fish" => Language::Shell,
        _ => Language::Plain,
    }
}

fn highlight_code(line: &str, keywords: &[&str]) -> String {
    if let Some((code, comment)) = line.split_once("//") {
        format!(
            "{}{}{}//{}{}",
            color_keywords(code, keywords),
            DIM,
            GREEN,
            comment,
            RESET
        )
    } else if let Some((code, comment)) = line.split_once('#') {
        format!(
            "{}{}{}#{}{}",
            color_keywords(code, keywords),
            DIM,
            GREEN,
            comment,
            RESET
        )
    } else {
        color_keywords(line, keywords)
    }
}

fn color_keywords(line: &str, keywords: &[&str]) -> String {
    let mut result = String::new();
    let mut word = String::new();
    let flush = |word: &mut String, result: &mut String| {
        if keywords.contains(&word.as_str()) {
            result.push_str(BLUE);
            result.push_str(word);
            result.push_str(RESET);
        } else {
            result.push_str(word);
        }
        word.clear();
    };
    for character in line.chars() {
        if character.is_alphanumeric() || character == '_' {
            word.push(character);
        } else {
            flush(&mut word, &mut result);
            result.push(character);
        }
    }
    flush(&mut word, &mut result);
    result
}

fn highlight_config(line: &str) -> String {
    if line.trim_start().starts_with('#') {
        return format!("{DIM}{GREEN}{line}{RESET}");
    }
    if let Some((key, value)) = line.split_once('=') {
        format!("{CYAN}{key}{RESET}={YELLOW}{value}{RESET}")
    } else if let Some((key, value)) = line.split_once(':') {
        format!("{CYAN}{key}{RESET}:{YELLOW}{value}{RESET}")
    } else {
        line.to_owned()
    }
}

fn highlight_json(line: &str) -> String {
    let mut result = line.replace("true", &format!("{MAGENTA}true{RESET}"));
    result = result.replace("false", &format!("{MAGENTA}false{RESET}"));
    result.replace("null", &format!("{MAGENTA}null{RESET}"))
}

fn highlight_markdown(line: &str) -> String {
    if line.starts_with('#') {
        format!("{BOLD}{CYAN}{line}{RESET}")
    } else if line.starts_with("```") {
        format!("{MAGENTA}{line}{RESET}")
    } else {
        line.to_owned()
    }
}

fn print_help() {
    print!(
        "ofat — a fast, syntax-aware cat alternative\n\nUSAGE:\n    ofat [OPTIONS] [FILE]...\n\nWith no FILE, or FILE set to -, read standard input.\n\nOPTIONS:\n    -n, --number          Numbers only (like bat)\n    -p, --plain           Plain output; no decorations or colors\n    -l, --language <LANG> Select syntax explicitly\n    -r, --line-range N:M Print an inclusive line range\n        --style <STYLE>   default, full, header, numbers, plain\n        --file-name <NAME> Name stdin content for display and syntax\n        --paging <WHEN>   Pager: always, auto (default), never\n    -P                    Alias for --paging=never\n    -A, --show-all        Show spaces, tabs and line endings\n    -s, --squeeze-blank   Collapse consecutive blank lines\n        --color <WHEN>    Color: always, auto (default), never\n    -L, --list-languages  Print available language names\n        --list-themes     Print available color themes\n    -h, --help            Print this help\n    -V, --version         Print version\n\nEXAMPLES:\n    ofat src/main.rs\n    ofat --style=full -r 20:60 src/main.rs\n    curl -s https://example.com/code.rs | ofat -l rust -n -\n"
    );
}

fn print_languages() {
    println!(
        "Rust\nTOML\nYAML\nJSON\nMarkdown\nShell\nPlain Text\n\nExtensions: rs, toml, yaml, yml, json, md, markdown, sh, bash, zsh, fish"
    );
}

fn print_themes() {
    println!(
        "default\nmonochrome\n\nofat follows your terminal palette; colors are disabled by --color=never, --plain, NO_COLOR, or redirected output in auto mode."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_regular_options() {
        let options =
            parse_args(vec!["-n".into(), "--color=never".into(), "note.rs".into()]).unwrap();
        assert!(options.numbers);
        assert_eq!(options.color, ColorChoice::Never);
        assert_eq!(options.files, ["note.rs"]);
    }
    #[test]
    fn stops_parsing_after_double_dash() {
        let options = parse_args(vec!["--".into(), "-draft.md".into()]).unwrap();
        assert_eq!(options.files, ["-draft.md"]);
    }
    #[test]
    fn marks_whitespace() {
        assert_eq!(visible_whitespace("a\tb c"), "a→   b·c¶");
    }
    #[test]
    fn detects_common_extensions() {
        assert!(matches!(language("src/main.rs"), Language::Rust));
        assert!(matches!(language("settings.toml"), Language::Toml));
        assert!(matches!(language("input"), Language::Plain));
    }

    #[test]
    fn squeeze_blank_numbers_rendered_lines() {
        let mut output = Vec::new();
        let options = Options {
            numbers: true,
            squeeze_blank: true,
            ..Options::default()
        };
        render(&mut output, "one\n\n\ntwo\n", "note.txt", &options, false).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "1 │ one\n2 │ \n3 │ two\n"
        );
    }
}
