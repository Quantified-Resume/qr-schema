use rusqlite::ToSql;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

// Builtin data type
#[derive(EnumString, AsRefStr, Clone, JsonSchema, Serialize, Deserialize, Debug)]
pub enum Builtin {
    BrowserTime,
}

impl Builtin {
    pub fn is_multiple(&self) -> bool {
        match self {
            Builtin::BrowserTime => true,
        }
    }
}

impl ToSql for Builtin {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}
