use crate::db::connect::DbState;
use sqlx::postgres::PgRow;

use super::where_ops::where_ops;
use super::join_ops::join_ops;

#[derive(Debug)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct Query<'a> {
    pub(super) db: &'a DbState,
    pub(super) select: String,
    pub(super) from: String,
    pub(super) joins: Vec<String>,
    pub(super) wheres: Vec<String>,
    pub(super) order_by: Option<(String, Order)>,
    pub(super) limit: Option<u32>,
}

impl<'a> Query<'a> {
    pub fn select(db: &'a DbState, columns: &str) -> Self {
        Self {
            db,
            select: columns.to_string(),
            from: String::new(),
            joins: Vec::new(),
            wheres: Vec::new(),
            order_by: None,
            limit: None,
        }
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = table.to_string();
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
    
        for join in &self.joins {
            query.push_str(&format!(" {}", join));
        }
    
        if !self.wheres.is_empty() {
            query.push_str(&format!(" WHERE {}", self.wheres.join(" AND ")));
        }
    
        if let Some((col, ord)) = &self.order_by {
            let ord = match ord {
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            };
            query.push_str(&format!(" ORDER BY {} {}", col, ord));
        }
    
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
    
        query
    }
    

    pub async fn get(&self) -> anyhow::Result<Vec<PgRow>> {
        let sql = self.build();
        let rows = sqlx::query(&sql)
            .fetch_all(&self.db.pool)
            .await?;
        Ok(rows)
    }

    // generated methods
    where_ops! {
        eq  => "=",
        ne  => "!=",
        gt  => ">",
        gte => ">=",
        lt  => "<",
        lte => "<=",
    }

    join_ops! {
        join => "JOIN",
        left_join => "LEFT JOIN"
    }
}
