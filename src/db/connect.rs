use tokio_postgres::{Client, NoTls};
use std::sync::Arc;
use anyhow::Result;

#[derive(Clone)]
pub struct DbState {
    pub client: Arc<Client>,
}

impl DbState {
    pub async fn connect() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let db_host = std::env::var("DB_HOST")
            .expect("DB_HOST must be set in .env file");
        let db_user = std::env::var("DB_USER")
            .expect("DB_USER must be set in .env file");
        let db_pass = std::env::var("DB_PASS")
            .expect("DB_PASS must be set in .env file");
        let db_name = std::env::var("DB_NAME")
            .expect("DB_NAME must be set in .env file");

        // Build the connection string
        let database_url = format!(
            "host={} user={} password={} dbname={}",
            db_host, db_user, db_pass, db_name
        );

        // Connect to the database
        let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

        // Spawn the connection in the background
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        println!("✅ Connected to database at {}", db_host);

        Ok(DbState {
            client: Arc::new(client),
        })
    }
}