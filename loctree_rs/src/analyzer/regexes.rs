use std::sync::OnceLock;

use regex::Regex;

pub(crate) fn regex_import() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*import\s+([^;]+?)\s+from\s+["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_side_effect_import() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*import\s+["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_reexport_star() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*export\s+\*\s+from\s+["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_reexport_named() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*export\s+\{([^}]+)\}\s+from\s+["']([^"']+)["']"#).unwrap()
    })
}

pub(crate) fn regex_dynamic_import() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap())
}

pub(crate) fn regex_export_named_decl() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?m)^\s*export\s+(?:async\s+)?(?:function|const|let|var|class|interface|type|enum)\s+([A-Za-z0-9_.$]+)"#,
        )
        .unwrap()
    })
}

pub(crate) fn regex_export_default() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*export\s+default(?:\s+(?:async\s+)?(?:function|class)\s+([A-Za-z0-9_.$]+))?"#).unwrap())
}

pub(crate) fn regex_export_brace() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*export\s+\{([^}]+)\}\s*;?"#).unwrap())
}

pub(crate) fn regex_safe_invoke() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"safeInvoke\s*(?:<[^)]*>)?\(\s*["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_invoke_snake() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"invokeSnake\s*(?:<[^)]*>)?\(\s*["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_invoke_audio() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // capture invokeAudio(...) and invokeAudioCamel(...) helpers used by FE audio API
    RE.get_or_init(|| {
        Regex::new(r#"invokeAudio(?:Camel)?\s*(?:<[^)]*>)?\(\s*["']([^"']+)["']"#).unwrap()
    })
}

pub(crate) fn regex_tauri_command_fn() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)#\s*\[\s*tauri::command([^\]]*)\]\s*(?:pub\s*(?:\([^)]*\)\s*)?)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)"#)
            .unwrap()
    })
}

pub(crate) fn regex_tauri_invoke() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Matches top-level invoke("cmd") calls (avoids foo.invoke())
        Regex::new(r#"(?m)(?:^|[^A-Za-z0-9_\.])invoke\s*(?:<[^)]*>)?\(\s*[\"']([^\"']+)[\"']"#)
            .unwrap()
    })
}

pub(crate) fn regex_css_import() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // @import "x.css";  @import url("x.css"); @import url(x.css);
        Regex::new(r#"(?m)@import\s+(?:url\()?['"]?([^"'()\s]+)['"]?\)?"#).unwrap()
    })
}

pub(crate) fn regex_rust_use() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*(?:pub\s*(?:\([^)]*\))?\s+)?use\s+([^;]+);"#).unwrap())
}

pub(crate) fn regex_rust_pub_use() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*pub\s*(?:\([^)]*\))?\s+use\s+([^;]+);"#).unwrap())
}

pub(crate) fn regex_rust_pub_item(kind: &str) -> Regex {
    // Matches visibility modifiers like pub(crate) and optional async for fn
    let pattern = format!(
        r#"(?m)^\s*pub\s*(?:\([^)]*\)\s*)?(?:async\s+)?{}\s+([A-Za-z0-9_]+)"#,
        kind
    );
    Regex::new(&pattern).unwrap()
}

pub(crate) fn regex_rust_pub_const_like(kind: &str) -> Regex {
    let pattern = format!(
        r#"(?m)^\s*pub\s*(?:\([^)]*\)\s*)?{}\s+([A-Za-z0-9_]+)"#,
        kind
    );
    Regex::new(&pattern).unwrap()
}

pub(crate) fn rust_pub_decl_regexes() -> &'static [Regex] {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        vec![
            regex_rust_pub_item("fn"),
            regex_rust_pub_item("struct"),
            regex_rust_pub_item("enum"),
            regex_rust_pub_item("trait"),
            regex_rust_pub_item("type"),
            regex_rust_pub_item("union"),
            regex_rust_pub_item("mod"),
        ]
    })
    .as_slice()
}

pub(crate) fn rust_pub_const_regexes() -> &'static [Regex] {
    static RE: OnceLock<Vec<Regex>> = OnceLock::new();
    RE.get_or_init(|| {
        vec![
            regex_rust_pub_const_like("const"),
            regex_rust_pub_const_like("static"),
        ]
    })
    .as_slice()
}

pub(crate) fn regex_py_dynamic_importlib() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"importlib\.import_module\(\s*["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_py_dynamic_dunder() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"__import__\(\s*["']([^"']+)["']"#).unwrap())
}

pub(crate) fn regex_py_all() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?s)__all__\s*=\s*\[([^\]]*)\]"#).unwrap())
}

pub(crate) fn regex_py_def() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)"#).unwrap())
}

pub(crate) fn regex_py_class() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)"#).unwrap())
}
