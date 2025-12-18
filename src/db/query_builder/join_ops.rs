extern crate paste;

macro_rules! join_ops {
    ($($name:ident => $sql:expr),+ $(,)?) => {
        paste::paste! {
            $(
                pub fn $name(mut self, table: &str, on: &str) -> Self {
                    self.joins.push(format!("{} {} ON {}", $sql, table, on));
                    self
                }
            )+
        }
    };
}

pub(crate) use join_ops;
