use crate::db::{DbState, Query, Order};

pub async fn populate_schema(db: &DbState) -> anyhow::Result<()> {
    let query = Query::select(db, "*")
        .from("app_user")
        .where_eq("id", 1)
        .order_by("created_at", Order::Desc)
        .limit(1);

    println!("SQL: {}", query.build());

    let rows = query.get().await?;
    println!("Rows returned: {}", rows.len());

    Ok(())
}
