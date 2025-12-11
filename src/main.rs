use hive::db::connect::DbState;
use hive::server::server::create_server;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _db_state = DbState::connect().await?;
    let app = create_server(_db_state);
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}