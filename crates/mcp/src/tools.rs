//! MCP tools: names, schemas and implementations on top of a [`Backend`].

use std::path::Path;

use serde_json::{Value, json};

use crate::backend::Backend;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolResult {
    pub content: Vec<Value>,
    pub is_error: bool,
}

impl ToolResult {
    pub fn text(t: impl Into<String>) -> Self {
        Self { content: vec![json!({"type": "text", "text": t.into()})], is_error: false }
    }
    pub fn json(v: &Value) -> Self {
        Self::text(serde_json::to_string_pretty(v).unwrap_or_default())
    }
    pub fn error(t: impl Into<String>) -> Self {
        Self { content: vec![json!({"type": "text", "text": t.into()})], is_error: true }
    }
    pub fn image(png_base64: String, info: &Value) -> Self {
        Self {
            content: vec![json!({"type": "image", "data": png_base64, "mimeType": "image/png"}), json!({"type": "text", "text": info.to_string()})],
            is_error: false,
        }
    }
    pub fn to_value(&self) -> Value {
        json!({"content": self.content, "isError": self.is_error})
    }
}

fn s(desc: &str) -> Value {
    json!({"type": "string", "description": desc})
}
fn n(desc: &str) -> Value {
    json!({"type": "number", "description": desc})
}
fn obj(props: Value, req: &[&str]) -> Value {
    json!({"type": "object", "properties": props, "required": req})
}
fn tool(name: &str, title: &str, desc: &str, schema: Value, ro: bool) -> Value {
    json!({"name": name, "title": title, "description": desc, "inputSchema": schema, "annotations": {"title": title, "readOnlyHint": ro, "openWorldHint": false}})
}

pub fn tool_definitions() -> Value {
    json!([
        tool(
            "list_commands",
            "List commands",
            "Every GoharScribe command: id, label, ribbon location, shortcut and params. Filter with `query`.",
            obj(json!({"query": s("Case-insensitive filter on id/label/location")}), &[]),
            true
        ),
        tool(
            "execute",
            "Run a command",
            "Run one command by id with params, e.g. {command:\"format.bold\"} or {command:\"insert.table\", params:{rows:3, cols:4}}.",
            obj(json!({"command": s("Command id"), "params": {"type": "object", "description": "Command parameters"}}), &["command"]),
            false
        ),
        tool(
            "batch",
            "Run several commands",
            "Run commands in order; stops at the first error unless keepGoing.",
            obj(
                json!({"commands": {"type": "array", "items": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}, "keepGoing": {"type": "boolean"}}),
                &["commands"]
            ),
            false
        ),
        tool(
            "new_document",
            "New document",
            "Start a document: blank, sample, letter, resume or report.",
            obj(json!({"template": s("blank|sample|letter|resume|report")}), &[]),
            false
        ),
        tool(
            "open_document",
            "Open",
            "Open a .docx, .odt, .rtf, .md, .html, .txt or .json file.",
            obj(json!({"path": s("File path")}), &["path"]),
            false
        ),
        tool(
            "save_document",
            "Save",
            "Save (format from the extension: docx, pdf, odt, rtf, html, md, txt, png).",
            obj(json!({"path": s("File path; omit to save in place")}), &[]),
            false
        ),
        tool(
            "type_text",
            "Type text",
            "Type at the caret (replacing the selection). Use \\n... for line breaks inside text; use execute text.newParagraph for new paragraphs, or `paragraphs: true` to split lines into paragraphs.",
            obj(json!({"text": s("Text"), "paragraphs": {"type": "boolean"}}), &["text"]),
            false
        ),
        tool(
            "select_text",
            "Select text",
            "Select the n-th occurrence of some text.",
            obj(json!({"text": s("Text to find"), "occurrence": n("1-based")}), &["text"]),
            false
        ),
        tool("get_text", "Document text", "Plain text of the document body.", obj(json!({}), &[]), true),
        tool(
            "inspect_document",
            "Inspect document",
            "Structure: blocks with text, styles, formatting runs, lists, tables, sections, parts, selection, pages.",
            obj(json!({}), &[]),
            true
        ),
        tool(
            "render_page",
            "Render page",
            "Render a page to PNG to look at the result.",
            obj(json!({"page": n("1-based page"), "scale": n("Pixels per point (default 1)")}), &[]),
            true
        ),
        tool("parity", "Feature parity", "GoharScribe's command coverage of the word-processor feature catalog.", obj(json!({}), &[]), true),
        tool("screenshot", "Screenshot app", "Screenshot of the whole GoharScribe window (desktop app only).", obj(json!({}), &[]), true),
        tool(
            "click",
            "Click",
            "Click at window coordinates (desktop app only).",
            obj(json!({"x": n("x"), "y": n("y"), "count": n("1-3 clicks"), "shift": {"type": "boolean"}, "cmd": {"type": "boolean"}}), &["x", "y"]),
            false
        ),
        tool(
            "key",
            "Press key",
            "Press a key with modifiers (desktop app only), e.g. {key:\"B\", cmd:true}.",
            obj(json!({"key": s("Key name"), "shift": {"type": "boolean"}, "alt": {"type": "boolean"}, "cmd": {"type": "boolean"}}), &["key"]),
            false
        ),
        tool(
            "ui_inspect",
            "Inspect UI",
            "UI state: tab, panes, dialog, page rectangles on screen, caret (desktop app only).",
            obj(json!({}), &[]),
            true
        ),
    ])
}

fn exec(b: &mut dyn Backend, id: &str, params: Value) -> Result<Value, String> {
    b.call("engine.execute", json!({"command": id, "params": params}))
}

fn wrap(r: Result<Value, String>) -> ToolResult {
    match r {
        Ok(v) => ToolResult::json(&v),
        Err(e) => ToolResult::error(e),
    }
}

/// Commands whose `path` parameter touches the filesystem.
pub(crate) const PATH_COMMANDS: &[&str] = &[
    "file.open",
    "file.save",
    "file.saveAs",
    "file.exportPdf",
    "file.exportPng",
    "file.newFromTemplate",
    "file.saveTemplate",
    "file.recover",
    // H2: Additional path-taking commands (MCP jail bypass fix).
    "insert.textFromFile",
    "insert.picture",
    "picture.change",
    "review.compare",
    "review.combine",
    "mailings.recipients",
    "mailings.finish",
];

/// Reject a file command whose `path` escapes the jail. No jail → always Ok.
fn jail_check(jail: Option<&Path>, command: &str, params: &Value) -> Result<(), String> {
    let Some(jail) = jail else { return Ok(()) };
    if !PATH_COMMANDS.contains(&command) {
        return Ok(());
    }
    let Some(path) = params.get("path").and_then(Value::as_str).filter(|p| !p.is_empty()) else {
        return Ok(()); // e.g. save-in-place: no new path touched
    };
    if jail_allows(jail, path) { Ok(()) } else { Err(format!("path `{path}` is outside the allowed directory")) }
}

/// True when `path` resolves inside `jail`. Relative paths are resolved against
/// the jail root; `..` and symlinks can't escape (both sides canonicalized).
pub(crate) fn jail_allows(jail: &Path, path: &str) -> bool {
    let p = Path::new(path);
    let abs = if p.is_absolute() { p.to_path_buf() } else { jail.join(p) };
    let target = if abs.exists() {
        abs.canonicalize().unwrap_or(abs)
    } else {
        // New file: canonicalize the parent, then reattach the file name.
        match abs.parent().map(|par| par.canonicalize()) {
            Some(Ok(par)) => par.join(abs.file_name().unwrap_or_default()),
            _ => abs,
        }
    };
    target.starts_with(jail)
}

pub fn call_tool(b: &mut dyn Backend, name: &str, a: &Value, jail: Option<&Path>) -> ToolResult {
    let st = |k: &str| a.get(k).and_then(Value::as_str);
    // Enforce the filesystem jail on every path a tool would touch.
    let check = |command: &str, params: &Value| -> Result<(), String> { jail_check(jail, command, params) };
    match name {
        "list_commands" => {
            let q = st("query").unwrap_or("").to_lowercase();
            match b.call("engine.commands", json!({})) {
                Ok(Value::Array(v)) => {
                    let f: Vec<Value> = v
                        .into_iter()
                        .filter(|c| {
                            q.is_empty()
                                || ["id", "label", "location"]
                                    .iter()
                                    .any(|k| c.get(*k).and_then(Value::as_str).is_some_and(|x| x.to_lowercase().contains(&q)))
                        })
                        .collect();
                    ToolResult::json(&Value::Array(f))
                }
                Ok(v) => ToolResult::json(&v),
                Err(e) => ToolResult::error(e),
            }
        }
        "execute" => match st("command") {
            Some(c) => {
                let params = a.get("params").cloned().unwrap_or(json!({}));
                if let Err(e) = check(c, &params) {
                    return ToolResult::error(e);
                }
                wrap(exec(b, c, params))
            }
            None => ToolResult::error("missing `command`"),
        },
        "batch" => {
            let keep = a.get("keepGoing").and_then(Value::as_bool).unwrap_or(false);
            let mut out = Vec::new();
            for c in a.get("commands").and_then(Value::as_array).cloned().unwrap_or_default() {
                let id = c.get("command").and_then(Value::as_str).unwrap_or("");
                let params = c.get("params").cloned().unwrap_or(json!({}));
                let r = match check(id, &params) {
                    Err(e) => Err(e),
                    Ok(()) => exec(b, id, params),
                };
                let failed = r.is_err();
                out.push(match r {
                    Ok(v) => json!({"command": id, "ok": true, "result": v}),
                    Err(e) => json!({"command": id, "ok": false, "error": e}),
                });
                if failed && !keep {
                    return ToolResult { content: vec![json!({"type": "text", "text": Value::Array(out).to_string()})], is_error: true };
                }
            }
            ToolResult::json(&Value::Array(out))
        }
        "new_document" => wrap(exec(b, "file.new", json!({"template": st("template").unwrap_or("blank")}))),
        "open_document" => {
            let params = json!({"path": st("path").unwrap_or("")});
            if let Err(e) = check("file.open", &params) {
                return ToolResult::error(e);
            }
            wrap(exec(b, "file.open", params))
        }
        "save_document" => {
            let p = st("path");
            let ext = p.and_then(|x| x.rsplit('.').next()).unwrap_or("").to_ascii_lowercase();
            if ext == "png" {
                let params = json!({"path": p});
                if let Err(e) = check("file.exportPng", &params) {
                    return ToolResult::error(e);
                }
                wrap(exec(b, "file.exportPng", params))
            } else {
                let params = match p {
                    Some(p) => json!({"path": p}),
                    None => json!({}),
                };
                if let Err(e) = check("file.save", &params) {
                    return ToolResult::error(e);
                }
                wrap(exec(b, "file.save", params))
            }
        }
        "type_text" => {
            let t = st("text").unwrap_or("");
            if a.get("paragraphs").and_then(Value::as_bool).unwrap_or(false) {
                let mut last = Ok(Value::Null);
                for (i, line) in t.split('\n').enumerate() {
                    if i > 0 {
                        last = exec(b, "text.newParagraph", json!({}));
                    }
                    if !line.is_empty() {
                        last = exec(b, "text.insert", json!({"text": line, "raw": true}));
                    }
                    if last.is_err() {
                        break;
                    }
                }
                wrap(last)
            } else {
                wrap(exec(b, "text.insert", json!({"text": t, "raw": true})))
            }
        }
        "select_text" => {
            wrap(exec(b, "select.text", json!({"text": st("text").unwrap_or(""), "occurrence": a.get("occurrence").cloned().unwrap_or(json!(1))})))
        }
        "get_text" => wrap(exec(b, "document.text", json!({}))),
        "inspect_document" => wrap(b.call("document.inspect", json!({}))),
        "parity" => wrap(b.call("ui.parity", json!({}))),
        "render_page" => {
            if b.has_ui() {
                let path = std::env::temp_dir().join("goharscribe-mcp-page.png");
                let path_s = path.to_string_lossy().to_string();
                match b.call("ui.render", json!({"path": path_s, "page": a.get("page").cloned().unwrap_or(json!(1)), "scale": a.get("scale").cloned().unwrap_or(json!(1.0))})) {
                    Ok(info) => match std::fs::read(&path) {
                        Ok(bytes) => ToolResult::image(goharscribe_engine::cmd::insert::base64_encode(&bytes), &info),
                        Err(e) => ToolResult::error(e.to_string()),
                    },
                    Err(e) => ToolResult::error(e),
                }
            } else {
                match b.call("ui.render", a.clone()) {
                    Ok(v) => {
                        let png = v.get("png").and_then(Value::as_str).unwrap_or("").to_string();
                        ToolResult::image(png, &json!({"width": v["width"], "height": v["height"], "pages": v["pages"]}))
                    }
                    Err(e) => ToolResult::error(e),
                }
            }
        }
        "screenshot" => {
            let path = std::env::temp_dir().join("goharscribe-mcp-shot.png");
            match b.call("ui.screenshot", json!({"path": path.to_string_lossy()})) {
                Ok(info) => match std::fs::read(&path) {
                    Ok(bytes) => ToolResult::image(goharscribe_engine::cmd::insert::base64_encode(&bytes), &info),
                    Err(e) => ToolResult::error(e.to_string()),
                },
                Err(e) => ToolResult::error(e),
            }
        }
        "click" => wrap(b.call("ui.click", a.clone())),
        "key" => wrap(b.call("ui.key", a.clone())),
        "ui_inspect" => wrap(b.call("ui.inspect", json!({}))),
        other => ToolResult::error(format!("unknown tool `{other}`")),
    }
}
