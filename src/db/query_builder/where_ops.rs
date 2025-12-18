extern crate paste;

macro_rules! where_ops {
    ($($name:ident => $op:expr),+ $(,)?) => {
        paste::paste! {
            $(
                pub fn [<where_ $name>](mut self, column: &str, value: impl ToString) -> Self {
                    self.wheres.push(format!("{} {} '{}'", column, $op, value.to_string()));
                    self
                }
            )+
        }
    };
}

pub(crate) use where_ops;