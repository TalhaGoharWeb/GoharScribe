//! An MCP client doing realistic tasks through the protocol only, checking results via MCP.

use serde_json::{Value, json};

use crate::{Headless, Server};

fn call(s: &mut Server, id: u64, method: &str, params: Value) -> Value {
    let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
    let r = s.handle_line(&line).expect("reply");
    serde_json::from_str(&r).expect("json")
}

fn tool(s: &mut Server, name: &str, args: Value) -> Value {
    let r = call(s, 9, "tools/call", json!({"name": name, "arguments": args}));
    let res = &r["result"];
    assert_eq!(res["isError"], false, "{name}: {res}");
    let text = res["content"][0]["text"].as_str().unwrap_or("null");
    serde_json::from_str(text).unwrap_or(Value::String(text.to_string()))
}

#[test]
fn lifecycle_and_tools() {
    let mut s = Server::new(Box::new(Headless::default()));
    let init = call(&mut s, 1, "initialize", json!({"protocolVersion": "2025-06-18"}));
    assert_eq!(init["result"]["serverInfo"]["name"], "goharscribe");
    assert!(s.handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
    let tools = call(&mut s, 2, "tools/list", json!({}));
    assert!(tools["result"]["tools"].as_array().unwrap().len() >= 12);
    let bad = call(&mut s, 3, "nope", json!({}));
    assert!(bad["error"].is_object());
    assert!(s.handle_line("{not json").unwrap().contains("parse error"));
}

#[test]
fn agent_writes_a_formatted_document() {
    let mut s = Server::new(Box::new(Headless::default()));
    tool(&mut s, "new_document", json!({}));
    tool(&mut s, "type_text", json!({"text": "Quarterly Report"}));
    tool(&mut s, "execute", json!({"command": "para.style", "params": {"style": "Heading 1"}}));
    tool(&mut s, "execute", json!({"command": "text.newParagraph"}));
    tool(&mut s, "type_text", json!({"text": "Sales grew strongly this quarter.\nCosts fell.", "paragraphs": true}));
    tool(&mut s, "select_text", json!({"text": "strongly"}));
    tool(&mut s, "execute", json!({"command": "format.bold"}));
    tool(&mut s, "batch", json!({"commands": [{"command": "caret.docEnd"}, {"command": "insert.table", "params": {"rows": 2, "cols": 2}}]}));
    let doc = tool(&mut s, "inspect_document", json!({}));
    let blocks = doc["blocks"].as_array().unwrap();
    assert_eq!(blocks[0]["style"], "Heading1");
    assert_eq!(blocks[0]["text"], "Quarterly Report");
    let runs = blocks[1]["runs"].as_array().unwrap();
    assert!(runs.iter().any(|r| r["props"]["bold"] == true));
    assert!(blocks.iter().any(|b| b["type"] == "table"));
    let text = tool(&mut s, "get_text", json!({}));
    assert!(text["text"].as_str().unwrap().contains("Costs fell."));
    let r = call(&mut s, 10, "tools/call", json!({"name": "render_page", "arguments": {"page": 1, "scale": 0.5}}));
    assert_eq!(r["result"]["content"][0]["type"], "image");
    let r = call(&mut s, 11, "tools/call", json!({"name": "screenshot", "arguments": {}}));
    assert_eq!(r["result"]["isError"], true);
    let p = tool(&mut s, "parity", json!({}));
    assert!(p["percent"].as_f64().unwrap() > 50.0);
}

#[test]
fn resources() {
    let mut s = Server::new(Box::new(Headless::default()));
    let l = call(&mut s, 1, "resources/list", json!({}));
    assert_eq!(l["result"]["resources"].as_array().unwrap().len(), 2);
    let r = call(&mut s, 2, "resources/read", json!({"uri": "goharscribe://document"}));
    assert!(r["result"]["contents"][0]["text"].as_str().unwrap().contains("blocks"));
}

/// Call a tool without asserting success; returns the raw result value.
fn tool_raw(s: &mut Server, name: &str, args: Value) -> Value {
    let r = call(s, 9, "tools/call", json!({"name": name, "arguments": args}));
    r["result"].clone()
}

#[test]
fn jail_blocks_paths_outside() {
    let jail = std::env::temp_dir().join("goharscribe-jail-test");
    std::fs::create_dir_all(&jail).unwrap();
    let mut s = crate::Server::with_jail(Box::new(Headless::default()), Some(jail.clone()));

    // Outside the jail: rejected with a jail error, never reaching the backend.
    let outside = tool_raw(&mut s, "open_document", json!({"path": "/etc/hostname"}));
    assert_eq!(outside["isError"], true);
    assert!(outside["content"][0]["text"].as_str().unwrap_or("").contains("outside the allowed directory"));

    // Via the generic execute tool too.
    let outside2 = tool_raw(&mut s, "execute", json!({"command": "file.open", "params": {"path": "/etc/hostname"}}));
    assert_eq!(outside2["isError"], true);

    // Inside the jail: passes the jail (the backend then fails on the missing file,
    // which proves the jail let it through).
    let inside_path = jail.join("doc.gohar").to_string_lossy().to_string();
    let inside = tool_raw(&mut s, "open_document", json!({"path": inside_path}));
    let text = inside["content"][0]["text"].as_str().unwrap_or("");
    assert!(!text.contains("outside the allowed directory"), "{text}");

    // `..` can't escape.
    let escape = tool_raw(&mut s, "open_document", json!({"path": jail.join("../escape").to_string_lossy()}));
    assert_eq!(escape["isError"], true);

    std::fs::remove_dir_all(&jail).ok();
}

#[test]
fn jail_allows_unit() {
    use crate::tools::{PATH_COMMANDS, jail_allows};
    assert!(PATH_COMMANDS.contains(&"file.open"));
    let jail = std::env::temp_dir().join("goharscribe-jail-unit");
    std::fs::create_dir_all(&jail).unwrap();
    let canon = jail.canonicalize().unwrap();
    // Inside.
    assert!(jail_allows(&canon, &canon.join("a.gohar").to_string_lossy()));
    // Outside.
    assert!(!jail_allows(&canon, "/etc/hostname"));
    // `..` escape.
    assert!(!jail_allows(&canon, &format!("{}/../x", canon.to_string_lossy())));
    std::fs::remove_dir_all(&jail).ok();
}

#[test]
fn h2_jail_covers_all_path_commands() {
    // H2: All path-taking commands must be in PATH_COMMANDS.
    use crate::tools::PATH_COMMANDS;
    for cmd in
        ["insert.textFromFile", "insert.picture", "picture.change", "review.compare", "review.combine", "mailings.recipients", "mailings.finish"]
    {
        assert!(PATH_COMMANDS.contains(&cmd), "{cmd} must be jailed");
    }
}

#[test]
fn h2_jail_blocks_textfromfile_outside() {
    let jail = std::env::temp_dir().join("goharscribe-jail-h2");
    std::fs::create_dir_all(&jail).unwrap();
    let mut s = crate::Server::with_jail(Box::new(Headless::default()), Some(jail.clone()));
    let r = tool_raw(&mut s, "execute", json!({"command": "insert.textFromFile", "params": {"path": "/etc/hostname"}}));
    assert_eq!(r["isError"], true);
    assert!(r["content"][0]["text"].as_str().unwrap_or("").contains("outside the allowed directory"));
    std::fs::remove_dir_all(&jail).ok();
}
