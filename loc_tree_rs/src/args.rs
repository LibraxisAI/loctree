use std::collections::HashSet;
use std::path::PathBuf;

use crate::types::{ColorMode, Mode, OutputMode, DEFAULT_LOC_THRESHOLD};

pub struct ParsedArgs {
    pub extensions: Option<HashSet<String>>,
    pub ignore_patterns: Vec<String>,
    pub use_gitignore: bool,
    pub max_depth: Option<usize>,
    pub color: ColorMode,
    pub output: OutputMode,
    pub summary: bool,
    pub summary_limit: usize,
    pub show_help: bool,
    pub show_version: bool,
    pub root_list: Vec<PathBuf>,
    pub show_hidden: bool,
    pub loc_threshold: usize,
    pub mode: Mode,
    pub analyze_limit: usize,
    pub report_path: Option<PathBuf>,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            extensions: None,
            ignore_patterns: Vec::new(),
            use_gitignore: false,
            max_depth: None,
            color: ColorMode::Auto,
            output: OutputMode::Human,
            summary: false,
            summary_limit: 5,
            show_help: false,
            show_version: false,
            root_list: Vec::new(),
            show_hidden: false,
            loc_threshold: DEFAULT_LOC_THRESHOLD,
            mode: Mode::Tree,
            analyze_limit: 8,
            report_path: None,
        }
    }
}

fn parse_color_mode(raw: &str) -> Result<ColorMode, String> {
    match raw {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        _ => Err("--color expects auto|always|never".to_string()),
    }
}

fn parse_summary_limit(raw: &str) -> Result<usize, String> {
    let value = raw
        .parse::<usize>()
        .map_err(|_| "--summary expects a positive integer".to_string())?;
    if value == 0 {
        Err("--summary expects a positive integer".to_string())
    } else {
        Ok(value)
    }
}

pub fn parse_extensions(raw: &str) -> Option<HashSet<String>> {
    let set: HashSet<String> = raw
        .split(',')
        .filter_map(|segment| {
            let trimmed = segment.trim().trim_start_matches('.').to_lowercase();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect();
    if set.is_empty() {
        None
    } else {
        Some(set)
    }
}

pub fn parse_args() -> Result<ParsedArgs, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut parsed = ParsedArgs {
        ..ParsedArgs::default()
    };

    let mut roots: Vec<PathBuf> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--help" | "-h" => {
                parsed.show_help = true;
                i += 1;
            }
            "--version" | "-V" => {
                parsed.show_version = true;
                i += 1;
            }
            "--color" | "-c" => {
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        parsed.color = parse_color_mode(next)?;
                        i += 2;
                        continue;
                    }
                }
                parsed.color = ColorMode::Always;
                i += 1;
            }
            _ if arg.starts_with("--color=") => {
                let value = arg.trim_start_matches("--color=");
                parsed.color = parse_color_mode(value)?;
                i += 1;
            }
            "--gitignore" | "-g" => {
                parsed.use_gitignore = true;
                i += 1;
            }
            "--show-hidden" | "-H" => {
                parsed.show_hidden = true;
                i += 1;
            }
            "--json" => {
                parsed.output = OutputMode::Json;
                i += 1;
            }
            "--jsonl" => {
                parsed.output = OutputMode::Jsonl;
                i += 1;
            }
            "--html-report" | "--report" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "--html-report requires a file path".to_string())?;
                parsed.report_path = Some(PathBuf::from(next));
                i += 2;
            }
            "--summary" => {
                parsed.summary = true;
                if let Some(next) = args.get(i + 1) {
                    if !next.starts_with('-') {
                        parsed.summary_limit = parse_summary_limit(next)?;
                        i += 2;
                        continue;
                    }
                }
                i += 1;
            }
            _ if arg.starts_with("--summary=") => {
                let value = arg.trim_start_matches("--summary=");
                parsed.summary = true;
                parsed.summary_limit = parse_summary_limit(value)?;
                i += 1;
            }
            "--loc" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "--loc requires a positive integer".to_string())?;
                let value = next
                    .parse::<usize>()
                    .map_err(|_| "--loc requires a positive integer".to_string())?;
                if value == 0 {
                    return Err("--loc requires a positive integer".to_string());
                }
                parsed.loc_threshold = value;
                i += 2;
            }
            "--limit" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "--limit requires a positive integer".to_string())?;
                let value = next
                    .parse::<usize>()
                    .map_err(|_| "--limit requires a positive integer".to_string())?;
                if value == 0 {
                    return Err("--limit requires a positive integer".to_string());
                }
                parsed.analyze_limit = value;
                i += 2;
            }
            "--analyze-imports" | "-A" => {
                parsed.mode = Mode::AnalyzeImports;
                i += 1;
            }
            "-L" | "--max-depth" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "-L/--max-depth requires a non-negative integer".to_string())?;
                let depth = next
                    .parse::<usize>()
                    .map_err(|_| "-L/--max-depth requires a non-negative integer".to_string())?;
                parsed.max_depth = Some(depth);
                i += 2;
            }
            "--ext" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "--ext requires a comma-separated value".to_string())?;
                parsed.extensions = parse_extensions(next);
                i += 2;
            }
            _ if arg.starts_with("--ext=") => {
                let value = arg.trim_start_matches("--ext=");
                parsed.extensions = parse_extensions(value);
                i += 1;
            }
            "-I" | "--ignore" => {
                let next = args
                    .get(i + 1)
                    .ok_or_else(|| "-I/--ignore requires a path argument".to_string())?;
                parsed.ignore_patterns.push(next.clone());
                i += 2;
            }
            _ if arg.starts_with('-') => {
                eprintln!("Ignoring unknown flag {}", arg);
                i += 1;
            }
            _ => {
                let trimmed = arg.trim();
                if !trimmed.is_empty() {
                    roots.push(PathBuf::from(trimmed));
                }
                i += 1;
            }
        }
    }

    if roots.is_empty() {
        roots.push(PathBuf::from("."));
    }
    parsed.root_list = roots;

    Ok(parsed)
}
