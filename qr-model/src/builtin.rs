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
            Self::BrowserTime => true,
            _ => false,
        }
    }
}
