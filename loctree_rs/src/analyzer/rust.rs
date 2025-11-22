use crate::types::{
    CommandRef, ExportSymbol, FileAnalysis, ImportEntry, ImportKind, ReexportEntry, ReexportKind,
};

use super::offset_to_line;
use super::regexes::{
    regex_rust_pub_use, regex_rust_use, regex_tauri_command_fn, rust_pub_const_regexes,
    rust_pub_decl_regexes,
};

fn parse_rust_brace_names(raw: &str) -> Vec<String> {
    raw.split(',')
        .filter_map(|item| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed == "self" {
                return None;
            }
            if let Some((_, alias)) = trimmed.split_once(" as ") {
                Some(alias.trim().to_string())
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

pub(crate) fn analyze_rust_file(content: &str, relative: String) -> FileAnalysis {
    let mut imports = Vec::new();
    for caps in regex_rust_use().captures_iter(content) {
        let source = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if !source.is_empty() {
            imports.push(ImportEntry {
                source: source.to_string(),
                kind: ImportKind::Static,
            });
        }
    }

    let mut reexports = Vec::new();
    let mut exports = Vec::new();

    for caps in regex_rust_pub_use().captures_iter(content) {
        let raw = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        if raw.is_empty() {
            continue;
        }

        if raw.contains('{') && raw.contains('}') {
            let mut parts = raw.splitn(2, '{');
            let _prefix = parts.next().unwrap_or("").trim().trim_end_matches("::");
            let braces = parts.next().unwrap_or("").trim_end_matches('}').trim();
            let names = parse_rust_brace_names(braces);
            reexports.push(ReexportEntry {
                source: raw.to_string(),
                kind: ReexportKind::Named(names.clone()),
                resolved: None,
            });
            for name in names {
                exports.push(ExportSymbol {
                    name,
                    kind: "reexport".to_string(),
                });
            }
        } else if raw.ends_with("::*") {
            reexports.push(ReexportEntry {
                source: raw.to_string(),
                kind: ReexportKind::Star,
                resolved: None,
            });
        } else {
            // pub use foo::bar as Baz;
            let (path_part, export_name) = if let Some((path, alias)) = raw.split_once(" as ") {
                (path.trim(), alias.trim())
            } else {
                let mut segments = raw.rsplitn(2, "::");
                let name = segments.next().unwrap_or(raw).trim();
                let _ = segments.next();
                (raw, name)
            };

            reexports.push(ReexportEntry {
                source: path_part.to_string(),
                kind: ReexportKind::Named(vec![export_name.to_string()]),
                resolved: None,
            });
            exports.push(ExportSymbol {
                name: export_name.to_string(),
                kind: "reexport".to_string(),
            });
        }
    }

    // public items
    for regex in rust_pub_decl_regexes() {
        for caps in regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                exports.push(ExportSymbol {
                    name: name.as_str().to_string(),
                    kind: "decl".to_string(),
                });
            }
        }
    }

    for regex in rust_pub_const_regexes() {
        for caps in regex.captures_iter(content) {
            if let Some(name) = caps.get(1) {
                exports.push(ExportSymbol {
                    name: name.as_str().to_string(),
                    kind: "decl".to_string(),
                });
            }
        }
    }

    let mut command_handlers = Vec::new();
    for caps in regex_tauri_command_fn().captures_iter(content) {
        if let Some(name) = caps.get(1) {
            let line = offset_to_line(content, name.start());
            command_handlers.push(CommandRef {
                name: name.as_str().to_string(),
                line,
            });
        }
    }

    FileAnalysis {
        path: relative,
        loc: 0,
        imports,
        reexports,
        dynamic_imports: Vec::new(),
        exports,
        command_calls: Vec::new(),
        command_handlers,
    }
}
