use hive::db::connect::DbState;
use hive::server::server::create_server;
use hive::state::app_state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = DbState::connect().await?;
    let state = AppState { db };
    let app = create_server(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}