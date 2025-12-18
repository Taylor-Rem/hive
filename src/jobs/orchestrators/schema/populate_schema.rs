use crate::db::{DbState, Query, Order};
use anyhow::Context;

pub async fn populate_schema(db: &DbState) -> anyhow::Result<()> {
    let query = Query::select(db, "*")
        .from("app_user AS u")
        .join("cart AS c", "c.app_user_id = u.id")
        .where_eq("u.id", 1)
        .order_by("u.created_at", Order::Desc)
        .limit(1);

    println!("SQL: {}", query.build());

    // Execute query with context for error handling
    let rows = query
        .get()
        .await
        .with_context(|| format!("Failed to execute query: {}", query.build()))?;

    if rows.is_empty() {
        println!("No rows returned for query: {}", query.build());
    } else {
        println!("Rows returned: {}", rows.len());
    }

    Ok(())
}
