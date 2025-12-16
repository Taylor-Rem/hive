use sqlx::postgres::PgPool;
use urlencoding::encode;

#[derive(Debug, Clone)]
pub struct DbState {
    pub pool: PgPool,
}

impl DbState {
    pub async fn connect() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        let db_host = std::env::var("DB_HOST")?.trim().to_string();
        let db_user = std::env::var("DB_USER")?.trim().to_string();
        let db_pass = std::env::var("DB_PASS")?.trim().to_string();
        let db_name = std::env::var("DB_NAME")?.trim().to_string();
        let db_port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .trim()
            .to_string();

        // URL-encode the password to handle special characters
        let encoded_pass = encode(&db_pass);

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, encoded_pass, db_host, db_port, db_name
        );
        
        println!("Connecting to database at {}...", db_host);

        let pool = PgPool::connect(&database_url).await?;

        println!("✅ Connected to database at {}", db_host);

        Ok(DbState { pool })
    }
}