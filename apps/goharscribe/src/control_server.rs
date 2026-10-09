//! Loopback JSON-lines control server: one request per line, one reply per line.
//! This is the transport the MCP server (`goharscribe mcp --connect`) wraps.
//!
//! Every request must carry the bearer token in its `"token"` field. The token is
//! generated fresh at startup (printed to stderr) unless `GOHARSCRIBE_CONTROL_TOKEN`
//! is set, so another local process can't silently drive the app.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use goharscribe_ui_egui::ControlRequest;
use serde_json::{Value, json};

/// Bearer token for the control channel: `GOHARSCRIBE_CONTROL_TOKEN` when set,
/// otherwise 32 random bytes from the OS RNG, hex-encoded.
fn make_token() -> String {
    let env_token = std::env::var("GOHARSCRIBE_CONTROL_TOKEN").ok();
    make_token_from(env_token.as_deref())
}

fn make_token_from(env_token: Option<&str>) -> String {
    if let Some(t) = env_token.map(str::trim).filter(|t| !t.is_empty()) {
        return t.to_string();
    }
    let mut bytes = [0u8; 32];
    if getrandom::fill(&mut bytes).is_err() {
        // Practically unreachable; fall back to a time/pid-seeded xorshift so we
        // never run without a token.
        let mut s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E37_79B9_7F4A_7C15)
            ^ (std::process::id() as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        for b in bytes.iter_mut() {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            *b = (s >> 56) as u8;
        }
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time token comparison (lengths must match first).
fn tokens_match(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn start(port: u16, ctx: egui::Context) -> Receiver<ControlRequest> {
    let (tx, rx) = channel::<ControlRequest>();
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("goharscribe: control server failed to bind 127.0.0.1:{port}: {e}");
            return rx;
        }
    };
    let token = Arc::new(make_token());
    eprintln!("goharscribe: control server listening on 127.0.0.1:{port}");
    eprintln!("goharscribe: control token: {}", token.as_str());
    eprintln!("goharscribe: (pass it as the \"token\" field, --token, or GOHARSCRIBE_CONTROL_TOKEN)");
    let token_accept = token.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let tx = tx.clone();
            let ctx = ctx.clone();
            let token = token_accept.clone();
            std::thread::spawn(move || serve(stream, tx, ctx, token));
        }
    });
    rx
}

fn serve(stream: TcpStream, tx: Sender<ControlRequest>, ctx: egui::Context, token: Arc<String>) {
    let Ok(read) = stream.try_clone() else { return };
    let mut out = stream;
    for line in BufReader::new(read).lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(msg) => {
                let id = msg.get("id").cloned().unwrap_or(Value::Null);
                let provided = msg.get("token").and_then(Value::as_str).unwrap_or("");
                if !tokens_match(provided, &token) {
                    let mut r = json!({"ok": false, "error": "bad or missing control token"});
                    if let Some(o) = r.as_object_mut() {
                        o.insert("id".into(), id);
                    }
                    r
                } else {
                    let method = msg.get("method").and_then(Value::as_str).unwrap_or("").to_string();
                    let params = msg.get("params").cloned().unwrap_or(json!({}));
                    let (req, rrx) = ControlRequest::new(method, params);
                    if tx.send(req).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                    let mut r = rrx.recv_timeout(Duration::from_secs(60)).unwrap_or_else(|_| json!({"ok": false, "error": "timeout"}));
                    if let Some(o) = r.as_object_mut() {
                        o.insert("id".into(), id);
                    }
                    r
                }
            }
            Err(e) => json!({"ok": false, "error": format!("bad JSON: {e}")}),
        };
        if writeln!(out, "{reply}").is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_compare_is_exact() {
        assert!(tokens_match("abc", "abc"));
        assert!(!tokens_match("abc", "abd"));
        assert!(!tokens_match("abc", "ab"));
        assert!(!tokens_match("abc", "abcd"));
        assert!(!tokens_match("", "abc"));
    }

    #[test]
    fn generated_tokens_are_unique() {
        // Env override takes precedence when set; empty falls back to generated.
        assert_eq!(make_token_from(Some("  fixed  ")), "fixed");
        assert_eq!(make_token_from(Some("")).len(), 64);
        let a = make_token_from(None);
        let b = make_token_from(None);
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
    }
}
