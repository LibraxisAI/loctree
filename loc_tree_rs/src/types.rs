use std::collections::{HashMap, HashSet};

pub const DEFAULT_LOC_THRESHOLD: usize = 1000;
pub const COLOR_RED: &str = "\u{001b}[31m";
pub const COLOR_RESET: &str = "\u{001b}[0m";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
    Jsonl,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Tree,
    AnalyzeImports,
}

#[derive(Clone)]
pub struct Options {
    pub extensions: Option<HashSet<String>>,
    pub ignore_paths: Vec<std::path::PathBuf>,
    pub use_gitignore: bool,
    pub max_depth: Option<usize>,
    pub color: ColorMode,
    pub output: OutputMode,
    pub summary: bool,
    pub summary_limit: usize,
    pub show_hidden: bool,
    pub loc_threshold: usize,
    pub analyze_limit: usize,
    pub report_path: Option<std::path::PathBuf>,
}

pub struct LineEntry {
    pub label: String,
    pub loc: Option<usize>,
    pub relative_path: String,
    pub is_dir: bool,
    pub is_large: bool,
}

pub struct LargeEntry {
    pub path: String,
    pub loc: usize,
}

#[derive(Default)]
pub struct Stats {
    pub directories: usize,
    pub files: usize,
    pub files_with_loc: usize,
    pub total_loc: usize,
}

pub struct Collectors<'a> {
    pub entries: &'a mut Vec<LineEntry>,
    pub large_entries: &'a mut Vec<LargeEntry>,
    pub stats: &'a mut Stats,
}

#[derive(Clone)]
pub struct ImportEntry {
    pub source: String,
    pub kind: ImportKind,
}

#[derive(Clone)]
pub enum ImportKind {
    Static,
    SideEffect,
}

#[derive(Clone)]
pub struct ReexportEntry {
    pub source: String,
    pub kind: ReexportKind,
    pub resolved: Option<String>,
}

#[derive(Clone)]
pub enum ReexportKind {
    Star,
    Named(Vec<String>),
}

#[derive(Clone)]
pub struct ExportSymbol {
    pub name: String,
    pub kind: String,
}

#[derive(Clone)]
pub struct FileAnalysis {
    pub path: String,
    pub imports: Vec<ImportEntry>,
    pub reexports: Vec<ReexportEntry>,
    pub dynamic_imports: Vec<String>,
    pub exports: Vec<ExportSymbol>,
}

// Convenience type aliases reused across modules
pub type ExportIndex = HashMap<String, Vec<String>>;
