extern crate serde;
use serde::{de, Deserialize};
use std::result::Result as StdResult;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Input {
    name: String,
    #[serde(rename(deserialize = "VALUE"))]
    #[serde(deserialize_with = "parse_input_friendly")]
    friendly_name: String,
    hashval: u32,
}

impl Input {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn friendly_name(&self) -> String {
        self.friendly_name.clone()
    }

    pub(crate) fn hashval(&self) -> u32 {
        self.hashval
    }
}

fn parse_input_friendly<'de, D>(deserializer: D) -> StdResult<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    let mut value = serde_json::Value::deserialize(deserializer)?;
    serde_json::from_value::<String>(value.clone())
    .or_else(|_|
        serde_json::from_value::<String>(value["NAME"].take())
        .map_err(|_|
            de::Error::missing_field("NAME")
        )
    )

}
