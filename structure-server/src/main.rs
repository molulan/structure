use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use structure_core::persistence::store::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Both are overridable so a test harness can boot a throwaway instance: a
    // fresh in-memory DB via `STRUCTURE_DB=:memory:` and a chosen free port via
    // `PORT`. Absent the vars, production defaults are unchanged.
    let db_path = env::var("STRUCTURE_DB").unwrap_or_else(|_| "structure.db".into());
    let store = Store::open(&db_path)?;

    // An explicit `STRUCTURE_WEB_DIR` without an `index.html` is a
    // misconfiguration — fail fast; an unset var serves the default build if
    // present, else runs API-only (as the test harnesses do).
    let web_dir = match env::var("STRUCTURE_WEB_DIR") {
        Ok(dir) if !dir.is_empty() => {
            let dir = PathBuf::from(dir);
            if !dir.join("index.html").is_file() {
                return Err(
                    format!("STRUCTURE_WEB_DIR has no index.html: {}", dir.display()).into(),
                );
            }
            Some(dir)
        }
        _ => {
            let default = PathBuf::from("web/dist");
            default.join("index.html").is_file().then_some(default)
        }
    };
    if let Some(dir) = &web_dir {
        println!("serving web app from {}", dir.display());
    }
    let app = structure_server::app(store, web_dir);

    let port: u16 = match env::var("PORT") {
        Ok(value) if !value.is_empty() => value.parse()?,
        _ => 3000,
    };
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
