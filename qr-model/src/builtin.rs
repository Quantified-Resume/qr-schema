use rusqlite::ToSql;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

use crate::{Metrics, MetricsValueType};

// Builtin data type
#[derive(EnumString, AsRefStr, Clone, JsonSchema, Serialize, Deserialize, Debug, Display)]
pub enum Builtin {
    BrowserTime,
}

impl Builtin {
    pub fn is_multiple(&self) -> bool {
        match self {
            Builtin::BrowserTime => true,
        }
    }

    pub fn get_metrics_def(&self) -> Vec<Metrics> {
        match self {
            Builtin::BrowserTime => vec![
                Metrics::new("visit", "Visit count", MetricsValueType::Count, false),
                Metrics::new("focus", "Focus time", MetricsValueType::Time, false),
            ],
        }
    }

    pub fn get_default_name(&self) -> String {
        match self {
            Builtin::BrowserTime => String::from("Browser Time"),
        }
    }
}

impl ToSql for Builtin {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}
