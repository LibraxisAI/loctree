use std::fs;
use std::io;
use std::path::Path;

use super::open_server::url_encode_component;
use super::ReportSection;

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn linkify(base: Option<&str>, file: &str, line: usize) -> String {
    if let Some(base) = base {
        let href = format!("{}/open?f={}&l={}", base, url_encode_component(file), line);
        format!("<a href=\"{}\">{}:{}</a>", href, file, line)
    } else {
        format!("{}:{}", file, line)
    }
}

pub(crate) fn render_html_report(path: &Path, sections: &[ReportSection]) -> io::Result<()> {
    let mut out = String::new();
    out.push_str(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8" />
<title>loctree import/export report</title>
<style>
body{font-family:system-ui,-apple-system,Segoe UI,Helvetica,Arial,sans-serif;margin:24px;line-height:1.5;}
h1,h2,h3{margin-bottom:0.2em;}
table{border-collapse:collapse;width:100%;margin:0.5em 0;}
th,td{border:1px solid #ddd;padding:6px 8px;font-size:14px;}
th{background:#f5f5f5;text-align:left;}
code{background:#f6f8fa;padding:2px 4px;border-radius:4px;}
.muted{color:#666;}
.graph{height:520px;border:1px solid #ddd;border-radius:8px;margin:12px 0;}
</style>
</head><body>
<h1>loctree import/export analysis</h1>
"#,
    );

    for section in sections {
        out.push_str(&format!(
            "<h2>{}</h2><p class=\"muted\">Files analyzed: {}</p>",
            escape_html(&section.root),
            section.files_analyzed
        ));

        // Duplicate exports
        out.push_str("<h3>Top duplicate exports</h3>");
        if section.ranked_dups.is_empty() {
            out.push_str("<p class=\"muted\">None</p>");
        } else {
            out.push_str("<table><tr><th>Symbol</th><th>Files</th><th>Prod</th><th>Dev</th><th>Canonical</th><th>Refactor targets</th></tr>");
            for dup in section.ranked_dups.iter().take(section.analyze_limit) {
                out.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>",
                    escape_html(&dup.name),
                    dup.files.len(),
                    dup.prod_count,
                    dup.dev_count,
                    escape_html(&dup.canonical),
                    escape_html(&dup.refactors.join(", "))
                ));
            }
            out.push_str("</table>");
        }

        // Cascades
        out.push_str("<h3>Re-export cascades</h3>");
        if section.cascades.is_empty() {
            out.push_str("<p class=\"muted\">None</p>");
        } else {
            out.push_str("<ul>");
            for (from, to) in &section.cascades {
                out.push_str(&format!(
                    "<li><code>{}</code> → <code>{}</code></li>",
                    escape_html(from),
                    escape_html(to)
                ));
            }
            out.push_str("</ul>");
        }

        // Dynamic imports
        out.push_str("<h3>Dynamic imports</h3>");
        if section.dynamic.is_empty() {
            out.push_str("<p class=\"muted\">None</p>");
        } else {
            out.push_str("<table><tr><th>File</th><th>Sources</th></tr>");
            for (file, sources) in section.dynamic.iter().take(section.analyze_limit) {
                out.push_str(&format!(
                    "<tr><td><code>{}</code></td><td>{}</td></tr>",
                    escape_html(file),
                    escape_html(&sources.join(", "))
                ));
            }
            out.push_str("</table>");
        }

        // Command coverage
        out.push_str("<h3>Tauri command coverage</h3>");
        if section.missing_handlers.is_empty() && section.unused_handlers.is_empty() {
            out.push_str("<p class=\"muted\">All frontend calls have matching handlers.</p>");
        } else {
            out.push_str("<table><tr><th>Missing handlers (FE→BE)</th><th>Handlers unused by FE</th></tr><tr><td>");
            if section.missing_handlers.is_empty() {
                out.push_str("<span class=\"muted\">None</span>");
            } else {
                let lines: Vec<String> = section
                    .missing_handlers
                    .iter()
                    .map(|g| {
                        let locs: Vec<String> = g
                            .locations
                            .iter()
                            .map(|(f, l)| linkify(section.open_base.as_deref(), f, *l))
                            .collect();
                        format!("{} ({})", g.name, locs.join("; "))
                    })
                    .collect();
                out.push_str(&escape_html(&lines.join(" · ")));
            }
            out.push_str("</td><td>");
            if section.unused_handlers.is_empty() {
                out.push_str("<span class=\"muted\">None</span>");
            } else {
                let lines: Vec<String> = section
                    .unused_handlers
                    .iter()
                    .map(|g| {
                        let locs: Vec<String> = g
                            .locations
                            .iter()
                            .map(|(f, l)| linkify(section.open_base.as_deref(), f, *l))
                            .collect();
                        format!("{} ({})", g.name, locs.join("; "))
                    })
                    .collect();
                out.push_str(&escape_html(&lines.join(" · ")));
            }
            out.push_str("</td></tr></table>");
        }

        if let Some(graph) = &section.graph {
            out.push_str("<h3>Import graph</h3>");
            out.push_str(&format!(
                "<div class=\"graph\" id=\"graph-{}\"></div>",
                escape_html(
                    &section
                        .root
                        .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
                )
            ));
            let nodes_json = serde_json::to_string(&graph.nodes).unwrap_or("[]".into());
            let edges_json = serde_json::to_string(&graph.edges).unwrap_or("[]".into());
            out.push_str("<script>");
            out.push_str("window.__LOCTREE_GRAPHS = window.__LOCTREE_GRAPHS || [];");
            out.push_str("window.__LOCTREE_GRAPHS.push({");
            out.push_str(&format!(
                "id:\"graph-{}\",nodes:{},edges:{}",
                escape_html(
                    &section
                        .root
                        .replace(|c: char| !c.is_ascii_alphanumeric(), "_")
                ),
                nodes_json,
                edges_json
            ));
            out.push_str("});</script>");
        }
    }

    // Graph bootstrap (Cytoscape via CDN)
    out.push_str(
        r#"<script src="https://unpkg.com/cytoscape@3.26.0/dist/cytoscape.min.js"></script>
<script>
(function(){
  const graphs = window.__LOCTREE_GRAPHS || [];
  graphs.forEach(g => {
    const container = document.getElementById(g.id);
    if (!container) return;
    const nodes = Array.from(new Set([].concat(g.nodes || []))).map(n => ({ data: { id: n, label: n }}));
    const edges = (g.edges || []).map((e, idx) => ({
      data: { id: 'e'+idx, source: e[0], target: e[1], label: e[2] }
    }));
    cytoscape({
      container,
      elements: { nodes, edges },
      style: [
        { selector: 'node', style: { 'label': 'data(label)', 'font-size': 10, 'text-wrap': 'wrap', 'text-max-width': 120, 'background-color': '#4f81e1', 'color': '#fff', 'width': 22, 'height': 22 } },
        { selector: 'edge', style: { 'curve-style': 'bezier', 'width': 1.5, 'line-color': '#888', 'target-arrow-color': '#888', 'target-arrow-shape': 'triangle', 'arrow-scale': 0.8, 'label': 'data(label)', 'font-size': 9, 'text-background-color': '#fff', 'text-background-opacity': 0.8, 'text-background-padding': 2 } }
      ],
      layout: { name: 'cose', idealEdgeLength: 120, nodeOverlap: 8, padding: 20 }
    });
  });
})();
</script>"#,
    );

    out.push_str("</body></html>");
    fs::write(path, out)
}
