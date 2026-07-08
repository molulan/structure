use std::env;
use std::net::SocketAddr;

use structure_core::persistence::store::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Both are overridable so a test harness can boot a throwaway instance: a
    // fresh in-memory DB via `STRUCTURE_DB=:memory:` and a chosen free port via
    // `PORT`. Absent the vars, production defaults are unchanged.
    let db_path = env::var("STRUCTURE_DB").unwrap_or_else(|_| "structure.db".into());
    let store = Store::open(&db_path)?;
    let app = structure_server::router(store);

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
