use std::net::SocketAddr;

use structure_core::persistence::store::Store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open("structure.db")?;
    let app = structure_server::router(store);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
