//! `goharscribe-server`: the GoharScribe SaaS backend.
//!
//! ```text
//! GOHARSCRIBE_JWT_SECRET=<32+ random bytes> goharscribe-server [--db path] [--port N] [--host H]
//! ```
//!
//! The JWT secret is required in production. If unset, the server refuses to
//! start unless `--dev` is passed (insecure fixed secret, local testing only).

use std::sync::{Arc, Mutex};

use goharscribe_server::{AppState, Db, auth::Auth, router};

fn print_usage() -> ! {
    eprintln!("usage: goharscribe-server [--db PATH] [--port N] [--host H] [--dev]");
    std::process::exit(2);
}

#[tokio::main]
async fn main() {
    let mut db_path = "goharscribe-saas.db".to_string();
    let mut port: u16 = 8080;
    let mut host = "127.0.0.1".to_string();
    let mut dev = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => db_path = args.next().unwrap_or_else(|| print_usage()),
            "--port" => {
                let p = args.next().unwrap_or_else(|| print_usage());
                port = p.parse().unwrap_or_else(|_| print_usage());
            }
            "--host" => host = args.next().unwrap_or_else(|| print_usage()),
            "--dev" => dev = true,
            _ => print_usage(),
        }
    }

    let secret: Vec<u8> = match std::env::var("GOHARSCRIBE_JWT_SECRET") {
        Ok(s) if s.len() >= 32 => s.into_bytes(),
        _ if dev => {
            eprintln!("WARNING: --dev mode, using insecure fixed JWT secret");
            b"dev-only-insecure-secret-0123456789".to_vec()
        }
        _ => {
            eprintln!("error: set GOHARSCRIBE_JWT_SECRET to 32+ random bytes (or pass --dev for local testing)");
            std::process::exit(1);
        }
    };

    let db = Db::open(&db_path).unwrap_or_else(|e| {
        eprintln!("error: cannot open database {db_path}: {e}");
        std::process::exit(1);
    });
    let state = AppState { db: Arc::new(Mutex::new(db)), auth: Auth::new(&secret) };
    let app = router(state);
    let addr = format!("{host}:{port}");
    eprintln!("GoharScribe SaaS listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("error: cannot bind {addr}: {e}");
        std::process::exit(1);
    });
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("error: server failed: {e}");
        std::process::exit(1);
    });
}
