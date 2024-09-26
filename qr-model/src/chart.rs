use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

#[derive(AsRefStr, Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq, Eq)]
pub enum ChartSeries {
    CalendarHeat,
    Line,
}
