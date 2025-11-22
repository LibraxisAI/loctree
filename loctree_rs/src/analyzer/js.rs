use std::collections::HashSet;
use std::path::Path;

use crate::types::{
    CommandRef, ExportSymbol, FileAnalysis, ImportEntry, ImportKind, ReexportEntry, ReexportKind,
};

use super::regexes::{
    regex_dynamic_import, regex_export_brace, regex_export_default, regex_export_named_decl,
    regex_import, regex_invoke_audio, regex_invoke_snake, regex_reexport_named,
    regex_reexport_star, regex_safe_invoke, regex_side_effect_import, regex_tauri_invoke,
};
use super::resolvers::resolve_reexport_target;
use super::{brace_list_to_names, offset_to_line};

pub(crate) fn analyze_js_file(
    content: &str,
    path: &Path,
    root: &Path,
    extensions: Option<&HashSet<String>>,
    relative: String,
) -> FileAnalysis {
    let mut imports = Vec::new();
    let mut command_calls = Vec::new();
    for caps in regex_import().captures_iter(content) {
        let source = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        imports.push(ImportEntry {
            source,
            kind: ImportKind::Static,
        });
    }
    for caps in regex_side_effect_import().captures_iter(content) {
        let source = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        imports.push(ImportEntry {
            source,
            kind: ImportKind::SideEffect,
        });
    }

    for caps in regex_safe_invoke().captures_iter(content) {
        if let Some(cmd) = caps.get(1) {
            let line = offset_to_line(content, cmd.start());
            command_calls.push(CommandRef {
                name: cmd.as_str().to_string(),
                exposed_name: None,
                line,
            });
        }
    }
    for caps in regex_tauri_invoke().captures_iter(content) {
        if let Some(cmd) = caps.get(1) {
            let line = offset_to_line(content, cmd.start());
            command_calls.push(CommandRef {
                name: cmd.as_str().to_string(),
                exposed_name: None,
                line,
            });
        }
    }
    for caps in regex_invoke_audio().captures_iter(content) {
        if let Some(cmd) = caps.get(1) {
            let line = offset_to_line(content, cmd.start());
            command_calls.push(CommandRef {
                name: cmd.as_str().to_string(),
                exposed_name: None,
                line,
            });
        }
    }
    for caps in regex_invoke_snake().captures_iter(content) {
        if let Some(cmd) = caps.get(1) {
            let line = offset_to_line(content, cmd.start());
            command_calls.push(CommandRef {
                name: cmd.as_str().to_string(),
                exposed_name: None,
                line,
            });
        }
    }

    let mut reexports = Vec::new();
    for caps in regex_reexport_star().captures_iter(content) {
        let source = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let resolved = resolve_reexport_target(path, root, &source, extensions);
        reexports.push(ReexportEntry {
            source,
            kind: ReexportKind::Star,
            resolved,
        });
    }
    for caps in regex_reexport_named().captures_iter(content) {
        let raw_names = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let source = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
        let names = brace_list_to_names(raw_names);
        let resolved = resolve_reexport_target(path, root, &source, extensions);
        reexports.push(ReexportEntry {
            source,
            kind: ReexportKind::Named(names.clone()),
            resolved,
        });
    }

    let mut dynamic_imports = Vec::new();
    for caps in regex_dynamic_import().captures_iter(content) {
        let source = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        dynamic_imports.push(source);
    }

    let mut exports = Vec::new();
    for caps in regex_export_named_decl().captures_iter(content) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        if !name.is_empty() {
            exports.push(ExportSymbol {
                name,
                kind: "decl".to_string(),
            });
        }
    }
    for caps in regex_export_default().captures_iter(content) {
        let name = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "default".to_string());
        exports.push(ExportSymbol {
            name,
            kind: "default".to_string(),
        });
    }
    for caps in regex_export_brace().captures_iter(content) {
        let raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        for name in brace_list_to_names(raw) {
            exports.push(ExportSymbol {
                name,
                kind: "named".to_string(),
            });
        }
    }
    for re in &reexports {
        if let ReexportKind::Named(names) = &re.kind {
            for name in names {
                exports.push(ExportSymbol {
                    name: name.clone(),
                    kind: "reexport".to_string(),
                });
            }
        }
    }

    FileAnalysis {
        path: relative,
        loc: 0,
        imports,
        reexports,
        dynamic_imports,
        exports,
        command_calls,
        command_handlers: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_js_file;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn detects_commands_reexports_and_exports() {
        let content = r#"
import defaultThing from "./dep";
import type { Foo } from "./types";
import "./side.css";
export { bar } from "./reexports";
export * from "./star";
export const localValue = 1;
export default function MyComp() {}
export { namedA, namedB as aliasB };
const dyn = import("./lazy");
safeInvoke("cmd_safe");
invokeSnake("cmd_snake");
invoke("cmd_invoke");
safeInvoke<Foo.Bar>("cmd_generic_safe");
invokeSnake<MyType>("cmd_generic_snake");
invoke<Inline<Ok>>("cmd_generic_invoke");
invokeAudioCamel<Baz>("cmd_audio_generic");
        "#;

        let analysis = analyze_js_file(
            content,
            Path::new("src/app.tsx"),
            Path::new("src"),
            Some(&HashSet::from(["ts".to_string(), "tsx".to_string()])),
            "app.tsx".to_string(),
        );

        assert!(analysis
            .imports
            .iter()
            .any(|i| i.source == "./dep" && matches!(i.kind, crate::types::ImportKind::Static)));
        assert!(analysis
            .imports
            .iter()
            .any(|i| i.source == "./side.css"
                && matches!(i.kind, crate::types::ImportKind::SideEffect)));
        assert!(analysis.reexports.iter().any(|r| r.source == "./reexports"));
        assert!(analysis.reexports.iter().any(|r| r.source == "./star"));
        assert!(analysis.dynamic_imports.iter().any(|s| s == "./lazy"));

        let commands: Vec<_> = analysis
            .command_calls
            .iter()
            .map(|c| c.name.clone())
            .collect();
        assert!(commands.contains(&"cmd_safe".to_string()));
        assert!(commands.contains(&"cmd_snake".to_string()));
        assert!(commands.contains(&"cmd_invoke".to_string()));
        assert!(commands.contains(&"cmd_generic_safe".to_string()));
        assert!(commands.contains(&"cmd_generic_snake".to_string()));
        assert!(commands.contains(&"cmd_generic_invoke".to_string()));
        assert!(commands.contains(&"cmd_audio_generic".to_string()));

        // exports should include defaults and named
        let export_names: Vec<_> = analysis.exports.iter().map(|e| e.name.clone()).collect();
        assert!(export_names.contains(&"localValue".to_string()));
        assert!(export_names.contains(&"MyComp".to_string()));
        assert!(export_names.contains(&"namedA".to_string()));
    }
}
