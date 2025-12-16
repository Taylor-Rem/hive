use crate::db::connect::DbState;

pub async fn scan_db(db: &DbState) -> anyhow::Result<()> {
    println!("Scanning DB");
    Ok(())
}