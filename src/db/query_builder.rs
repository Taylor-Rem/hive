use crate::db::connect::DbState;
use sqlx::postgres::PgRow;

#[derive(Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Query<'a> {
    db: &'a DbState,
    select: String,
    from: String,
    where_clause: Option<String>,
    order_by: Option<(String, Order)>,
    limit: Option<u32>,
}

impl<'a> Query<'a> {
    pub fn select(db: &'a DbState, columns: &str) -> Self {
        Self {
            db,
            select: columns.to_string(),
            from: "".to_string(),
            where_clause: None,
            order_by: None,
            limit: None,
        }
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = table.to_string();
        self
    }

    pub fn where_eq(mut self, column: &str, value: impl ToString) -> Self {
        self.where_clause = Some(format!("{} = '{}'", column, value.to_string()));
        self
    }

    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        self.order_by = Some((column.to_string(), order));
        self
    }

    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn build(&self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select, self.from);

        if let Some(where_clause) = &self.where_clause {
            query.push_str(&format!(" WHERE {}", where_clause));
        }

        if let Some((col, ord)) = &self.order_by {
            let ord_str = match ord {
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            };
            query.push_str(&format!(" ORDER BY {} {}", col, ord_str));
        }

        if let Some(limit) = &self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query
    }

    // Build + execute
    pub async fn get(&self) -> anyhow::Result<Vec<PgRow>> {
        let sql = self.build();
        let rows = sqlx::query(&sql)
            .fetch_all(&self.db.pool)
            .await?;
        Ok(rows)
    }
}
