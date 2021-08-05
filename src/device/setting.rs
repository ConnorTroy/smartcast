use super::Result;
use super::Error;

use serde_json::Value;
use serde::{de, Deserialize};

use std::result::Result as StdResult;

const STATIC_BASE: &str = "/menu_native/static/";
const DYNAMIC_BASE: &str = "/menu_native/dynamic/";

pub trait SettingValue<T> {
    fn value(&self) -> Result<T>;
}

pub enum UrlBase {
    Dynamic,
    Static,
    None,
}

impl UrlBase {
    fn base(&self) -> String {
        match self {
            Self::Static => STATIC_BASE,
            Self::Dynamic => DYNAMIC_BASE,
            Self::None => "",
        }.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Slider,
    List,
    Value,
    Menu,
    XList,
    Other(String),
}

/// Deserializer for [`ObjectType`]
impl<'de> Deserialize<'de> for ObjectType {
    fn deserialize<D>(deserializer: D) -> StdResult<ObjectType, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(
            match String::deserialize(deserializer)?.as_str() {
                "T_VALUE_ABS_V1" => ObjectType::Slider,
                "T_LIST_V1" => ObjectType::List,
                "T_VALUE_V1" => ObjectType::Value,
                "T_MENU_V1" => ObjectType::Menu,
                "T_LIST_X_V1" => ObjectType::XList,
                other => ObjectType::Other(other.into()),
            }
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SliderInfo {
    #[serde(rename = "DECMARKER")]
    pub dec_marker: String,
    #[serde(rename = "INCMARKER")]
    pub inc_marker: String,
    pub increment: i32,
    #[serde(rename = "MAXIMUM")]
    pub max: i32,
    #[serde(rename = "MINIMUM")]
    pub min: i32,
    pub center: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SubSetting {
    #[serde(rename = "CNAME")]
    endpoint: String,
    hashval: Option<u32>,
    #[serde(deserialize_with = "string_to_bool", default)]
    hidden: bool,
    name: String,
    #[serde(deserialize_with = "string_to_bool", default)]
    readonly: bool,
    #[serde(rename = "TYPE")]
    object_type: ObjectType,
    value: Option<Value>,
    slider_info: Option<SliderInfo>,
    elements: Option<Vec<String>>,
}

impl SubSetting {

    pub fn top_level() -> Self {
        Self::default()
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    pub fn object_type(&self) -> ObjectType {
        self.object_type.clone()
    }

    pub fn slider_info(&self) -> Option<SliderInfo> {
        self.slider_info.clone()
    }

    pub fn elements(&self) -> Option<Vec<String>> {
        self.elements.clone()
    }

    pub(crate) fn endpoint(&self, endpoint_base: UrlBase, device_type: &super::DeviceType) -> String {
        format!("{}/{}/{}", endpoint_base.base(), device_type.endpoint(), self.endpoint)
    }

    pub(crate) fn add_parent_endpoint(&mut self, parent_endpoint: String) {
        if parent_endpoint.as_str() != "" {
            self.endpoint.insert_str(0, &(parent_endpoint + "/"));
        }
    }

    pub(crate) fn add_slider_info(&mut self, slider_info: SliderInfo) {
        self.slider_info = Some(slider_info);
    }

    pub(crate) fn add_elements(&mut self, elements: Vec<String>) {
        self.elements = Some(elements);
    }
}

impl SettingValue<String> for SubSetting {
    fn value(&self) -> Result<String> {
        match &self.value {
            Some(value) => Ok(serde_json::from_value(value.clone())?),
            None => Err(Error::Other("Value is None".into())),
        }
    }
}

impl SettingValue<Value> for SubSetting {
    fn value(&self) -> Result<Value> {
        match &self.value {
            Some(value) => Ok(value.clone()),
            None => Err(Error::Other("Value is None".into())),
        }
    }
}

impl Default for SubSetting {
    fn default() -> Self {
        Self {
            endpoint:       "".into(),
            hashval:        None,
            hidden:         false,
            name:           "TV Settings".into(),
            readonly:       false,
            object_type:    ObjectType::Menu,
            value:          None,
            slider_info:    None,
            elements:       None,
        }
    }
}

fn string_to_bool<'de, D>(deserializer: D) -> StdResult<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    string
        .to_lowercase()
        .parse::<bool>().map_err(|_| de::Error::invalid_type(
            de::Unexpected::Str(&string),
            &"a boolean"
        ))
}
