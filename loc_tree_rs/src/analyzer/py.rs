use std::collections::HashSet;
use std::path::Path;

use crate::types::{
    ExportSymbol, FileAnalysis, ImportEntry, ImportKind, ReexportEntry, ReexportKind,
};

use super::regexes::{
    regex_py_all, regex_py_class, regex_py_def, regex_py_dynamic_dunder, regex_py_dynamic_importlib,
};
use super::resolvers::resolve_python_relative;

pub(crate) fn analyze_py_file(
    content: &str,
    path: &Path,
    root: &Path,
    extensions: Option<&HashSet<String>>,
    relative: String,
) -> FileAnalysis {
    let mut imports = Vec::new();
    let mut reexports = Vec::new();
    let mut dynamic_imports = Vec::new();
    let mut exports = Vec::new();

    for line in content.lines() {
        let without_comment = line.split('#').next().unwrap_or("").trim_end();
        let trimmed = without_comment.trim_start();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            for part in rest.split(',') {
                let mut name = part.trim();
                if let Some((lhs, _)) = name.split_once(" as ") {
                    name = lhs.trim();
                }
                if !name.is_empty() {
                    imports.push(ImportEntry {
                        source: name.to_string(),
                        kind: ImportKind::Static,
                    });
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("from ") {
            if let Some((module, names_raw)) = rest.split_once(" import ") {
                let module = module.trim().trim_end_matches('.');
                let names_clean = names_raw.trim().trim_matches('(').trim_matches(')');
                let names_clean = names_clean.split('#').next().unwrap_or("").trim();
                if !module.is_empty() {
                    imports.push(ImportEntry {
                        source: module.to_string(),
                        kind: ImportKind::Static,
                    });
                }
                if names_clean == "*" {
                    let resolved = resolve_python_relative(module, path, root, extensions);
                    reexports.push(ReexportEntry {
                        source: module.to_string(),
                        kind: ReexportKind::Star,
                        resolved,
                    });
                }
            }
        }
    }

    for caps in regex_py_dynamic_importlib().captures_iter(content) {
        if let Some(m) = caps.get(1) {
            dynamic_imports.push(m.as_str().to_string());
        }
    }
    for caps in regex_py_dynamic_dunder().captures_iter(content) {
        if let Some(m) = caps.get(1) {
            dynamic_imports.push(m.as_str().to_string());
        }
    }

    for caps in regex_py_all().captures_iter(content) {
        let body = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        for item in body.split(',') {
            let trimmed = item.trim();
            let name = trimmed
                .trim_matches(|c| c == '\'' || c == '"')
                .trim()
                .to_string();
            if !name.is_empty() {
                exports.push(ExportSymbol {
                    name,
                    kind: "__all__".to_string(),
                });
            }
        }
    }

    for caps in regex_py_def().captures_iter(content) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                exports.push(ExportSymbol {
                    name: n.to_string(),
                    kind: "def".to_string(),
                });
            }
        }
    }
    for caps in regex_py_class().captures_iter(content) {
        if let Some(name) = caps.get(1) {
            let n = name.as_str();
            if !n.starts_with('_') {
                exports.push(ExportSymbol {
                    name: n.to_string(),
                    kind: "class".to_string(),
                });
            }
        }
    }

    FileAnalysis {
        path: relative,
        imports,
        reexports,
        dynamic_imports,
        exports,
        command_calls: Vec::new(),
        command_handlers: Vec::new(),
    }
}
