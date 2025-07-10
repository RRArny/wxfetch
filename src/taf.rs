use colored::ColoredString;
use serde_json::Value;

use crate::config::Config;

pub struct Taf {}

impl Taf {
    pub fn from_json(json: &Value, config: &Config) -> Option<Self> {
        todo!("Taf::from_json");
    }
    pub fn colourise(&self, config: &Config) -> ColoredString {
        todo!("Taf::colourise");
    }
}
