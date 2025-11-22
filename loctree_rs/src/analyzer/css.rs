use crate::types::{FileAnalysis, ImportEntry, ImportKind};

use super::regexes::regex_css_import;

pub(crate) fn analyze_css_file(content: &str, relative: String) -> FileAnalysis {
    let mut imports = Vec::new();
    for caps in regex_css_import().captures_iter(content) {
        let source = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        imports.push(ImportEntry {
            source,
            kind: ImportKind::Static,
        });
    }

    FileAnalysis {
        path: relative,
        loc: 0,
        imports,
        reexports: Vec::new(),
        dynamic_imports: Vec::new(),
        exports: Vec::new(),
        command_calls: Vec::new(),
        command_handlers: Vec::new(),
    }
}
